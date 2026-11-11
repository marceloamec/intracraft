//! Turns a code block into instructions for the interpreter and checks it against the model before any run (A17 §13.1):
//! unknown names and members (with a suggestion), events on objects that don't have them, `wait` where nothing may
//! wait, Python's scoping traps. Every problem is a [`Diagnostic`] with its line; a block with errors has no program.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ast::{BinOp, CmpOp, Expr, ExprKind, FPart, FuncDef, Stmt, StmtKind, UnOp};
use crate::builtins::{self, Builtin};
use crate::diag::Diagnostic;
use crate::host::{MemberKind, ModelInfo, PALLET_MEMBERS};
use crate::ops::binary;
use crate::parser::parse;
use crate::units::{self, Dim};
use crate::value::Value;

/// A target an instruction jumps to, before it is known.
const PENDING: u32 = u32::MAX;
/// "No format spec" / "no keyword names".
pub const NONE: u32 = u32::MAX;
/// The modules code may import (they are built in; importing only documents).
const IMPORTABLE: [&str; 3] = ["intrascript", "math", "random"];
/// What a handler may take: the pallet of its event, and the object that raised it.
const HANDLER_PARAMS: [&str; 2] = ["pallet", "sender"];
/// The event decorators, for messages and the editor.
pub const DECORATORS: [(&str, &str); 11] = [
    ("on_start", "@on_start: when the run starts, after the block's variables get their values"),
    ("on_finish", "@on_finish: when the run ends; it can't wait"),
    ("on_create", "@on_create(Source1): a source made a pallet"),
    ("on_enter", "@on_enter(Roller3): a pallet now belongs to the object"),
    ("on_leave", "@on_leave(Roller3): a pallet no longer belongs to the object"),
    ("on_rise", "@on_rise(Photoeye3.detected): a signal turns True"),
    ("on_fall", "@on_fall(Photoeye3.detected): a signal turns False"),
    ("on_change", "@on_change(Photoeye3.detected): a signal changes"),
    ("every", "@every(10 * minute, start=0 * s): again and again"),
    ("at", "@at(2 * h): once, at that time"),
    ("choose_exit", "@choose_exit(Turntable1): return the object the pallet goes to; it can't wait"),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Const(u32),
    LoadLocal(u32),
    StoreLocal(u32),
    LoadGlobal(u32),
    StoreGlobal(u32),
    LoadObject(u32),
    LoadBuiltin(Builtin),
    LoadFunc(u32),
    GetAttr(u32),
    SetAttr(u32),
    GetItem,
    SetItem,
    BuildSlice,
    Unary(UnOp),
    Binary(BinOp),
    Compare(CmpOp),
    Jump(u32),
    JumpIfFalse(u32),
    JumpIfTrue(u32),
    /// Jumps keeping the value when it is false; pops it otherwise (`and`).
    JumpIfFalseKeep(u32),
    JumpIfTrueKeep(u32),
    Pop,
    Dup,
    Dup2,
    Swap,
    /// Moves the top value under the next two.
    Rot3,
    BuildList(u32),
    BuildTuple(u32),
    BuildDict(u32),
    /// Appends the top value to the list `n` places down (a list comprehension).
    ListAppend(u32),
    Unpack(u32),
    GetIter,
    /// The next item, or (iterator popped) a jump when there is none.
    ForIter(u32),
    /// Positional arguments, and the constant tuple of keyword names (or `NONE`).
    Call(u32, u32),
    Return,
    Wait,
    WaitUntilStart(u32),
    WaitUntilPause(u32, u32),
    Format(u32),
    BuildString(u32),
    Assert,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    /// Every local variable's name, by slot (parameters first).
    pub locals: Vec<String>,
    /// The default values of the last parameters.
    pub defaults: Vec<Value>,
    pub nlocals: u32,
    pub code: Vec<Op>,
    /// The line of each instruction.
    pub lines: Vec<u32>,
    pub doc: Option<String>,
    pub line: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Rise,
    Fall,
    Change,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventSpec {
    Start,
    Finish,
    Create(u32),
    Enter(u32),
    Leave(u32),
    Signal {
        object: u32,
        signal: String,
        edge: Edge,
    },
    /// Every `interval` µs, the first at `first` µs.
    Every {
        interval: i64,
        first: i64,
    },
    At(i64),
    ChooseExit(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Handler {
    pub event: EventSpec,
    pub func: u32,
    /// Higher runs first at the same instant.
    pub priority: i64,
    pub line: u32,
}

/// A checked code block, ready to run.
#[derive(Clone, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    /// The function that gives the block's variables their first values.
    pub init: u32,
    pub globals: Vec<String>,
    pub consts: Vec<Value>,
    pub names: Vec<Arc<str>>,
    pub handlers: Vec<Handler>,
    /// Whether each function may wait (itself or through what it calls).
    pub waits: Vec<bool>,
}

impl Program {
    pub fn function(&self, name: &str) -> Option<u32> {
        self.functions.iter().position(|f| f.name == name).and_then(|i| u32::try_from(i).ok())
    }

    pub fn global(&self, name: &str) -> Option<u32> {
        self.globals.iter().position(|g| g == name).and_then(|i| u32::try_from(i).ok())
    }
}

/// A code block to compile: the object it is in the model, and its code.
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    pub object: u32,
    pub code: &'a str,
}

/// The compiled block, or none when it has errors, and everything the checker found.
#[derive(Clone, Debug)]
pub struct Compiled {
    pub program: Option<Program>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Compiles every code block of a model together, so blocks can use each other's functions and variables.
pub fn compile_blocks(model: &ModelInfo, sources: &[Source]) -> Vec<Compiled> {
    let parsed: Vec<Result<Vec<Stmt>, Diagnostic>> = sources.iter().map(|s| parse(s.code)).collect();
    let mut exports: BTreeMap<u32, Option<Vec<String>>> = BTreeMap::new();
    for (s, p) in sources.iter().zip(&parsed) {
        exports.insert(s.object, p.as_ref().ok().map(|body| top_names(body).into_iter().map(|(n, _)| n).collect()));
    }
    sources
        .iter()
        .zip(parsed)
        .map(|(_, p)| match p {
            Err(d) => Compiled { program: None, diagnostics: vec![d] },
            Ok(body) => {
                let mut c = Compiler::new(model, &exports);
                let program = c.module(&body);
                let errors = c.diags.iter().any(Diagnostic::is_error);
                c.diags.sort_by_key(|d| (d.line, d.col));
                Compiled { program: if errors { None } else { Some(program) }, diagnostics: c.diags }
            }
        })
        .collect()
}

/// The checker alone, for one block (the editor checks while the code is typed).
pub fn check(model: &ModelInfo, object: u32, code: &str, others: &[Source]) -> Vec<Diagnostic> {
    let mut all: Vec<Source> = others.iter().copied().filter(|s| s.object != object).collect();
    all.push(Source { object, code });
    compile_blocks(model, &all).pop().map(|c| c.diagnostics).unwrap_or_default()
}

/// The names a block defines at its top: variables, functions, and names its functions declare `global`.
fn top_names(body: &[Stmt]) -> Vec<(String, bool)> {
    let mut names: Vec<(String, bool)> = Vec::new();
    let add = |n: &str, is_def: bool, names: &mut Vec<(String, bool)>| {
        if !names.iter().any(|(m, _)| m == n) {
            names.push((n.to_string(), is_def));
        }
    };
    for s in body {
        if let StmtKind::Def(f) = &s.kind {
            add(&f.name, true, &mut names);
        }
    }
    for s in body {
        match &s.kind {
            StmtKind::Assign { targets, .. } => {
                for t in targets {
                    for n in target_names(t) {
                        add(&n, false, &mut names);
                    }
                }
            }
            StmtKind::AugAssign { target, .. } | StmtKind::AnnAssign { target, .. } => {
                for n in target_names(target) {
                    add(&n, false, &mut names);
                }
            }
            StmtKind::Def(f) => {
                for n in global_decls(&f.body) {
                    add(&n, false, &mut names);
                }
            }
            _ => {}
        }
    }
    names
}

fn target_names(t: &Expr) -> Vec<String> {
    match &t.kind {
        ExprKind::Name(n) => vec![n.clone()],
        ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().flat_map(target_names).collect(),
        _ => Vec::new(),
    }
}

fn global_decls(body: &[Stmt]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk(body, &mut |s| {
        if let StmtKind::Global(names) = &s.kind {
            out.extend(names.iter().cloned());
        }
    });
    out
}

/// The names a function assigns (its locals, Python's rule), in order of appearance.
fn assigned_names(body: &[Stmt]) -> Vec<(String, u32, u32)> {
    let mut out: Vec<(String, u32, u32)> = Vec::new();
    walk(body, &mut |s| {
        let targets: Vec<&Expr> = match &s.kind {
            StmtKind::Assign { targets, .. } => targets.iter().collect(),
            StmtKind::AugAssign { target, .. } | StmtKind::AnnAssign { target, .. } | StmtKind::For { target, .. } => vec![target],
            _ => Vec::new(),
        };
        for t in targets {
            for n in target_names(t) {
                if !out.iter().any(|(m, ..)| *m == n) {
                    out.push((n, s.line, s.col));
                }
            }
        }
    });
    out
}

/// Visits every statement, inside blocks too (not inside nested functions).
fn walk(body: &[Stmt], f: &mut dyn FnMut(&Stmt)) {
    for s in body {
        f(s);
        match &s.kind {
            StmtKind::If { body, orelse, .. } => {
                walk(body, f);
                walk(orelse, f);
            }
            StmtKind::While { body, .. } | StmtKind::For { body, .. } => walk(body, f),
            _ => {}
        }
    }
}

enum Resolved {
    Local(u32),
    Global(u32),
    Object(u32),
    Builtin(Builtin),
    Unit(f64, Dim),
    Unknown,
}

struct Loop {
    start: usize,
    breaks: Vec<usize>,
    is_for: bool,
}

#[derive(Default)]
struct FnState {
    code: Vec<Op>,
    lines: Vec<u32>,
    locals: Vec<String>,
    local_map: BTreeMap<String, u32>,
    params: usize,
    global_decl: BTreeSet<String>,
    comp: Vec<(String, u32)>,
    loops: Vec<Loop>,
    module: bool,
    waits: bool,
    calls: BTreeSet<u32>,
    loaded: BTreeSet<u32>,
    line: u32,
    /// The handler's events, when the function is one (to say why it can't wait).
    no_wait: Option<&'static str>,
}

struct Compiler<'a> {
    model: &'a ModelInfo,
    exports: &'a BTreeMap<u32, Option<Vec<String>>>,
    globals: Vec<String>,
    defs: BTreeMap<String, u32>,
    consts: Vec<Value>,
    names: Vec<Arc<str>>,
    functions: Vec<Function>,
    handlers: Vec<Handler>,
    calls: Vec<BTreeSet<u32>>,
    direct_waits: Vec<bool>,
    diags: Vec<Diagnostic>,
    f: FnState,
}

impl<'a> Compiler<'a> {
    fn new(model: &'a ModelInfo, exports: &'a BTreeMap<u32, Option<Vec<String>>>) -> Compiler<'a> {
        Compiler {
            model,
            exports,
            globals: Vec::new(),
            defs: BTreeMap::new(),
            consts: Vec::new(),
            names: Vec::new(),
            functions: Vec::new(),
            handlers: Vec::new(),
            calls: Vec::new(),
            direct_waits: Vec::new(),
            diags: Vec::new(),
            f: FnState::default(),
        }
    }

    fn error(&mut self, code: &str, line: u32, col: u32, msg: impl Into<String>) {
        self.diags.push(Diagnostic::error(code, line, col, msg));
    }

    fn warn(&mut self, code: &str, line: u32, col: u32, msg: impl Into<String>) {
        self.diags.push(Diagnostic::warning(code, line, col, msg));
    }

    // ---- the block ----

    fn module(&mut self, body: &[Stmt]) -> Program {
        let tops = top_names(body);
        for (name, _) in &tops {
            self.globals.push(name.clone());
        }
        let defs: Vec<&FuncDef> = body
            .iter()
            .filter_map(|s| match &s.kind {
                StmtKind::Def(f) => Some(f.as_ref()),
                _ => None,
            })
            .collect();
        for (i, d) in defs.iter().enumerate() {
            if self.defs.insert(d.name.clone(), i as u32).is_some() {
                self.error("duplicateName", d.line, d.col, format!("function `{}` is defined twice in this block", d.name));
            }
        }
        for s in body {
            self.check_top_statement(s);
        }
        for (name, is_def) in &tops {
            let line = body.first().map_or(1, |s| s.line);
            if self.model.find(name).is_some() {
                self.warn("shadowedName", line, 1, format!("this block's `{name}` hides the object of the same name"));
            } else if !is_def && (builtins::global(name).is_some() || units::unit(name).is_some()) {
                self.warn("shadowedName", line, 1, format!("this block's `{name}` hides the built-in `{name}`"));
            }
        }
        // Functions first (their indices are known), then the block's own code as the last function.
        self.functions = defs.iter().map(|d| placeholder(&d.name)).collect();
        self.calls = vec![BTreeSet::new(); defs.len() + 1];
        self.direct_waits = vec![false; defs.len() + 1];
        for (i, d) in defs.iter().enumerate() {
            self.function(i as u32, d);
        }
        let init = defs.len() as u32;
        self.functions.push(placeholder("<block>"));
        self.f = FnState { module: true, line: 1, ..FnState::default() };
        for (name, &i) in &self.defs.clone() {
            if let Some(slot) = self.globals.iter().position(|g| g == name) {
                self.emit(Op::LoadFunc(i));
                self.emit(Op::StoreGlobal(slot as u32));
            }
        }
        for s in body {
            if !matches!(s.kind, StmtKind::Def(_)) {
                self.stmt(s);
            }
        }
        let none = self.konst(Value::None);
        self.emit(Op::Const(none));
        self.emit(Op::Return);
        let f = std::mem::take(&mut self.f);
        self.functions[init as usize] = Function {
            name: "<block>".into(),
            params: Vec::new(),
            defaults: Vec::new(),
            nlocals: f.locals.len() as u32,
            locals: f.locals,
            code: f.code,
            lines: f.lines,
            doc: None,
            line: 1,
        };
        let waits = self.waits();
        for h in self.handlers.clone() {
            let why = match h.event {
                EventSpec::ChooseExit(_) => "an exit decision answers at once",
                EventSpec::Finish => "the run is ending",
                _ => continue,
            };
            if waits.get(h.func as usize).copied().unwrap_or(false) {
                self.error("waitInDecision", h.line, 1, format!("this function waits (itself or in what it calls), but {why}"));
            }
        }
        Program {
            functions: std::mem::take(&mut self.functions),
            init,
            globals: self.globals.clone(),
            consts: std::mem::take(&mut self.consts),
            names: std::mem::take(&mut self.names),
            handlers: std::mem::take(&mut self.handlers),
            waits,
        }
    }

    /// Which functions may wait: those that wait, and those that call one that may.
    fn waits(&self) -> Vec<bool> {
        let mut w = self.direct_waits.clone();
        loop {
            let mut changed = false;
            for (i, calls) in self.calls.iter().enumerate() {
                if !w[i] && calls.iter().any(|&c| w.get(c as usize).copied().unwrap_or(false)) {
                    w[i] = true;
                    changed = true;
                }
            }
            if !changed {
                return w;
            }
        }
    }

    fn check_top_statement(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Assign { .. }
            | StmtKind::AugAssign { .. }
            | StmtKind::AnnAssign { .. }
            | StmtKind::Def(_)
            | StmtKind::Pass
            | StmtKind::Global(_) => {}
            StmtKind::Import { module } => {
                if !IMPORTABLE.contains(&module.as_str()) {
                    self.error(
                        "importNotAllowed",
                        s.line,
                        s.col,
                        format!(
                            "`{module}` can't be imported: code reaches only the model and the built-in modules (intrascript, math, random)"
                        ),
                    );
                }
            }
            StmtKind::Expr(Expr { kind: ExprKind::Str(_), .. }) => {}
            StmtKind::Return(_) => self.error("returnOutsideFunction", s.line, s.col, "`return` outside a function"),
            _ => self.error(
                "topLevelCode",
                s.line,
                s.col,
                "only variables, functions and imports go at the top of a block; put this in a function with @on_start",
            ),
        }
    }

    // ---- functions ----

    fn function(&mut self, index: u32, d: &FuncDef) {
        let events = self.decorators(d, index);
        let mut f = FnState { line: d.line, ..FnState::default() };
        f.global_decl = global_decls(&d.body);
        for p in &d.params {
            let slot = f.locals.len() as u32;
            f.locals.push(p.name.clone());
            f.local_map.insert(p.name.clone(), slot);
        }
        f.params = d.params.len();
        if events.iter().any(|e| matches!(e, EventSpec::ChooseExit(_))) {
            f.no_wait = Some("an exit decision answers at once");
        } else if events.contains(&EventSpec::Finish) {
            f.no_wait = Some("the run is ending");
        }
        let assigned = assigned_names(&d.body);
        for (name, line, col) in &assigned {
            if f.global_decl.contains(name) || f.local_map.contains_key(name) {
                continue;
            }
            if self.globals.contains(name) {
                self.warn(
                    "localShadowsGlobal",
                    *line,
                    *col,
                    format!("assigning makes `{name}` a new variable of this function; write `global {name}` first to change the block's"),
                );
            } else if self.model.find(name).is_some() {
                self.warn("shadowedName", *line, *col, format!("`{name}` here is a variable of this function, not the object"));
            }
            let slot = f.locals.len() as u32;
            f.locals.push(name.clone());
            f.local_map.insert(name.clone(), slot);
        }
        let mut defaults = Vec::new();
        for p in &d.params {
            if let Some(e) = &p.default {
                match const_eval(e) {
                    Some(v) => defaults.push(v),
                    None => self.error(
                        "defaultNotConstant",
                        e.line,
                        e.col,
                        "a default value is a fixed value (a number, a text, a quantity like 2 * s, True, None)",
                    ),
                }
            }
        }
        if !events.is_empty() {
            for (k, p) in d.params.iter().enumerate() {
                if !HANDLER_PARAMS.contains(&p.name.as_str()) && p.default.is_none() {
                    self.error(
                        "handlerParameters",
                        d.line,
                        d.col,
                        format!(
                            "an event handler takes `pallet` and `sender` only; `{}` (parameter {}) needs a default value",
                            p.name,
                            k + 1
                        ),
                    );
                }
            }
        }
        self.f = f;
        self.block(&d.body);
        let none = self.konst(Value::None);
        self.emit(Op::Const(none));
        self.emit(Op::Return);
        let f = std::mem::take(&mut self.f);
        for (slot, name) in f.locals.iter().enumerate().skip(f.params) {
            if !f.loaded.contains(&(slot as u32))
                && !name.starts_with('_')
                && !name.starts_with('$')
                && let Some((_, line, col)) = assigned.iter().find(|(n, ..)| n == name)
            {
                self.warn("unusedVariable", *line, *col, format!("`{name}` gets a value that is never read"));
            }
        }
        self.calls[index as usize] = f.calls.clone();
        self.direct_waits[index as usize] = f.waits;
        self.functions[index as usize] = Function {
            name: d.name.clone(),
            params: d.params.iter().map(|p| p.name.clone()).collect(),
            defaults,
            nlocals: f.locals.len() as u32,
            locals: f.locals,
            code: f.code,
            lines: f.lines,
            doc: d.doc.clone(),
            line: d.line,
        };
    }

    /// The events a function's decorators bind it to.
    fn decorators(&mut self, d: &FuncDef, index: u32) -> Vec<EventSpec> {
        let mut out = Vec::new();
        for dec in &d.decorators {
            let (name, args, kwargs) = match &dec.kind {
                ExprKind::Name(n) => (n.clone(), Vec::new(), Vec::new()),
                ExprKind::Call { func, args, kwargs } => match &func.kind {
                    ExprKind::Name(n) => (n.clone(), args.clone(), kwargs.clone()),
                    _ => {
                        self.error("unknownEvent", dec.line, dec.col, "a decorator is an event: @on_enter(Roller3), @every(1 * minute)…");
                        continue;
                    }
                },
                _ => {
                    self.error("unknownEvent", dec.line, dec.col, "a decorator is an event: @on_enter(Roller3), @every(1 * minute)…");
                    continue;
                }
            };
            let mut priority = 0;
            let mut start = None;
            for (k, v) in &kwargs {
                match (k.as_str(), const_eval(v)) {
                    ("priority", Some(Value::Int(p))) => priority = p,
                    ("start", Some(t)) if name == "every" => match to_us(&t) {
                        Some(us) if us >= 0 => start = Some(us),
                        _ => self.error("wrongArguments", v.line, v.col, "start= is a time: start=0 * s"),
                    },
                    _ => self.error(
                        "wrongArguments",
                        v.line,
                        v.col,
                        format!("@{name} takes no `{k}=` like this (priority=5 is a whole number)"),
                    ),
                }
            }
            let (line, col) = (dec.line, dec.col);
            let mut events = Vec::new();
            match name.as_str() {
                "on_start" | "on_finish" => {
                    if !args.is_empty() {
                        self.error("wrongArguments", line, col, format!("@{name} takes no objects"));
                    }
                    events.push(if name == "on_start" { EventSpec::Start } else { EventSpec::Finish });
                }
                "on_create" | "on_enter" | "on_leave" | "choose_exit" => {
                    if args.is_empty() {
                        self.error("wrongArguments", line, col, format!("@{name} names at least one object: @{name}(Roller3)"));
                    }
                    for a in &args {
                        let Some(o) = self.decorator_object(a) else { continue };
                        let info = self.model.object(o).cloned().unwrap_or_default();
                        let (fits, what) = match name.as_str() {
                            "on_create" => (info.creates_pallets, "doesn't make pallets (only sources do)"),
                            "choose_exit" => (info.decides_exit, "has no exits to choose between (turntables do)"),
                            _ => (info.carries_pallets, "carries no pallets"),
                        };
                        if !fits {
                            self.error("unknownEvent", a.line, a.col, format!("@{name}: `{}` {what}", info.name));
                            continue;
                        }
                        events.push(match name.as_str() {
                            "on_create" => EventSpec::Create(o),
                            "on_enter" => EventSpec::Enter(o),
                            "on_leave" => EventSpec::Leave(o),
                            _ => EventSpec::ChooseExit(o),
                        });
                    }
                }
                "on_rise" | "on_fall" | "on_change" => {
                    if args.is_empty() {
                        self.error("wrongArguments", line, col, format!("@{name} names a signal: @{name}(Photoeye3.detected)"));
                    }
                    for a in &args {
                        let ExprKind::Attr(base, signal) = &a.kind else {
                            self.error("wrongArguments", a.line, a.col, format!("@{name} takes a signal: Object.signal"));
                            continue;
                        };
                        let Some(o) = self.decorator_object(base) else { continue };
                        let info = self.model.object(o).cloned().unwrap_or_default();
                        match info.member(signal).map(|m| m.kind) {
                            Some(MemberKind::Signal { .. }) => {}
                            _ => {
                                let signals: Vec<&str> = info
                                    .members
                                    .iter()
                                    .filter(|m| matches!(m.kind, MemberKind::Signal { .. }))
                                    .map(|m| m.name.as_str())
                                    .collect();
                                let list = if signals.is_empty() { "none".to_string() } else { signals.join(", ") };
                                self.error(
                                    "unknownEvent",
                                    a.line,
                                    a.col,
                                    format!("`{}` has no signal `{signal}` (its signals: {list})", info.name),
                                );
                                continue;
                            }
                        }
                        let edge = match name.as_str() {
                            "on_rise" => Edge::Rise,
                            "on_fall" => Edge::Fall,
                            _ => Edge::Change,
                        };
                        events.push(EventSpec::Signal { object: o, signal: signal.clone(), edge });
                    }
                }
                "every" | "at" => {
                    let t = args.first().and_then(const_eval).and_then(|v| to_us(&v));
                    match (t, args.len()) {
                        (Some(us), 1) if us > 0 || (name == "at" && us == 0) => events.push(if name == "every" {
                            EventSpec::Every { interval: us, first: start.unwrap_or(us) }
                        } else {
                            EventSpec::At(us)
                        }),
                        _ => self.error("wrongArguments", line, col, format!("@{name} takes one fixed time: @{name}(10 * minute)")),
                    }
                }
                other => {
                    let known: Vec<&str> = DECORATORS.iter().map(|(n, _)| *n).collect();
                    let hint = suggest(other, known.iter().copied()).map(|s| format!(" (did you mean @{s}?)")).unwrap_or_default();
                    self.error("unknownEvent", line, col, format!("@{other} is not an event{hint}"));
                }
            }
            for e in events {
                self.handlers.push(Handler { event: e.clone(), func: index, priority, line: d.line });
                out.push(e);
            }
        }
        out
    }

    /// The object a decorator names: by name, or by name or id in a text.
    fn decorator_object(&mut self, e: &Expr) -> Option<u32> {
        let found = match &e.kind {
            ExprKind::Name(n) => self.model.find(n),
            ExprKind::Str(s) => self.model.find_name_or_id(s),
            // A whole address: `Seccion1.Roller3` (ADR-0029).
            ExprKind::Attr(..) if e.dotted().is_some() => self.whole_address(e),
            _ => {
                self.error("wrongArguments", e.line, e.col, "name the object: `Roller3`, or its address `Seccion1.Roller3`");
                return None;
            }
        };
        if found.is_none() {
            let n = match &e.kind {
                ExprKind::Name(n) | ExprKind::Str(n) => n.clone(),
                _ => e.dotted().map(|p| p.join(".")).unwrap_or_default(),
            };
            let pool = self.model.objects.iter().flat_map(|o| [o.name.as_str(), o.address.as_str()]).filter(|s| !s.is_empty());
            let hint = suggest(&n, pool).map(|s| format!(" (did you mean `{s}`?)")).unwrap_or_default();
            self.error("unknownName", e.line, e.col, format!("the layout has no object `{n}`{hint}"));
        }
        found
    }

    // ---- emitting ----

    fn emit(&mut self, op: Op) -> usize {
        self.f.code.push(op);
        self.f.lines.push(self.f.line);
        self.f.code.len() - 1
    }

    fn here(&self) -> u32 {
        self.f.code.len() as u32
    }

    fn patch(&mut self, at: usize, target: u32) {
        if let Some(op) = self.f.code.get_mut(at) {
            *op = match *op {
                Op::Jump(_) => Op::Jump(target),
                Op::JumpIfFalse(_) => Op::JumpIfFalse(target),
                Op::JumpIfTrue(_) => Op::JumpIfTrue(target),
                Op::JumpIfFalseKeep(_) => Op::JumpIfFalseKeep(target),
                Op::JumpIfTrueKeep(_) => Op::JumpIfTrueKeep(target),
                Op::ForIter(_) => Op::ForIter(target),
                Op::WaitUntilPause(slot, _) => Op::WaitUntilPause(slot, target),
                other => other,
            };
        }
    }

    fn konst(&mut self, v: Value) -> u32 {
        self.consts.push(v);
        (self.consts.len() - 1) as u32
    }

    fn name(&mut self, n: &str) -> u32 {
        if let Some(i) = self.names.iter().position(|m| m.as_ref() == n) {
            return i as u32;
        }
        self.names.push(Arc::from(n));
        (self.names.len() - 1) as u32
    }

    fn hidden_local(&mut self, what: &str) -> u32 {
        let slot = self.f.locals.len() as u32;
        self.f.locals.push(format!("${what}{slot}"));
        slot
    }

    // ---- statements ----

    fn block(&mut self, body: &[Stmt]) {
        let (mut ended, mut warned) = (false, false);
        for s in body {
            if ended && !warned {
                self.warn("unreachableCode", s.line, s.col, "this never runs: the lines before it leave the block");
                warned = true;
            }
            self.stmt(s);
            ended |= matches!(s.kind, StmtKind::Return(_) | StmtKind::Break | StmtKind::Continue);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        self.f.line = s.line;
        match &s.kind {
            StmtKind::Expr(e) => {
                self.expr(e);
                self.emit(Op::Pop);
            }
            StmtKind::Assign { targets, value } => {
                self.expr(value);
                for (i, t) in targets.iter().enumerate() {
                    if i + 1 < targets.len() {
                        self.emit(Op::Dup);
                    }
                    self.store(t);
                }
            }
            StmtKind::AnnAssign { target, value } => {
                if let Some(v) = value {
                    self.expr(v);
                    self.store(target);
                }
            }
            StmtKind::AugAssign { target, op, value } => self.aug_assign(target, *op, value),
            StmtKind::If { cond, body, orelse } => {
                self.expr(cond);
                let to_else = self.emit(Op::JumpIfFalse(PENDING));
                self.block(body);
                if orelse.is_empty() {
                    let end = self.here();
                    self.patch(to_else, end);
                } else {
                    let to_end = self.emit(Op::Jump(PENDING));
                    let else_at = self.here();
                    self.patch(to_else, else_at);
                    self.block(orelse);
                    let end = self.here();
                    self.patch(to_end, end);
                }
            }
            StmtKind::While { cond, body } => {
                let start = self.f.code.len();
                self.expr(cond);
                let exit = self.emit(Op::JumpIfFalse(PENDING));
                self.f.loops.push(Loop { start, breaks: vec![exit], is_for: false });
                self.block(body);
                self.emit(Op::Jump(start as u32));
                self.end_loop();
            }
            StmtKind::For { target, iter, body } => {
                self.expr(iter);
                self.emit(Op::GetIter);
                let start = self.f.code.len();
                let exit = self.emit(Op::ForIter(PENDING));
                self.store(target);
                self.f.loops.push(Loop { start, breaks: vec![exit], is_for: true });
                self.block(body);
                self.emit(Op::Jump(start as u32));
                self.end_loop();
            }
            StmtKind::Break => match self.f.loops.last().map(|l| l.is_for) {
                None => self.error("breakOutsideLoop", s.line, s.col, "`break` outside a loop"),
                Some(is_for) => {
                    if is_for {
                        self.emit(Op::Pop);
                    }
                    let j = self.emit(Op::Jump(PENDING));
                    if let Some(l) = self.f.loops.last_mut() {
                        l.breaks.push(j);
                    }
                }
            },
            StmtKind::Continue => match self.f.loops.last().map(|l| l.start) {
                None => self.error("breakOutsideLoop", s.line, s.col, "`continue` outside a loop"),
                Some(start) => {
                    self.emit(Op::Jump(start as u32));
                }
            },
            StmtKind::Pass | StmtKind::Global(_) | StmtKind::Import { .. } => {}
            StmtKind::Return(v) => {
                if self.f.module {
                    return;
                }
                match v {
                    Some(e) => self.expr(e),
                    None => {
                        let none = self.konst(Value::None);
                        self.emit(Op::Const(none));
                    }
                }
                self.emit(Op::Return);
            }
            StmtKind::Def(d) => {
                self.error("nestedFunction", d.line, d.col, "functions are defined at the top of a block, not inside another")
            }
            StmtKind::Assert { cond, msg } => {
                self.expr(cond);
                let ok = self.emit(Op::JumpIfTrue(PENDING));
                match msg {
                    Some(m) => self.expr(m),
                    None => {
                        let none = self.konst(Value::None);
                        self.emit(Op::Const(none));
                    }
                }
                self.emit(Op::Assert);
                let end = self.here();
                self.patch(ok, end);
            }
        }
    }

    fn end_loop(&mut self) {
        let end = self.here();
        if let Some(l) = self.f.loops.pop() {
            for b in l.breaks {
                self.patch(b, end);
            }
        }
    }

    fn aug_assign(&mut self, target: &Expr, op: BinOp, value: &Expr) {
        match &target.kind {
            ExprKind::Name(_) => {
                self.expr(target);
                self.expr(value);
                self.emit(Op::Binary(op));
                self.store(target);
            }
            ExprKind::Attr(base, name) => {
                self.check_member(base, name, true);
                self.expr(base);
                self.emit(Op::Dup);
                let n = self.name(name);
                self.emit(Op::GetAttr(n));
                self.expr(value);
                self.emit(Op::Binary(op));
                self.emit(Op::Swap);
                self.emit(Op::SetAttr(n));
            }
            ExprKind::Index(base, index) => {
                self.expr(base);
                self.expr(index);
                self.emit(Op::Dup2);
                self.emit(Op::GetItem);
                self.expr(value);
                self.emit(Op::Binary(op));
                self.emit(Op::Rot3);
                self.emit(Op::SetItem);
            }
            _ => self.error("syntax", target.line, target.col, "can't change this with an augmented assignment"),
        }
    }

    /// Stores the value on top of the stack into a target.
    fn store(&mut self, t: &Expr) {
        match &t.kind {
            ExprKind::Name(n) => {
                if let Some((_, slot)) = self.f.comp.iter().rev().find(|(m, _)| m == n) {
                    let slot = *slot;
                    self.emit(Op::StoreLocal(slot));
                    return;
                }
                if !self.f.module
                    && !self.f.global_decl.contains(n)
                    && let Some(&slot) = self.f.local_map.get(n)
                {
                    self.emit(Op::StoreLocal(slot));
                    return;
                }
                match self.globals.iter().position(|g| g == n) {
                    Some(slot) => {
                        self.emit(Op::StoreGlobal(slot as u32));
                    }
                    None => self.error("unknownName", t.line, t.col, format!("can't assign `{n}` here")),
                }
            }
            ExprKind::Attr(base, name) => {
                self.check_member(base, name, true);
                self.expr(base);
                let n = self.name(name);
                self.emit(Op::SetAttr(n));
            }
            ExprKind::Index(base, index) => {
                self.expr(base);
                self.expr(index);
                self.emit(Op::SetItem);
            }
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                self.emit(Op::Unpack(items.len() as u32));
                for i in items {
                    self.store(i);
                }
            }
            _ => self.error("syntax", t.line, t.col, "can't assign to this"),
        }
    }

    // ---- expressions ----

    fn resolve(&self, name: &str) -> Resolved {
        if let Some((_, slot)) = self.f.comp.iter().rev().find(|(m, _)| m == name) {
            return Resolved::Local(*slot);
        }
        if !self.f.module
            && !self.f.global_decl.contains(name)
            && let Some(&slot) = self.f.local_map.get(name)
        {
            return Resolved::Local(slot);
        }
        if let Some(i) = self.globals.iter().position(|g| g == name) {
            return Resolved::Global(i as u32);
        }
        if let Some(o) = self.model.find(name) {
            return Resolved::Object(o);
        }
        if let Some(b) = builtins::global(name) {
            return Resolved::Builtin(b);
        }
        if let Some((x, d)) = units::unit(name) {
            return Resolved::Unit(x, d);
        }
        Resolved::Unknown
    }

    fn load_name(&mut self, n: &str, line: u32, col: u32) {
        match self.resolve(n) {
            Resolved::Local(slot) => {
                self.f.loaded.insert(slot);
                self.emit(Op::LoadLocal(slot));
            }
            Resolved::Global(slot) => {
                self.emit(Op::LoadGlobal(slot));
            }
            Resolved::Object(o) => {
                self.emit(Op::LoadObject(o));
            }
            Resolved::Builtin(b) => {
                self.emit(Op::LoadBuiltin(b));
            }
            Resolved::Unit(x, d) => {
                let k = self.konst(Value::qty(x, d));
                self.emit(Op::Const(k));
            }
            Resolved::Unknown => {
                let msg = match n {
                    "wait" | "wait_until" => format!("`{n}` is called, not used as a value: {n}(…)"),
                    "true" | "false" | "none" | "null" => format!("Python writes `{}`", capitalised(n)),
                    _ if DECORATORS.iter().any(|(d, _)| *d == n) => {
                        format!("`{n}` is an event: write it as a decorator above a function, @{n}(…)")
                    }
                    _ => {
                        let mut pool: Vec<String> = self.f.locals.iter().filter(|l| !l.starts_with('$')).cloned().collect();
                        pool.extend(self.globals.iter().cloned());
                        pool.extend(self.model.objects.iter().map(|o| o.name.clone()));
                        pool.extend(builtins::GLOBALS.iter().map(|(g, ..)| g.to_string()));
                        pool.extend(units::UNITS.iter().map(|(u, ..)| u.to_string()));
                        let hint =
                            suggest(n, pool.iter().map(String::as_str)).map(|s| format!(" (did you mean `{s}`?)")).unwrap_or_default();
                        format!("`{n}` is not defined: not a variable, function, object of the layout or built-in name{hint}")
                    }
                };
                self.error("unknownName", line, col, msg);
            }
        }
    }

    /// Checks `base.name` when the base is known before the run: an object of the layout, or a built-in module.
    /// The checks of one object's member, as `check_member` runs them: `label` is how the program names the object.
    fn check_object(&mut self, o: u32, b: &str, base: &Expr, name: &str, writing: bool) {
        let Some(info) = self.model.object(o).cloned() else { return };
        if info.is_code_block() {
            if let Some(Some(exports)) = self.exports.get(&o)
                && !exports.iter().any(|e| e == name)
            {
                let hint = suggest(name, exports.iter().map(String::as_str)).map(|s| format!(" (did you mean `{s}`?)")).unwrap_or_default();
                self.error("unknownMember", base.line, base.col, format!("code block `{b}` has no variable or function `{name}`{hint}"));
            }
            return;
        }
        match info.member(name) {
            None => {
                let hint = suggest(name, info.members.iter().map(|m| m.name.as_str()))
                    .map(|s| format!(" (did you mean `{s}`?)"))
                    .unwrap_or_default();
                self.error("unknownMember", base.line, base.col, format!("`{b}` ({}) has no member `{name}`{hint}", info.kind));
            }
            Some(m) if writing && m.kind != (MemberKind::Signal { writable: true }) => {
                let why = match m.kind {
                    MemberKind::Signal { .. } => "an output signal: the object sets it",
                    MemberKind::Value => "read-only during a run: design data are changed in the Design module",
                    MemberKind::Method => "a method: call it",
                };
                self.error("notWritable", base.line, base.col, format!("`{b}.{name}` is {why}"));
            }
            Some(_) => {}
        }
    }

    /// The object a whole address names (`Seccion1.Conveyor1`), when the expression is only names and dots and the
    /// layout has an object at that address (ADR-0029).
    fn whole_address(&self, e: &Expr) -> Option<u32> {
        let path = e.dotted()?;
        if path.len() < 2 {
            return None;
        }
        let address = path.join(".");
        self.model.objects.iter().position(|o| o.address == address).and_then(|i| u32::try_from(i).ok())
    }

    fn check_member(&mut self, base: &Expr, name: &str, writing: bool) {
        if let Some(o) = self.whole_address(base) {
            let label = self.model.object(o).map(|i| i.address.clone()).unwrap_or_default();
            self.check_object(o, &label, base, name, writing);
            return;
        }
        let ExprKind::Name(b) = &base.kind else { return };
        match self.resolve(b) {
            Resolved::Object(o) => self.check_object(o, b, base, name, writing),
            Resolved::Builtin(m @ (Builtin::Math | Builtin::Random | Builtin::Simulation)) => {
                let names = builtins::module_names(m);
                if !names.contains(&name) {
                    let hint = suggest(name, names.iter().copied()).map(|s| format!(" (did you mean `{s}`?)")).unwrap_or_default();
                    self.error("unknownMember", base.line, base.col, format!("`{b}` has no `{name}`{hint}"));
                } else if writing {
                    self.error("notWritable", base.line, base.col, format!("`{b}.{name}` can't be changed"));
                }
            }
            Resolved::Local(slot)
                if writing
                    && self.f.locals.get(slot as usize).is_some_and(|l| l == "pallet")
                    && PALLET_MEMBERS.iter().any(|(m, _)| *m == name) =>
            {
                self.error(
                    "notWritable",
                    base.line,
                    base.col,
                    format!("`pallet.{name}` is the pallet's own data; give it an attribute of your own instead"),
                );
            }
            _ => {}
        }
    }

    fn expr(&mut self, e: &Expr) {
        let saved = self.f.line;
        self.f.line = e.line;
        if let Some(o) = self.whole_address(e) {
            self.emit(Op::LoadObject(o));
            self.f.line = saved;
            return;
        }
        match &e.kind {
            ExprKind::Int(i) => {
                let k = self.konst(Value::Int(*i));
                self.emit(Op::Const(k));
            }
            ExprKind::Float(x) => {
                let k = self.konst(Value::Float(*x));
                self.emit(Op::Const(k));
            }
            ExprKind::Str(s) => {
                let k = self.konst(Value::str(s));
                self.emit(Op::Const(k));
            }
            ExprKind::Bool(b) => {
                let k = self.konst(Value::Bool(*b));
                self.emit(Op::Const(k));
            }
            ExprKind::None => {
                let k = self.konst(Value::None);
                self.emit(Op::Const(k));
            }
            ExprKind::FStr(parts) => {
                for p in parts {
                    match p {
                        FPart::Lit(s) => {
                            let k = self.konst(Value::str(s));
                            self.emit(Op::Const(k));
                        }
                        FPart::Expr(x, spec) => {
                            self.expr(x);
                            let k = match spec {
                                Some(s) => self.konst(Value::str(s)),
                                None => NONE,
                            };
                            self.emit(Op::Format(k));
                        }
                    }
                }
                self.emit(Op::BuildString(parts.len() as u32));
            }
            ExprKind::Name(n) => self.load_name(n, e.line, e.col),
            ExprKind::Attr(base, name) => {
                self.check_member(base, name, false);
                self.expr(base);
                let n = self.name(name);
                self.emit(Op::GetAttr(n));
            }
            ExprKind::Index(base, index) => {
                self.expr(base);
                if let ExprKind::Slice(a, b, c) = &index.kind {
                    for part in [a, b, c] {
                        match part {
                            Some(x) => self.expr(x),
                            None => {
                                let k = self.konst(Value::None);
                                self.emit(Op::Const(k));
                            }
                        }
                    }
                    self.emit(Op::BuildSlice);
                } else {
                    self.expr(index);
                }
                self.emit(Op::GetItem);
            }
            ExprKind::Slice(..) => self.error("syntax", e.line, e.col, "a slice goes inside brackets: list[1:3]"),
            ExprKind::Call { func, args, kwargs } => self.call(e, func, args, kwargs),
            ExprKind::Unary(op, x) => {
                self.expr(x);
                self.emit(Op::Unary(*op));
            }
            ExprKind::Binary(op, a, b) => {
                self.expr(a);
                self.expr(b);
                self.emit(Op::Binary(*op));
            }
            ExprKind::Compare(first, ops) => {
                self.expr(first);
                if ops.len() == 1 {
                    self.expr(&ops[0].1);
                    self.emit(Op::Compare(ops[0].0));
                } else {
                    let mut cleanups = Vec::new();
                    for (i, (op, x)) in ops.iter().enumerate() {
                        self.expr(x);
                        if i + 1 < ops.len() {
                            self.emit(Op::Dup);
                            self.emit(Op::Rot3);
                            self.emit(Op::Compare(*op));
                            cleanups.push(self.emit(Op::JumpIfFalseKeep(PENDING)));
                        } else {
                            self.emit(Op::Compare(*op));
                        }
                    }
                    let to_end = self.emit(Op::Jump(PENDING));
                    let cleanup = self.here();
                    for c in cleanups {
                        self.patch(c, cleanup);
                    }
                    self.emit(Op::Swap);
                    self.emit(Op::Pop);
                    let end = self.here();
                    self.patch(to_end, end);
                }
            }
            ExprKind::And(a, b) | ExprKind::Or(a, b) => {
                self.expr(a);
                let j = if matches!(e.kind, ExprKind::And(..)) {
                    self.emit(Op::JumpIfFalseKeep(PENDING))
                } else {
                    self.emit(Op::JumpIfTrueKeep(PENDING))
                };
                self.expr(b);
                let end = self.here();
                self.patch(j, end);
            }
            ExprKind::IfExp { cond, then, orelse } => {
                self.expr(cond);
                let to_else = self.emit(Op::JumpIfFalse(PENDING));
                self.expr(then);
                let to_end = self.emit(Op::Jump(PENDING));
                let at = self.here();
                self.patch(to_else, at);
                self.expr(orelse);
                let end = self.here();
                self.patch(to_end, end);
            }
            ExprKind::List(items) => {
                for i in items {
                    self.expr(i);
                }
                self.emit(Op::BuildList(items.len() as u32));
            }
            ExprKind::Tuple(items) => {
                for i in items {
                    self.expr(i);
                }
                self.emit(Op::BuildTuple(items.len() as u32));
            }
            ExprKind::Dict(pairs) => {
                for (k, v) in pairs {
                    self.expr(k);
                    self.expr(v);
                }
                self.emit(Op::BuildDict(pairs.len() as u32));
            }
            ExprKind::ListComp { elt, target, iter, cond } => {
                self.emit(Op::BuildList(0));
                self.expr(iter);
                self.emit(Op::GetIter);
                let start = self.f.code.len();
                let exit = self.emit(Op::ForIter(PENDING));
                let names = target_names(target);
                let depth = self.f.comp.len();
                for n in &names {
                    let slot = self.hidden_local("comp");
                    self.f.loaded.insert(slot);
                    self.f.comp.push((n.clone(), slot));
                }
                self.store(target);
                if let Some(c) = cond {
                    self.expr(c);
                    self.emit(Op::JumpIfFalse(start as u32));
                }
                self.expr(elt);
                self.emit(Op::ListAppend(2));
                self.emit(Op::Jump(start as u32));
                let end = self.here();
                self.patch(exit, end);
                self.f.comp.truncate(depth);
            }
        }
        self.f.line = saved;
    }

    fn call(&mut self, e: &Expr, func: &Expr, args: &[Expr], kwargs: &[(String, Expr)]) {
        if let ExprKind::Name(n) = &func.kind
            && matches!(self.resolve(n), Resolved::Unknown)
        {
            match n.as_str() {
                "wait" => return self.wait(e, args, kwargs),
                "wait_until" => return self.wait_until(e, args, kwargs),
                _ => {}
            }
        }
        if let ExprKind::Name(n) = &func.kind
            && let Resolved::Global(_) = self.resolve(n)
            && let Some(&i) = self.defs.get(n)
        {
            self.f.calls.insert(i);
        }
        self.expr(func);
        for a in args {
            self.expr(a);
        }
        let kw = if kwargs.is_empty() {
            NONE
        } else {
            for (_, v) in kwargs {
                self.expr(v);
            }
            self.konst(Value::tuple(kwargs.iter().map(|(k, _)| Value::str(k)).collect()))
        };
        self.f.line = e.line;
        self.emit(Op::Call(args.len() as u32, kw));
    }

    fn may_wait_here(&mut self, e: &Expr, what: &str) -> bool {
        if self.f.module {
            self.error("waitOutsideHandler", e.line, e.col, format!("`{what}` pauses a handler; here nothing runs as one"));
            return false;
        }
        if let Some(why) = self.f.no_wait {
            self.error("waitInDecision", e.line, e.col, format!("`{what}` can't be used here: {why}"));
            return false;
        }
        self.f.waits = true;
        true
    }

    fn wait(&mut self, e: &Expr, args: &[Expr], kwargs: &[(String, Expr)]) {
        if args.len() != 1 || !kwargs.is_empty() {
            self.error("wrongArguments", e.line, e.col, "wait() takes one time: wait(2 * s)");
            return;
        }
        self.may_wait_here(e, "wait()");
        self.expr(&args[0]);
        self.f.line = e.line;
        self.emit(Op::Wait);
    }

    fn wait_until(&mut self, e: &Expr, args: &[Expr], kwargs: &[(String, Expr)]) {
        if args.len() != 1 || kwargs.iter().any(|(k, _)| k != "timeout") {
            self.error(
                "wrongArguments",
                e.line,
                e.col,
                "wait_until() takes a condition and, if you like, timeout=: wait_until(Roller4.occupied, timeout=30 * s)",
            );
            return;
        }
        self.may_wait_here(e, "wait_until()");
        match kwargs.first() {
            Some((_, t)) => self.expr(t),
            None => {
                let k = self.konst(Value::None);
                self.emit(Op::Const(k));
            }
        }
        let slot = self.hidden_local("deadline");
        self.f.line = e.line;
        self.emit(Op::WaitUntilStart(slot));
        let check = self.here();
        self.expr(&args[0]);
        let to_true = self.emit(Op::JumpIfTrue(PENDING));
        let pause = self.emit(Op::WaitUntilPause(slot, PENDING));
        self.emit(Op::Jump(check));
        let t = self.here();
        self.patch(to_true, t);
        let yes = self.konst(Value::Bool(true));
        self.emit(Op::Const(yes));
        let to_end = self.emit(Op::Jump(PENDING));
        let f = self.here();
        self.patch(pause, f);
        let no = self.konst(Value::Bool(false));
        self.emit(Op::Const(no));
        let end = self.here();
        self.patch(to_end, end);
    }
}

fn placeholder(name: &str) -> Function {
    Function {
        name: name.into(),
        params: Vec::new(),
        locals: Vec::new(),
        defaults: Vec::new(),
        nlocals: 0,
        code: Vec::new(),
        lines: Vec::new(),
        doc: None,
        line: 0,
    }
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// A fixed value known before the run: literals, unit names and arithmetic on them (`10 * minute`).
pub fn const_eval(e: &Expr) -> Option<Value> {
    match &e.kind {
        ExprKind::Int(i) => Some(Value::Int(*i)),
        ExprKind::Float(x) => Some(Value::Float(*x)),
        ExprKind::Str(s) => Some(Value::str(s)),
        ExprKind::Bool(b) => Some(Value::Bool(*b)),
        ExprKind::None => Some(Value::None),
        ExprKind::Name(n) => units::unit(n).map(|(x, d)| Value::qty(x, d)),
        ExprKind::Unary(UnOp::Neg, x) => crate::ops::unary(UnOp::Neg, &const_eval(x)?).ok(),
        ExprKind::Unary(UnOp::Pos, x) => const_eval(x),
        ExprKind::Binary(op, a, b) => binary(*op, &const_eval(a)?, &const_eval(b)?).ok(),
        ExprKind::Tuple(items) => Some(Value::tuple(items.iter().map(const_eval).collect::<Option<_>>()?)),
        _ => None,
    }
}

/// A time (a quantity, or a plain number of seconds) in whole µs.
pub fn to_us(v: &Value) -> Option<i64> {
    let s = match v {
        Value::Qty(x, d) if *d == Dim::TIME => *x,
        Value::Int(i) => *i as f64,
        Value::Float(x) => *x,
        _ => return None,
    };
    if !s.is_finite() || s.abs() > 9.0e12 {
        return None;
    }
    Some((s * 1e6).round() as i64)
}

/// The closest name to a misspelt one, if one is close enough.
pub fn suggest<'n>(name: &str, pool: impl Iterator<Item = &'n str>) -> Option<&'n str> {
    let limit = (name.chars().count() / 3).clamp(1, 3);
    pool.filter(|c| *c != name)
        .map(|c| (distance(&name.to_lowercase(), &c.to_lowercase()), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

/// Edits between two names, a swap of neighbours counting as one (`rnu` is one edit from `run`).
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{Member, ObjectInfo};

    pub fn model() -> ModelInfo {
        let signal = |n: &str, w: bool| Member { name: n.into(), kind: MemberKind::Signal { writable: w }, help: String::new() };
        let value = |n: &str| Member { name: n.into(), kind: MemberKind::Value, help: String::new() };
        ModelInfo {
            objects: vec![
                ObjectInfo {
                    name: "Source1".into(),
                    id: "source1".into(),
                    kind: "flow.source".into(),
                    members: vec![signal("enable", true)],
                    creates_pallets: true,
                    ..Default::default()
                },
                ObjectInfo {
                    name: "Roller3".into(),
                    id: "roller3".into(),
                    kind: "conveyor.roller".into(),
                    members: vec![signal("run", true), signal("occupied", false), value("length"), value("count")],
                    carries_pallets: true,
                    ..Default::default()
                },
                ObjectInfo {
                    name: "Photoeye3".into(),
                    id: "pe3".into(),
                    kind: "sensor.photoeye".into(),
                    members: vec![signal("detected", false)],
                    ..Default::default()
                },
            ],
        }
    }

    fn diags(src: &str) -> Vec<Diagnostic> {
        check(&model(), 99, src, &[])
    }

    fn codes(src: &str) -> Vec<String> {
        diags(src).into_iter().map(|d| d.code).collect()
    }

    #[test]
    fn a_correct_block_compiles_with_its_handlers() {
        let src = "routed = 0\n\n@on_rise(Photoeye3.detected)\ndef hold(pallet):\n    global routed\n    routed += 1\n    Roller3.run = False\n    wait(2 * s)\n    Roller3.run = True\n\n@every(10 * minute, start=0 * s)\ndef report():\n    print(f\"{routed} pallets\")\n";
        let c = compile_blocks(&model(), &[Source { object: 99, code: src }]).remove(0);
        assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
        let p = c.program.unwrap();
        assert_eq!(p.handlers.len(), 2);
        assert_eq!(p.handlers[0].event, EventSpec::Signal { object: 2, signal: "detected".into(), edge: Edge::Rise });
        assert_eq!(p.handlers[1].event, EventSpec::Every { interval: 600_000_000, first: 0 });
        assert!(p.waits[0] && !p.waits[1]);
    }

    #[test]
    fn misspelt_names_and_members_are_errors_with_a_suggestion() {
        let d = diags("@on_start\ndef go():\n    Roler3.run = False\n");
        assert_eq!(d[0].code, "unknownName");
        assert!(d[0].message.contains("Roller3"), "{}", d[0].message);
        let d = diags("@on_start\ndef go():\n    Roller3.rnu = False\n");
        assert_eq!(d[0].code, "unknownMember");
        assert!(d[0].message.contains("run"));
        assert_eq!(codes("@on_start\ndef go():\n    Roller3.length = 4 * m\n"), ["notWritable"]);
        assert_eq!(codes("@on_start\ndef go():\n    Photoeye3.detected = True\n"), ["notWritable"]);
        assert_eq!(codes("@on_start\ndef go():\n    x = math.sqroot(2)\n    print(x)\n"), ["unknownMember"]);
    }

    #[test]
    fn events_must_fit_their_objects() {
        assert_eq!(codes("@on_enter(Photoeye3)\ndef f(pallet):\n    pass\n"), ["unknownEvent"]);
        assert_eq!(codes("@on_create(Roller3)\ndef f(pallet):\n    pass\n"), ["unknownEvent"]);
        assert_eq!(codes("@on_rise(Roller3.detected)\ndef f():\n    pass\n"), ["unknownEvent"]);
        assert_eq!(codes("@on_enterr(Roller3)\ndef f():\n    pass\n"), ["unknownEvent"]);
        assert_eq!(codes("@choose_exit(Roller3)\ndef f(pallet):\n    return None\n"), ["unknownEvent"]);
        assert_eq!(codes("@on_enter(Roller3)\ndef f(box):\n    pass\n"), ["handlerParameters"]);
        assert!(diags("@on_enter(Roller3)\ndef f(pallet):\n    pass\n").is_empty());
    }

    #[test]
    fn waiting_where_nothing_may_wait_is_an_error() {
        let mut m = model();
        m.objects[1].decides_exit = true;
        let d =
            check(&m, 99, "def slow():\n    wait(1 * s)\n\n@choose_exit(Roller3)\ndef route(pallet):\n    slow()\n    return None\n", &[]);
        assert_eq!(d.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), ["waitInDecision"]);
        assert_eq!(codes("@on_finish\ndef end():\n    wait(1 * s)\n"), ["waitInDecision"]);
        assert_eq!(codes("x = wait(1 * s)\n"), ["waitOutsideHandler"]);
    }

    #[test]
    fn pythons_scoping_trap_gets_a_warning_and_top_level_code_an_error() {
        assert_eq!(codes("n = 0\n@on_start\ndef go():\n    n = n + 1\n"), ["localShadowsGlobal"]);
        assert_eq!(codes("print('hi')\n"), ["topLevelCode"]);
        assert_eq!(codes("import os\n"), ["importNotAllowed"]);
        assert!(codes("from intrascript import *\nimport math\n").is_empty());
        assert_eq!(codes("@on_start\ndef go():\n    return 1\n    print(2)\n"), ["unreachableCode"]);
        assert_eq!(codes("@on_start\ndef go():\n    x = 3\n"), ["unusedVariable"]);
        assert_eq!(codes("@on_start\ndef go():\n    break\n"), ["breakOutsideLoop"]);
    }

    #[test]
    fn blocks_see_each_others_functions_and_variables() {
        let mut m = model();
        m.objects.push(ObjectInfo { name: "Routing".into(), id: "code1".into(), kind: crate::CODE_BLOCK.into(), ..Default::default() });
        m.objects.push(ObjectInfo { name: "Main".into(), id: "code2".into(), kind: crate::CODE_BLOCK.into(), ..Default::default() });
        let routing = Source { object: 3, code: "count = 0\ndef next_exit():\n    return None\n" };
        let ok = check(&m, 4, "@on_start\ndef go():\n    Routing.next_exit()\n    Routing.count = 2\n", &[routing]);
        assert!(ok.is_empty(), "{ok:?}");
        let bad = check(&m, 4, "@on_start\ndef go():\n    Routing.next_exti()\n", &[routing]);
        assert!(bad[0].message.contains("next_exit"));
    }

    #[test]
    fn suggests_close_names_only() {
        assert_eq!(suggest("Roler3", ["Roller3", "Source1"].into_iter()), Some("Roller3"));
        assert_eq!(suggest("xyz", ["Roller3"].into_iter()), None);
    }
}

#[cfg(test)]
mod address_tests {
    use super::*;

    /// The test model of `compile::tests`, with Roller3 in section `Seccion1`, so its address is `Seccion1.Roller3`.
    fn addressed() -> ModelInfo {
        let mut m = super::tests::model();
        m.objects[1].address = "Seccion1.Roller3".into();
        m
    }

    #[test]
    fn a_whole_address_names_its_object_in_code() {
        let d =
            check(&addressed(), 99, "@on_start\ndef go():\n    Seccion1.Roller3.run = False\n    print(Seccion1.Roller3.length)\n", &[]);
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn a_misspelt_member_under_an_address_is_found_with_a_suggestion() {
        let d = check(&addressed(), 99, "@on_start\ndef go():\n    Seccion1.Roller3.rnu = False\n", &[]);
        assert_eq!(d.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), ["unknownMember"]);
        assert!(d[0].message.contains("run"), "{}", d[0].message);
    }

    #[test]
    fn an_output_under_an_address_is_still_read_only() {
        let d = check(&addressed(), 99, "@on_start\ndef go():\n    Seccion1.Roller3.occupied = True\n", &[]);
        assert_eq!(d.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), ["notWritable"]);
    }

    #[test]
    fn events_take_an_object_by_its_address() {
        let mut m = addressed();
        m.objects[2].address = "Seccion1.Photoeye3".into();
        let ok = check(
            &m,
            99,
            "@on_enter(Seccion1.Roller3)\ndef seen(pallet):\n    pass\n\n@on_rise(Seccion1.Photoeye3.detected)\ndef beam():\n    pass\n",
            &[],
        );
        assert!(ok.is_empty(), "{ok:?}");
        let bad = check(&m, 99, "@on_enter(Seccion1.Roler3)\ndef seen(pallet):\n    pass\n", &[]);
        assert_eq!(bad[0].code, "unknownName");
        assert!(bad[0].message.contains("Seccion1.Roller3"), "{}", bad[0].message);
    }

    #[test]
    fn a_plain_name_still_works_next_to_an_address() {
        let d = check(&addressed(), 99, "@on_start\ndef go():\n    Roller3.run = False\n", &[]);
        assert!(d.is_empty(), "{d:?}");
    }
}
