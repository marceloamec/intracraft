//! Runs checked code blocks inside a simulation (A17 §14). Each event starts an **activation** of its handler; an
//! activation runs until it ends or waits, and nothing else runs meanwhile. A waiting activation keeps its whole state
//! (loops, calls in progress) and resumes in event order: by time, then priority, then the order it started in. Nothing
//! here reads the clock, files or the network, and every random number comes from the block's seeded stream, so the same
//! layout, code and seed always give the same run.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::builtins::{self, Builtin, Rng};
use crate::compile::{Edge, EventSpec, Function, NONE, Op, Program, Source, compile_blocks};
use crate::diag::Diagnostic;
use crate::host::{Host, MemberKind, ModelInfo, PALLET_MEMBERS};
use crate::ops::{self, Fault, MAX_ITEMS};
use crate::units::Dim;
use crate::value::{Dict, Value, lock};

/// Instructions one activation may run without waiting (A17 §14.5).
pub const MAX_STEPS: u64 = 1_000_000;
/// Calls inside calls.
pub const MAX_DEPTH: usize = 200;
/// Activations started at one instant (code and signals that keep triggering each other).
pub const MAX_PER_INSTANT: u32 = 1_000;
/// Activations waiting at once.
pub const MAX_WAITING: usize = 10_000;
/// Lines the console keeps (the oldest go first).
pub const MAX_CONSOLE: usize = 10_000;

/// Something that happened in the model, as the engine reports it.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Create { source: u32, pallet: u64 },
    Enter { object: u32, pallet: u64 },
    Leave { object: u32, pallet: u64 },
    Signal { object: u32, signal: String, value: bool },
}

/// A line the code printed.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleLine {
    /// Simulation time, µs.
    pub time: i64,
    pub block: String,
    pub warning: bool,
    pub text: String,
}

/// An error that stopped the run: where in the code, when, and what.
#[derive(Clone, Debug, PartialEq)]
pub struct RunError {
    pub block: String,
    pub line: u32,
    /// Simulation time, µs.
    pub time: i64,
    pub code: String,
    pub message: String,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, line {}: {} [{}] (at {})", self.block, self.line, self.message, self.code, clock(self.time))
    }
}

/// A time as h:mm:ss.sss.
pub fn clock(us: i64) -> String {
    let ms = us.max(0) / 1000;
    format!("{}:{:02}:{:02}.{:03}", ms / 3_600_000, (ms / 60_000) % 60, (ms / 1000) % 60, ms % 1000)
}

/// What a code block did during the run (S21 §7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockStats {
    pub activations: u64,
    pub steps: u64,
}

struct Frame {
    block: u32,
    func: u32,
    pc: usize,
    base: usize,
    locals: Vec<Value>,
}

struct Activation {
    id: u64,
    frames: Vec<Frame>,
    stack: Vec<Value>,
    priority: i64,
    /// A decision or a finish handler: it runs to its end at once.
    no_wait: bool,
    steps: u64,
    block: u32,
}

#[derive(Clone, Copy, Debug)]
enum Wake {
    At(i64),
    /// `wait_until`: looked at again after every event, until its deadline.
    Poll(Option<i64>),
}

enum Flow {
    Returned(Value),
    Suspended(Wake),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Timer {
    /// Resume an activation, if it still waits for this (its token).
    Resume(u64, u64),
    Every(u32, u32),
    At(u32, u32),
}

struct BlockState {
    name: String,
    object: u32,
    program: Arc<Program>,
    globals: Vec<Value>,
    rng: Rng,
    stats: BlockStats,
}

/// Every code block of a run, and its activations.
pub struct Runtime {
    model: ModelInfo,
    blocks: Vec<BlockState>,
    /// Waiting activations, with what they wait for and the token of that wait.
    waiting: BTreeMap<u64, (Activation, Wake, u64)>,
    timers: BTreeMap<(i64, i64, u64), Timer>,
    seq: u64,
    pallet_attrs: BTreeMap<u64, Dict>,
    console: Vec<ConsoleLine>,
    dropped: u64,
    instant: i64,
    started_now: u32,
}

impl Runtime {
    /// Checks and compiles every code block (`object` in the model, its code); the errors of every block that has any.
    pub fn build(model: ModelInfo, sources: &[Source], seed: u64) -> Result<Runtime, Vec<(u32, Vec<Diagnostic>)>> {
        let compiled = compile_blocks(&model, sources);
        let mut errors = Vec::new();
        let mut programs = Vec::new();
        for (s, c) in sources.iter().zip(compiled) {
            match c.program {
                Some(p) => programs.push((s.object, p)),
                None => errors.push((s.object, c.diagnostics)),
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        programs.sort_by_key(|(o, _)| *o);
        let blocks = programs
            .into_iter()
            .map(|(object, program)| {
                let info = model.object(object);
                let name = info.map_or_else(|| format!("block {object}"), |o| o.name.clone());
                let stream = info.map_or(u64::from(object), |o| fnv(&o.id));
                BlockState {
                    name,
                    object,
                    globals: vec![Value::None; program.globals.len()],
                    program: Arc::new(program),
                    rng: Rng::new(seed, stream),
                    stats: BlockStats::default(),
                }
            })
            .collect();
        Ok(Runtime {
            model,
            blocks,
            waiting: BTreeMap::new(),
            timers: BTreeMap::new(),
            seq: 0,
            pallet_attrs: BTreeMap::new(),
            console: Vec::new(),
            dropped: 0,
            instant: -1,
            started_now: 0,
        })
    }

    /// Time 0: the blocks' variables get their values, the timers are set and the `@on_start` handlers run.
    pub fn start(&mut self, host: &mut dyn Host) -> Result<(), RunError> {
        for b in 0..self.blocks.len() as u32 {
            let init = self.blocks[b as usize].program.init;
            self.run_sync(b, init, Vec::new(), host)?;
        }
        for (b, block) in self.blocks.iter().enumerate() {
            for (h, handler) in block.program.handlers.iter().enumerate() {
                let (b, h) = (b as u32, h as u32);
                match handler.event {
                    EventSpec::Every { first, .. } => {
                        self.seq += 1;
                        self.timers.insert((first, -handler.priority, self.seq), Timer::Every(b, h));
                    }
                    EventSpec::At(t) => {
                        self.seq += 1;
                        self.timers.insert((t, -handler.priority, self.seq), Timer::At(b, h));
                    }
                    _ => {}
                }
            }
        }
        let starts = self.matching(|e| *e == EventSpec::Start);
        for (b, h) in starts {
            self.start_handler(b, h, None, None, host)?;
        }
        self.react(&[], host)
    }

    /// Runs what is due at the host's time: timers, then a handler for each event in order, then every `wait_until`
    /// looks at its condition again.
    pub fn react(&mut self, events: &[Event], host: &mut dyn Host) -> Result<(), RunError> {
        let now = host.now();
        while let Some((&key, &timer)) = self.timers.iter().next() {
            if key.0 > now {
                break;
            }
            self.timers.remove(&key);
            match timer {
                Timer::Resume(id, token) => {
                    if self.waiting.get(&id).is_some_and(|(_, _, t)| *t == token)
                        && let Some((act, ..)) = self.waiting.remove(&id)
                    {
                        self.resume(act, host)?;
                    }
                }
                Timer::Every(b, h) => {
                    if let EventSpec::Every { interval, .. } = self.handler_event(b, h) {
                        self.seq += 1;
                        self.timers.insert((key.0 + interval, key.1, self.seq), Timer::Every(b, h));
                    }
                    self.start_handler(b, h, None, None, host)?;
                }
                Timer::At(b, h) => self.start_handler(b, h, None, None, host)?,
            }
        }
        for e in events {
            let (pallet, sender) = match e {
                Event::Create { source, pallet } => (Some(*pallet), Some(*source)),
                Event::Enter { object, pallet } | Event::Leave { object, pallet } => (Some(*pallet), Some(*object)),
                Event::Signal { object, .. } => (None, Some(*object)),
            };
            let handlers = self.matching(|spec| matches_event(spec, e));
            for (b, h) in handlers {
                self.start_handler(b, h, pallet, sender, host)?;
            }
        }
        let polling: Vec<u64> = self.waiting.iter().filter(|(_, (_, w, _))| matches!(w, Wake::Poll(_))).map(|(id, _)| *id).collect();
        for id in polling {
            if let Some((act, ..)) = self.waiting.remove(&id) {
                self.resume(act, host)?;
            }
        }
        Ok(())
    }

    /// When the next timer is due, µs.
    pub fn next_wake(&self) -> Option<i64> {
        self.timers.keys().next().map(|k| k.0)
    }

    /// Whether code chooses the exits of this object.
    pub fn decides_exit(&self, object: u32) -> bool {
        !self.matching(|e| *e == EventSpec::ChooseExit(object)).is_empty()
    }

    /// Asks the `@choose_exit` handler of `object` where `pallet` goes: one of `candidates`, or `None` to let the object's
    /// own rule decide.
    pub fn choose_exit(&mut self, object: u32, pallet: u64, candidates: &[u32], host: &mut dyn Host) -> Result<Option<u32>, RunError> {
        let Some((b, h)) = self.matching(|e| *e == EventSpec::ChooseExit(object)).first().copied() else { return Ok(None) };
        let func = self.handler_func(b, h);
        let args = self.handler_args(b, func, Some(pallet), Some(object));
        self.blocks[b as usize].stats.activations += 1;
        let line = self.handler_line(b, h);
        match self.run_sync(b, func, args, host)? {
            Value::None => Ok(None),
            Value::Object(o) if candidates.contains(&o) => Ok(Some(o)),
            other => {
                let names: Vec<String> = candidates.iter().filter_map(|c| self.model.object(*c)).map(|o| o.name.clone()).collect();
                let what = match &other {
                    Value::Object(o) => self.model.object(*o).map_or("that object".to_string(), |i| format!("`{}`", i.name)),
                    v => format!("a {}", v.type_name()),
                };
                Err(RunError {
                    block: self.blocks[b as usize].name.clone(),
                    line,
                    time: host.now(),
                    code: "notAnExit".into(),
                    message: format!("the exit decision returned {what}; it must return one of {} or None", names.join(", ")),
                })
            }
        }
    }

    /// The end of the run: the `@on_finish` handlers run, waiting activations are dropped.
    pub fn finish(&mut self, host: &mut dyn Host) -> Result<(), RunError> {
        let ends = self.matching(|e| *e == EventSpec::Finish);
        for (b, h) in ends {
            let func = self.handler_func(b, h);
            let args = self.handler_args(b, func, None, None);
            self.run_sync(b, func, args, host)?;
        }
        self.waiting.clear();
        self.timers.clear();
        Ok(())
    }

    /// A pallet left the model: its attributes go with it.
    pub fn forget_pallet(&mut self, pallet: u64) {
        self.pallet_attrs.remove(&pallet);
    }

    pub fn console(&self) -> &[ConsoleLine] {
        &self.console
    }

    /// Lines dropped because the console was full.
    pub fn dropped_lines(&self) -> u64 {
        self.dropped
    }

    /// Each block's object, statistics and activations waiting now.
    pub fn block_stats(&self) -> Vec<(u32, BlockStats, usize)> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(b, s)| (s.object, s.stats, self.waiting.values().filter(|(a, ..)| a.block as usize == b).count()))
            .collect()
    }

    /// A block's variables and their values now, as text (the editor's watch list).
    pub fn variables(&self, block_object: u32) -> Vec<(String, String)> {
        let Some(b) = self.blocks.iter().find(|b| b.object == block_object) else { return Vec::new() };
        b.program
            .globals
            .iter()
            .zip(&b.globals)
            .filter(|(_, v)| !matches!(v, Value::Func { .. }))
            .map(|(n, v)| (n.clone(), v.repr()))
            .collect()
    }

    /// The pallet's attributes code gave it.
    pub fn pallet_attributes(&self, pallet: u64) -> Vec<(String, String)> {
        self.pallet_attrs.get(&pallet).map(|d| d.items().iter().map(|(k, v)| (k.to_text(), v.repr())).collect()).unwrap_or_default()
    }

    // ---- activations ----

    /// Handlers whose event matches, in the order they run: higher priority first, then block and line order.
    fn matching(&self, f: impl Fn(&EventSpec) -> bool) -> Vec<(u32, u32)> {
        let mut out: Vec<(i64, u32, u32)> = Vec::new();
        for (b, block) in self.blocks.iter().enumerate() {
            for (h, handler) in block.program.handlers.iter().enumerate() {
                if f(&handler.event) {
                    out.push((-handler.priority, b as u32, h as u32));
                }
            }
        }
        out.sort();
        out.into_iter().map(|(_, b, h)| (b, h)).collect()
    }

    fn handler_event(&self, b: u32, h: u32) -> EventSpec {
        self.blocks[b as usize].program.handlers.get(h as usize).map_or(EventSpec::Start, |x| x.event.clone())
    }

    fn handler_func(&self, b: u32, h: u32) -> u32 {
        self.blocks[b as usize].program.handlers.get(h as usize).map_or(0, |x| x.func)
    }

    fn handler_line(&self, b: u32, h: u32) -> u32 {
        self.blocks[b as usize].program.handlers.get(h as usize).map_or(0, |x| x.line)
    }

    /// A handler's locals: `pallet` and `sender` by name, defaults for the rest.
    fn handler_args(&self, b: u32, func: u32, pallet: Option<u64>, sender: Option<u32>) -> Vec<Value> {
        let Some(f) = self.blocks[b as usize].program.functions.get(func as usize) else { return Vec::new() };
        let first_default = f.params.len() - f.defaults.len();
        f.params
            .iter()
            .enumerate()
            .map(|(i, p)| match p.as_str() {
                "pallet" => pallet.map_or(Value::None, Value::Pallet),
                "sender" => sender.map_or(Value::None, Value::Object),
                _ => f.defaults.get(i.wrapping_sub(first_default)).cloned().unwrap_or(Value::None),
            })
            .collect()
    }

    fn start_handler(&mut self, b: u32, h: u32, pallet: Option<u64>, sender: Option<u32>, host: &mut dyn Host) -> Result<(), RunError> {
        let now = host.now();
        if now != self.instant {
            self.instant = now;
            self.started_now = 0;
        }
        self.started_now += 1;
        let func = self.handler_func(b, h);
        let line = self.handler_line(b, h);
        let block = &self.blocks[b as usize];
        if self.started_now > MAX_PER_INSTANT {
            return Err(RunError {
                block: block.name.clone(),
                line,
                time: now,
                code: "cascadeLimit".into(),
                message: format!(
                    "more than {MAX_PER_INSTANT} handlers started at this instant: code and signals keep triggering each other"
                ),
            });
        }
        let priority = block.program.handlers.get(h as usize).map_or(0, |x| x.priority);
        let args = self.handler_args(b, func, pallet, sender);
        let frame = self.frame(b, func, args, Vec::new(), 0).map_err(|f| self.error_at(b, line, now, f))?;
        self.seq += 1;
        let act = Activation { id: self.seq, frames: vec![frame], stack: Vec::new(), priority, no_wait: false, steps: 0, block: b };
        self.blocks[b as usize].stats.activations += 1;
        self.resume(act, host)
    }

    fn resume(&mut self, mut act: Activation, host: &mut dyn Host) -> Result<(), RunError> {
        act.steps = 0;
        let result = self.exec(&mut act, host, 0);
        self.blocks[act.block as usize].stats.steps += act.steps;
        match result {
            Err(f) => Err(self.error_in(&act, host.now(), f)),
            Ok(Flow::Returned(_)) => Ok(()),
            Ok(Flow::Suspended(wake)) => {
                if self.waiting.len() >= MAX_WAITING {
                    return Err(self.error_in(
                        &act,
                        host.now(),
                        Fault::new("tooLarge", format!("more than {MAX_WAITING} handlers wait at once")),
                    ));
                }
                self.seq += 1;
                let token = self.seq;
                let deadline = match wake {
                    Wake::At(t) => Some(t),
                    Wake::Poll(d) => d,
                };
                if let Some(t) = deadline {
                    self.timers.insert((t, -act.priority, token), Timer::Resume(act.id, token));
                }
                self.waiting.insert(act.id, (act, wake, token));
                Ok(())
            }
        }
    }

    /// Runs a function to its end at once (a block's start, a decision, a finish handler).
    fn run_sync(&mut self, b: u32, func: u32, args: Vec<Value>, host: &mut dyn Host) -> Result<Value, RunError> {
        let now = host.now();
        let line = self.blocks[b as usize].program.functions.get(func as usize).map_or(0, |f| f.line);
        let frame = self.frame(b, func, args, Vec::new(), 0).map_err(|f| self.error_at(b, line, now, f))?;
        self.seq += 1;
        let mut act = Activation { id: self.seq, frames: vec![frame], stack: Vec::new(), priority: 0, no_wait: true, steps: 0, block: b };
        let result = self.exec(&mut act, host, 0);
        self.blocks[b as usize].stats.steps += act.steps;
        match result {
            Ok(Flow::Returned(v)) => Ok(v),
            Ok(Flow::Suspended(_)) => Err(self.error_in(&act, now, Fault::new("waitInDecision", "this code can't wait"))),
            Err(f) => Err(self.error_in(&act, now, f)),
        }
    }

    fn error_at(&self, b: u32, line: u32, time: i64, f: Fault) -> RunError {
        RunError {
            block: self.blocks.get(b as usize).map_or_else(String::new, |x| x.name.clone()),
            line,
            time,
            code: f.code.into(),
            message: f.message,
        }
    }

    /// A fault where the activation stands: the block and line of its innermost call.
    fn error_in(&self, act: &Activation, time: i64, f: Fault) -> RunError {
        let (b, line) = match act.frames.last() {
            Some(fr) => {
                let line = self.blocks[fr.block as usize]
                    .program
                    .functions
                    .get(fr.func as usize)
                    .and_then(|func| func.lines.get(fr.pc.saturating_sub(1)))
                    .copied()
                    .unwrap_or(0);
                (fr.block, line)
            }
            None => (act.block, 0),
        };
        self.error_at(b, line, time, f)
    }

    /// A new call's frame with its arguments bound to the parameters.
    fn frame(&self, b: u32, func: u32, args: Vec<Value>, kwargs: Vec<(Arc<str>, Value)>, base: usize) -> Result<Frame, Fault> {
        let f = self.blocks[b as usize].program.functions.get(func as usize).ok_or_else(|| Fault::new("internal", "no such function"))?;
        Ok(Frame { block: b, func, pc: 0, base, locals: bind(f, args, kwargs)? })
    }

    // ---- the interpreter ----

    fn exec(&mut self, act: &mut Activation, host: &mut dyn Host, stop_depth: usize) -> Result<Flow, Fault> {
        let mut cache: Option<(u32, Arc<Program>)> = None;
        loop {
            let Some(fi) = act.frames.len().checked_sub(1) else { return Ok(Flow::Returned(Value::None)) };
            let b = act.frames[fi].block;
            let program = match &cache {
                Some((cb, p)) if *cb == b => p.clone(),
                _ => {
                    let p = self.blocks[b as usize].program.clone();
                    cache = Some((b, p.clone()));
                    p
                }
            };
            let func = program.functions.get(act.frames[fi].func as usize).ok_or_else(|| Fault::new("internal", "no such function"))?;
            let pc = act.frames[fi].pc;
            let op = *func.code.get(pc).ok_or_else(|| Fault::new("internal", "ran past the end of a function"))?;
            act.frames[fi].pc += 1;
            act.steps += 1;
            if act.steps > MAX_STEPS {
                return Err(Fault::new("stepLimit", format!("more than {MAX_STEPS} steps without waiting: a loop that never ends?")));
            }
            match op {
                Op::Const(k) => act.stack.push(program.consts.get(k as usize).cloned().unwrap_or_default()),
                Op::LoadLocal(s) => {
                    let v = act.frames[fi].locals.get(s as usize).cloned().unwrap_or(Value::Unbound);
                    if matches!(v, Value::Unbound) {
                        let name = func.locals.get(s as usize).map_or("?", String::as_str);
                        return Err(Fault::new(
                            "unboundLocal",
                            format!("`{name}` is used before it gets a value (to use the block's variable, write `global {name}` first)"),
                        ));
                    }
                    act.stack.push(v);
                }
                Op::StoreLocal(s) => {
                    let v = pop(act)?;
                    if let Some(slot) = act.frames[fi].locals.get_mut(s as usize) {
                        *slot = v;
                    }
                }
                Op::LoadGlobal(g) => act.stack.push(self.blocks[b as usize].globals.get(g as usize).cloned().unwrap_or_default()),
                Op::StoreGlobal(g) => {
                    let v = pop(act)?;
                    if let Some(slot) = self.blocks[b as usize].globals.get_mut(g as usize) {
                        *slot = v;
                    }
                }
                Op::LoadObject(o) => act.stack.push(Value::Object(o)),
                Op::LoadBuiltin(x) => act.stack.push(Value::Builtin(x)),
                Op::LoadFunc(f) => act.stack.push(Value::Func { block: b, func: f }),
                Op::GetAttr(n) => {
                    let base = pop(act)?;
                    let name = name_of(&program, n);
                    let v = self.get_attr(&base, &name, host)?;
                    act.stack.push(v);
                }
                Op::SetAttr(n) => {
                    let base = pop(act)?;
                    let value = pop(act)?;
                    let name = name_of(&program, n);
                    self.set_attr(&base, &name, value, host)?;
                }
                Op::GetItem => {
                    let index = pop(act)?;
                    let base = pop(act)?;
                    act.stack.push(ops::get_item(&base, &index)?);
                }
                Op::SetItem => {
                    let index = pop(act)?;
                    let base = pop(act)?;
                    let value = pop(act)?;
                    ops::set_item(&base, &index, value)?;
                }
                Op::BuildSlice => {
                    let step = slice_part(pop(act)?)?;
                    let stop = slice_part(pop(act)?)?;
                    let start = slice_part(pop(act)?)?;
                    act.stack.push(Value::Slice(start, stop, step));
                }
                Op::Unary(o) => {
                    let v = pop(act)?;
                    act.stack.push(ops::unary(o, &v)?);
                }
                Op::Binary(o) => {
                    let y = pop(act)?;
                    let x = pop(act)?;
                    act.stack.push(ops::binary(o, &x, &y)?);
                }
                Op::Compare(o) => {
                    let y = pop(act)?;
                    let x = pop(act)?;
                    act.stack.push(Value::Bool(ops::compare(o, &x, &y)?));
                }
                Op::Jump(t) => act.frames[fi].pc = t as usize,
                Op::JumpIfFalse(t) => {
                    if !pop(act)?.truthy() {
                        act.frames[fi].pc = t as usize;
                    }
                }
                Op::JumpIfTrue(t) => {
                    if pop(act)?.truthy() {
                        act.frames[fi].pc = t as usize;
                    }
                }
                Op::JumpIfFalseKeep(t) | Op::JumpIfTrueKeep(t) => {
                    let truth = act.stack.last().is_some_and(Value::truthy);
                    if truth == matches!(op, Op::JumpIfTrueKeep(_)) {
                        act.frames[fi].pc = t as usize;
                    } else {
                        pop(act)?;
                    }
                }
                Op::Pop => {
                    pop(act)?;
                }
                Op::Dup => {
                    let v = act.stack.last().cloned().ok_or_else(stack_fault)?;
                    act.stack.push(v);
                }
                Op::Dup2 => {
                    let n = act.stack.len();
                    if n < 2 {
                        return Err(stack_fault());
                    }
                    let (x, y) = (act.stack[n - 2].clone(), act.stack[n - 1].clone());
                    act.stack.push(x);
                    act.stack.push(y);
                }
                Op::Swap => {
                    let n = act.stack.len();
                    if n < 2 {
                        return Err(stack_fault());
                    }
                    act.stack.swap(n - 1, n - 2);
                }
                Op::Rot3 => {
                    let n = act.stack.len();
                    if n < 3 {
                        return Err(stack_fault());
                    }
                    let top = act.stack.remove(n - 1);
                    act.stack.insert(n - 3, top);
                }
                Op::BuildList(n) => {
                    let items = pop_n(act, n as usize)?;
                    act.stack.push(Value::list(items));
                }
                Op::BuildTuple(n) => {
                    let items = pop_n(act, n as usize)?;
                    act.stack.push(Value::tuple(items));
                }
                Op::BuildDict(n) => {
                    let items = pop_n(act, 2 * n as usize)?;
                    let mut d = Dict::default();
                    for pair in items.chunks(2) {
                        if let [k, v] = pair {
                            d.set(k.clone(), v.clone()).map_err(Fault::types)?;
                        }
                    }
                    act.stack.push(Value::dict(d));
                }
                Op::ListAppend(depth) => {
                    let v = pop(act)?;
                    let at = act.stack.len().checked_sub(depth as usize).ok_or_else(stack_fault)?;
                    if let Some(Value::List(l)) = act.stack.get(at) {
                        let mut items = lock(l);
                        if items.len() >= MAX_ITEMS {
                            return Err(Fault::new("tooLarge", format!("a list holds at most {MAX_ITEMS} items")));
                        }
                        items.push(v);
                    }
                }
                Op::Unpack(n) => {
                    let v = pop(act)?;
                    let items = builtins::items(&v)?;
                    if items.len() != n as usize {
                        return Err(Fault::new("wrongArguments", format!("expected {n} values to unpack, got {}", items.len())));
                    }
                    act.stack.extend(items.into_iter().rev());
                }
                Op::GetIter => {
                    let v = pop(act)?;
                    act.stack.push(Value::Iter(Arc::new(Mutex::new(builtins::iterate(&v)?))));
                }
                Op::ForIter(t) => {
                    let next = match act.stack.last() {
                        Some(Value::Iter(it)) => lock(it).next(),
                        _ => return Err(stack_fault()),
                    };
                    match next {
                        Some(v) => act.stack.push(v),
                        None => {
                            pop(act)?;
                            act.frames[fi].pc = t as usize;
                        }
                    }
                }
                Op::Call(n, kw) => {
                    let kwnames: Vec<Arc<str>> = if kw == NONE {
                        Vec::new()
                    } else {
                        match program.consts.get(kw as usize) {
                            Some(Value::Tuple(t)) => t.iter().map(|v| Arc::from(v.to_text())).collect(),
                            _ => Vec::new(),
                        }
                    };
                    let kwvalues = pop_n(act, kwnames.len())?;
                    let args = pop_n(act, n as usize)?;
                    let callee = pop(act)?;
                    let kwargs: Vec<(Arc<str>, Value)> = kwnames.into_iter().zip(kwvalues).collect();
                    if let Some(v) = self.call(&callee, args, kwargs, act, host)? {
                        act.stack.push(v);
                    }
                }
                Op::Return => {
                    let v = pop(act)?;
                    let Some(frame) = act.frames.pop() else { return Ok(Flow::Returned(v)) };
                    act.stack.truncate(frame.base);
                    if act.frames.len() <= stop_depth {
                        return Ok(Flow::Returned(v));
                    }
                    act.stack.push(v);
                }
                Op::Wait => {
                    let d = pop(act)?;
                    let us = duration(&d)?;
                    if act.no_wait || stop_depth > 0 {
                        return Err(Fault::new("waitInDecision", "this code can't wait: it must answer at once"));
                    }
                    act.stack.push(Value::None);
                    return Ok(Flow::Suspended(Wake::At(host.now().saturating_add(us))));
                }
                Op::WaitUntilStart(slot) => {
                    let t = pop(act)?;
                    let deadline = match t {
                        Value::None => Value::None,
                        other => Value::Int(host.now().saturating_add(duration(&other)?)),
                    };
                    if let Some(s) = act.frames[fi].locals.get_mut(slot as usize) {
                        *s = deadline;
                    }
                }
                Op::WaitUntilPause(slot, target) => {
                    let deadline = match act.frames[fi].locals.get(slot as usize) {
                        Some(Value::Int(d)) => Some(*d),
                        _ => None,
                    };
                    if deadline.is_some_and(|d| host.now() >= d) {
                        act.frames[fi].pc = target as usize;
                    } else {
                        if act.no_wait || stop_depth > 0 {
                            return Err(Fault::new("waitInDecision", "this code can't wait: it must answer at once"));
                        }
                        return Ok(Flow::Suspended(Wake::Poll(deadline)));
                    }
                }
                Op::Format(k) => {
                    let v = pop(act)?;
                    let spec =
                        if k == NONE { String::new() } else { program.consts.get(k as usize).map(Value::to_text).unwrap_or_default() };
                    act.stack.push(Value::str(&ops::format_value(&v, &spec)?));
                }
                Op::BuildString(n) => {
                    let parts = pop_n(act, n as usize)?;
                    let s: String = parts.iter().map(Value::to_text).collect();
                    if s.chars().count() > MAX_ITEMS {
                        return Err(Fault::new("tooLarge", "the text grows too long"));
                    }
                    act.stack.push(Value::str(&s));
                }
                Op::Assert => {
                    let msg = pop(act)?;
                    let text = if matches!(msg, Value::None) { "an assert failed".to_string() } else { msg.to_text() };
                    return Err(Fault::new("assertionFailed", text));
                }
            }
        }
    }

    /// Calls a value. `None` when the call pushed a frame (a function of a block) that the loop now runs.
    fn call(
        &mut self,
        callee: &Value,
        args: Vec<Value>,
        kwargs: Vec<(Arc<str>, Value)>,
        act: &mut Activation,
        host: &mut dyn Host,
    ) -> Result<Option<Value>, Fault> {
        match callee {
            Value::Func { block, func } => {
                if act.frames.len() >= MAX_DEPTH {
                    return Err(Fault::new("depthLimit", format!("functions call each other more than {MAX_DEPTH} deep")));
                }
                let frame = self.frame(*block, *func, args, kwargs, act.stack.len())?;
                act.frames.push(frame);
                Ok(None)
            }
            Value::Builtin(b) => self.call_builtin(*b, args, kwargs, act, host).map(Some),
            Value::Method(m) => {
                let (recv, name) = (&m.0, m.1.as_ref());
                match recv {
                    Value::Object(o) => {
                        if let Some(block) = self.block_of(*o) {
                            let program = self.blocks[block as usize].program.clone();
                            let f =
                                program.function(name).ok_or_else(|| Fault::new("unknownAttribute", format!("no function `{name}`")))?;
                            return self.call(&Value::Func { block, func: f }, args, kwargs, act, host);
                        }
                        if let Some((k, _)) = kwargs.first() {
                            return Err(Fault::new("wrongArguments", format!("{name}() takes no argument `{k}`")));
                        }
                        host.call(*o, name, &args).map(Some)
                    }
                    Value::List(l) if name == "sort" && kwargs.iter().any(|(k, _)| k.as_ref() == "key") => {
                        let items = lock(l).clone();
                        let (key, reverse) = sort_options(&kwargs)?;
                        let keys = self.keys(&items, key.as_ref(), act, host)?;
                        *lock(l) = builtins::sort_by_keys(items, keys, reverse)?;
                        Ok(Some(Value::None))
                    }
                    _ => builtins::method(recv, name, &args, &kwargs).map(Some),
                }
            }
            other => Err(Fault::types(format!("a {} can't be called", other.type_name()))),
        }
    }

    /// Calls a function and waits for its result here (a sort key); it may not wait.
    fn call_sync(&mut self, f: &Value, args: Vec<Value>, act: &mut Activation, host: &mut dyn Host) -> Result<Value, Fault> {
        match self.call(f, args, Vec::new(), act, host)? {
            Some(v) => Ok(v),
            None => {
                let depth = act.frames.len() - 1;
                match self.exec(act, host, depth)? {
                    Flow::Returned(v) => Ok(v),
                    Flow::Suspended(_) => Err(Fault::new("waitInDecision", "a key function can't wait")),
                }
            }
        }
    }

    fn keys(&mut self, items: &[Value], key: Option<&Value>, act: &mut Activation, host: &mut dyn Host) -> Result<Vec<Value>, Fault> {
        match key {
            None | Some(Value::None) => Ok(items.to_vec()),
            Some(f) => items.iter().map(|v| self.call_sync(f, vec![v.clone()], act, host)).collect(),
        }
    }

    fn call_builtin(
        &mut self,
        b: Builtin,
        args: Vec<Value>,
        kwargs: Vec<(Arc<str>, Value)>,
        act: &mut Activation,
        host: &mut dyn Host,
    ) -> Result<Value, Fault> {
        let block = act.frames.last().map_or(act.block, |f| f.block) as usize;
        match b {
            Builtin::Print | Builtin::Warn => {
                if let Some((k, _)) = kwargs.first() {
                    return Err(Fault::new("wrongArguments", format!("print() takes no argument `{k}`")));
                }
                let text: Vec<String> = args.iter().map(|v| self.show(v)).collect();
                let line = ConsoleLine {
                    time: host.now(),
                    block: self.blocks[block].name.clone(),
                    warning: b == Builtin::Warn,
                    text: text.join(" "),
                };
                if self.console.len() >= MAX_CONSOLE {
                    self.console.remove(0);
                    self.dropped += 1;
                }
                self.console.push(line);
                Ok(Value::None)
            }
            Builtin::Error => Err(Fault::new("userError", args.iter().map(Value::to_text).collect::<Vec<_>>().join(" "))),
            Builtin::Now => Ok(Value::Qty(host.now() as f64 / 1e6, Dim::TIME)),
            Builtin::SimStop => {
                let reason = args.first().map(Value::to_text).unwrap_or_default();
                host.stop(&reason);
                Ok(Value::None)
            }
            Builtin::RandomRandom
            | Builtin::RandomUniform
            | Builtin::RandomNormal
            | Builtin::RandomExponential
            | Builtin::RandomTriangular
            | Builtin::RandomRandint
            | Builtin::RandomChance
            | Builtin::RandomChoice => builtins::random(b, &mut self.blocks[block].rng, &args, &kwargs),
            Builtin::Min | Builtin::Max => {
                let (key, _) = sort_options(&kwargs)?;
                let candidates = if args.len() == 1 { builtins::items(&args[0])? } else { args };
                let keys = match key {
                    Some(k) => Some(self.keys(&candidates, Some(&k), act, host)?),
                    None => None,
                };
                builtins::extreme(b == Builtin::Max, candidates, keys)
            }
            Builtin::Sorted => {
                if args.len() != 1 {
                    return Err(Fault::new("wrongArguments", "sorted() takes one list"));
                }
                let (key, reverse) = sort_options(&kwargs)?;
                let items = builtins::items(&args[0])?;
                let keys = self.keys(&items, key.as_ref(), act, host)?;
                Ok(Value::list(builtins::sort_by_keys(items, keys, reverse)?))
            }
            Builtin::Math | Builtin::Random | Builtin::Simulation => {
                Err(Fault::types("a module can't be called; call one of its functions"))
            }
            other => builtins::pure(other, &args, &kwargs),
        }
    }

    /// What `print` shows for a value: an object by its name in the layout (also inside a list or tuple), anything else as
    /// `str()` gives it.
    fn show(&self, v: &Value) -> String {
        match v {
            Value::Object(o) => self.model.object(*o).map_or_else(|| v.to_text(), |i| i.name.clone()),
            Value::List(l) => format!("[{}]", lock(l).iter().map(|x| self.show_inner(x)).collect::<Vec<_>>().join(", ")),
            Value::Tuple(t) => format!("({})", t.iter().map(|x| self.show_inner(x)).collect::<Vec<_>>().join(", ")),
            other => other.to_text(),
        }
    }

    /// Inside a container, texts keep their quotes, as `repr` writes them.
    fn show_inner(&self, v: &Value) -> String {
        match v {
            Value::Object(_) | Value::List(_) | Value::Tuple(_) => self.show(v),
            other => other.repr(),
        }
    }

    fn block_of(&self, object: u32) -> Option<u32> {
        self.blocks.iter().position(|b| b.object == object).map(|i| i as u32)
    }

    fn get_attr(&mut self, base: &Value, name: &str, host: &mut dyn Host) -> Result<Value, Fault> {
        match base {
            Value::Object(o) => {
                if let Some(b) = self.block_of(*o) {
                    let block = &self.blocks[b as usize];
                    return match block.program.global(name) {
                        Some(g) => Ok(block.globals.get(g as usize).cloned().unwrap_or_default()),
                        None => Err(Fault::new("unknownAttribute", format!("code block `{}` has no `{name}`", block.name))),
                    };
                }
                if self.model.object(*o).and_then(|i| i.member(name)).is_some_and(|m| m.kind == MemberKind::Method) {
                    return Ok(Value::Method(Arc::new((base.clone(), Arc::from(name)))));
                }
                host.get(*o, name)
            }
            Value::Pallet(id) => {
                if name == "attributes" {
                    return Ok(Value::dict(self.pallet_attrs.get(id).cloned().unwrap_or_default()));
                }
                if let Some(v) = host.pallet(*id, name)? {
                    return Ok(v);
                }
                self.pallet_attrs.get(id).and_then(|d| d.get(&Value::str(name)).ok().flatten()).ok_or_else(|| {
                    Fault::new("unknownAttribute", format!("pallet {id} has no attribute `{name}`; give it one first: pallet.{name} = …"))
                })
            }
            Value::Builtin(Builtin::Simulation) => match name {
                "mode" => Ok(Value::str(host.mode())),
                "seed" => Ok(Value::Int(i64::try_from(host.seed()).unwrap_or(i64::MAX))),
                "stop" => Ok(Value::Builtin(Builtin::SimStop)),
                _ => Err(Fault::new("unknownAttribute", format!("simulation has no `{name}`"))),
            },
            Value::Builtin(m) => {
                builtins::module_member(*m, name).ok_or_else(|| Fault::new("unknownAttribute", format!("there is no `{name}` here")))
            }
            Value::List(_) | Value::Dict(_) | Value::Str(_) | Value::Tuple(_) => {
                if builtins::has_method(base, name) {
                    Ok(Value::Method(Arc::new((base.clone(), Arc::from(name)))))
                } else {
                    Err(builtins::no_method(base, name))
                }
            }
            Value::None => {
                Err(Fault::new("noneReference", format!("`.{name}` of None: the value is missing (an empty place, no pallet?)")))
            }
            other => Err(builtins::no_method(other, name)),
        }
    }

    fn set_attr(&mut self, base: &Value, name: &str, value: Value, host: &mut dyn Host) -> Result<(), Fault> {
        match base {
            Value::Object(o) => {
                if let Some(b) = self.block_of(*o) {
                    let block = &mut self.blocks[b as usize];
                    let Some(g) = block.program.global(name) else {
                        return Err(Fault::new("unknownAttribute", format!("code block `{}` has no variable `{name}`", block.name)));
                    };
                    if let Some(slot) = block.globals.get_mut(g as usize) {
                        *slot = value;
                    }
                    return Ok(());
                }
                host.set(*o, name, &value)
            }
            Value::Pallet(id) => {
                if PALLET_MEMBERS.iter().any(|(m, _)| *m == name) {
                    return Err(Fault::new("notWritable", format!("`pallet.{name}` is the pallet's own data and can't be changed")));
                }
                let attrs = self.pallet_attrs.entry(*id).or_default();
                if attrs.len() >= 1000 && !attrs.contains(&Value::str(name)).unwrap_or(false) {
                    return Err(Fault::new("tooLarge", "a pallet holds at most 1000 attributes"));
                }
                attrs.set(Value::str(name), value).map_err(Fault::types)
            }
            Value::None => Err(Fault::new("noneReference", format!("`.{name} = …` on None: the value is missing"))),
            other => Err(Fault::types(format!("a {} has no attributes to set", other.type_name()))),
        }
    }
}

fn matches_event(spec: &EventSpec, e: &Event) -> bool {
    match (spec, e) {
        (EventSpec::Create(o), Event::Create { source, .. }) => o == source,
        (EventSpec::Enter(o), Event::Enter { object, .. }) => o == object,
        (EventSpec::Leave(o), Event::Leave { object, .. }) => o == object,
        (EventSpec::Signal { object, signal, edge }, Event::Signal { object: o, signal: s, value }) => {
            object == o && signal == s && (*edge == Edge::Change || (*edge == Edge::Rise) == *value)
        }
        _ => false,
    }
}

/// Binds a call's arguments to a function's parameters, Python's way.
fn bind(f: &Function, args: Vec<Value>, kwargs: Vec<(Arc<str>, Value)>) -> Result<Vec<Value>, Fault> {
    let n = f.params.len();
    if args.len() > n {
        return Err(Fault::new("wrongArguments", format!("{}() takes {n} arguments, not {}", f.name, args.len())));
    }
    let mut locals = vec![Value::Unbound; (f.nlocals as usize).max(n)];
    let mut given = vec![false; n];
    for (i, a) in args.into_iter().enumerate() {
        locals[i] = a;
        given[i] = true;
    }
    for (k, v) in kwargs {
        let Some(i) = f.params.iter().position(|p| p.as_str() == k.as_ref()) else {
            return Err(Fault::new("wrongArguments", format!("{}() has no parameter `{k}`", f.name)));
        };
        if given[i] {
            return Err(Fault::new("wrongArguments", format!("{}() got `{k}` twice", f.name)));
        }
        locals[i] = v;
        given[i] = true;
    }
    let first_default = n - f.defaults.len();
    for i in 0..n {
        if !given[i] {
            match i.checked_sub(first_default).and_then(|d| f.defaults.get(d)) {
                Some(d) => locals[i] = d.clone(),
                None => return Err(Fault::new("wrongArguments", format!("{}() is missing its argument `{}`", f.name, f.params[i]))),
            }
        }
    }
    Ok(locals)
}

fn sort_options(kwargs: &[(Arc<str>, Value)]) -> Result<(Option<Value>, bool), Fault> {
    let mut key = None;
    let mut reverse = false;
    for (k, v) in kwargs {
        match k.as_ref() {
            "key" => key = Some(v.clone()),
            "reverse" => reverse = v.truthy(),
            other => return Err(Fault::new("wrongArguments", format!("takes no argument `{other}`"))),
        }
    }
    Ok((key, reverse))
}

/// A wait's length in µs: a time, or a plain number of seconds; never negative.
fn duration(v: &Value) -> Result<i64, Fault> {
    let s = match v {
        Value::Qty(x, d) if *d == Dim::TIME => *x,
        Value::Qty(_, d) => return Err(Fault::new("unitMismatch", format!("a wait takes a time, not a {}", d.name()))),
        other => other.as_f64().ok_or_else(|| Fault::types(format!("a wait takes a time, not a {}", other.type_name())))?,
    };
    if s.is_nan() || s < 0.0 {
        return Err(Fault::new("negativeWait", "a wait can't be negative"));
    }
    Ok((s.min(9.0e12) * 1e6).round() as i64)
}

fn slice_part(v: Value) -> Result<Option<i64>, Fault> {
    match v {
        Value::None => Ok(None),
        Value::Int(i) => Ok(Some(i)),
        Value::Bool(b) => Ok(Some(i64::from(b))),
        other => Err(Fault::types(format!("a slice takes whole numbers, not a {}", other.type_name()))),
    }
}

fn name_of(p: &Program, n: u32) -> Arc<str> {
    p.names.get(n as usize).cloned().unwrap_or_else(|| Arc::from("?"))
}

fn pop(act: &mut Activation) -> Result<Value, Fault> {
    act.stack.pop().ok_or_else(stack_fault)
}

fn pop_n(act: &mut Activation, n: usize) -> Result<Vec<Value>, Fault> {
    let at = act.stack.len().checked_sub(n).ok_or_else(stack_fault)?;
    Ok(act.stack.split_off(at))
}

fn stack_fault() -> Fault {
    Fault::new("internal", "the interpreter's stack is out of step")
}

/// FNV-1a, for a block's random stream from its id (stable across builds, unlike the standard hasher).
fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3))
}
