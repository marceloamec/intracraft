//! Completion: the names that fit where the cursor is, for the code editor and for agents. After `Roller3.` it offers
//! Roller3's members; after `@` the events; after `math.`, `random.`, `simulation.` and `pallet.` their names; after a
//! section `Seccion1.` the objects in it; after another code block's name its functions and variables; anywhere else
//! the keywords, the block's own names, the layout's objects and sections, the built-ins and the units that start with
//! what is typed. It reads the code as it is while typing, so it never needs the code to parse.

use std::collections::BTreeSet;

use crate::builtins::{self, Builtin, DICT_METHODS, LIST_METHODS, STR_METHODS};
use crate::compile::DECORATORS;
use crate::host::{MemberKind, ModelInfo, PALLET_MEMBERS};
use crate::units::UNITS;

/// Most candidates returned.
pub const MAX_CANDIDATES: usize = 60;

/// Python's keywords the editor offers (the ones IntraScript has).
const KEYWORDS: [&str; 22] = [
    "def", "return", "if", "elif", "else", "for", "while", "in", "not", "and", "or", "is", "global", "pass", "break", "continue", "import",
    "from", "assert", "True", "False", "None",
];

/// What kind of name a candidate is, for its icon and its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A variable or function of this block, or a local.
    Own,
    Object,
    Section,
    Signal,
    Member,
    Event,
    Function,
    Builtin,
    Unit,
    Keyword,
    Method,
}

impl Kind {
    /// A short word for the list.
    pub fn tag(self) -> &'static str {
        match self {
            Kind::Own => "block",
            Kind::Object => "object",
            Kind::Section => "section",
            Kind::Signal => "signal",
            Kind::Member => "member",
            Kind::Event => "event",
            Kind::Function => "function",
            Kind::Builtin => "built-in",
            Kind::Unit => "unit",
            Kind::Keyword => "keyword",
            Kind::Method => "method",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// What the list shows and what is typed in place of the word.
    pub label: String,
    pub kind: Kind,
    /// One line of help.
    pub help: String,
}

/// The candidates for the cursor, and where the word they replace starts (a char index).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Completion {
    pub start: usize,
    pub items: Vec<Candidate>,
}

/// The names of another code block code can reach through its object: its object index and its code.
#[derive(Clone, Copy, Debug)]
pub struct Block<'a> {
    pub object: u32,
    pub code: &'a str,
}

/// The completion at `cursor` (a char index into `code`), for a block of `model`; `others` are the other code blocks.
pub fn complete(code: &str, cursor: usize, model: &ModelInfo, others: &[Block]) -> Completion {
    let chars: Vec<char> = code.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut start = cursor;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let prefix: String = chars[start..cursor].iter().collect();
    if in_comment_or_text(&chars[..start]) {
        return Completion { start, items: Vec::new() };
    }
    let before = if start > 0 { Some(chars[start - 1]) } else { None };
    let mut items = match before {
        Some('@') => DECORATORS.iter().map(|(n, h)| cand(n, Kind::Event, h)).collect(),
        Some('.') => after_dot(&path_before(&chars, start - 1), model, others),
        _ => anywhere(code, model),
    };
    let lower = prefix.to_lowercase();
    items.retain(|c| c.label.to_lowercase().starts_with(&lower));
    // The ones that start with exactly what is typed first, then by kind, then by name.
    items.sort_by(|a, b| {
        (!a.label.starts_with(&prefix), a.kind, a.label.to_lowercase()).cmp(&(
            !b.label.starts_with(&prefix),
            b.kind,
            b.label.to_lowercase(),
        ))
    });
    items.dedup_by(|a, b| a.label == b.label);
    items.truncate(MAX_CANDIDATES);
    Completion { start, items }
}

/// The names a block defines at its top (`name = …`, `def name(`), read line by line, so code that doesn't parse yet
/// still offers them.
pub fn top_level_names(code: &str) -> Vec<(String, Kind)> {
    let mut out: Vec<(String, Kind)> = Vec::new();
    for line in code.lines() {
        if line.starts_with([' ', '\t', '#', '@']) {
            continue;
        }
        let (name, kind) = if let Some(rest) = line.strip_prefix("def ") {
            (rest.chars().take_while(|c| is_word(*c)).collect::<String>(), Kind::Function)
        } else {
            let name: String = line.chars().take_while(|c| is_word(*c)).collect();
            let rest = line[name.len()..].trim_start();
            let assigns = rest.starts_with('=') && !rest.starts_with("==") || rest.starts_with(':') || rest.starts_with("+=");
            if !assigns {
                continue;
            }
            (name, Kind::Own)
        };
        if !name.is_empty() && !name.chars().next().is_some_and(|c| c.is_ascii_digit()) && !out.iter().any(|(n, _)| *n == name) {
            out.push((name, kind));
        }
    }
    out
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn cand(label: &str, kind: Kind, help: &str) -> Candidate {
    Candidate { label: label.to_string(), kind, help: help.to_string() }
}

/// Whether the cursor's line, up to it, is inside a comment or a text (no names are offered there).
fn in_comment_or_text(before: &[char]) -> bool {
    let line_start = before.iter().rposition(|c| *c == '\n').map_or(0, |i| i + 1);
    let mut quote: Option<char> = None;
    for &c in &before[line_start..] {
        match (quote, c) {
            (None, '#') => return true,
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), c) if c == q => quote = None,
            _ => {}
        }
    }
    quote.is_some()
}

/// The dotted names just before the dot at `dot`: `Seccion1.Roller3` in `Seccion1.Roller3.`.
fn path_before(chars: &[char], dot: usize) -> String {
    let mut i = dot;
    while i > 0 && (is_word(chars[i - 1]) || chars[i - 1] == '.') {
        i -= 1;
    }
    chars[i..dot].iter().collect()
}

fn after_dot(path: &str, model: &ModelInfo, others: &[Block]) -> Vec<Candidate> {
    match path {
        "math" | "random" | "simulation" => {
            let m = builtins::global(path).unwrap_or(Builtin::Math);
            return builtins::module_names(m).into_iter().map(|n| cand(n, Kind::Function, &format!("{path}.{n}"))).collect();
        }
        "pallet" => return PALLET_MEMBERS.iter().map(|(n, h)| cand(n, Kind::Member, h)).collect(),
        _ => {}
    }
    let object = model.objects.iter().enumerate().find(|(_, o)| o.name == path || (!o.address.is_empty() && o.address == path));
    if let Some((i, o)) = object {
        if o.is_code_block() {
            let code = others.iter().find(|b| b.object as usize == i).map_or("", |b| b.code);
            return top_level_names(code).into_iter().map(|(n, k)| cand(&n, k, &format!("of code block {}", o.name))).collect();
        }
        return o
            .members
            .iter()
            .map(|m| {
                let (kind, tag) = match m.kind {
                    MemberKind::Signal { writable: true } => (Kind::Signal, "signal, read and write"),
                    MemberKind::Signal { writable: false } => (Kind::Signal, "signal, read only"),
                    MemberKind::Value => (Kind::Member, "read only"),
                    MemberKind::Method => (Kind::Method, "call it"),
                };
                cand(&m.name, kind, &format!("{} ({tag})", m.help))
            })
            .collect();
    }
    // A section: the objects whose address starts with it.
    let section_prefix = format!("{path}.");
    let in_section: Vec<Candidate> = model
        .objects
        .iter()
        .filter_map(|o| o.address.strip_prefix(&section_prefix).filter(|rest| !rest.contains('.')).map(|rest| (rest, o)))
        .map(|(rest, o)| cand(rest, Kind::Object, &format!("{} in section {path}", o.kind)))
        .collect();
    if !in_section.is_empty() {
        return in_section;
    }
    // A value whose type the editor can't know: the methods of lists, dictionaries and texts.
    let mut methods: BTreeSet<&str> = BTreeSet::new();
    methods.extend(LIST_METHODS);
    methods.extend(DICT_METHODS);
    methods.extend(STR_METHODS);
    methods.into_iter().map(|m| cand(m, Kind::Method, "a method of lists, dictionaries or texts")).collect()
}

fn anywhere(code: &str, model: &ModelInfo) -> Vec<Candidate> {
    let mut items: Vec<Candidate> = Vec::new();
    for (name, kind) in top_level_names(code) {
        items.push(cand(&name, kind, "defined in this block"));
    }
    items.extend(local_names(code).into_iter().map(|n| cand(&n, Kind::Own, "a name used in this block")));
    let mut sections: BTreeSet<String> = BTreeSet::new();
    for o in &model.objects {
        items.push(cand(&o.name, Kind::Object, &o.kind));
        if let Some((section, _)) = o.address.split_once('.') {
            sections.insert(section.to_string());
        }
    }
    items.extend(sections.into_iter().map(|s| cand(&s, Kind::Section, "a section: write a dot for its objects")));
    items.extend(builtins::GLOBALS.iter().map(|(n, _, h)| cand(n, Kind::Builtin, h)));
    items.push(cand("wait", Kind::Builtin, "wait(2 * s): this handler pauses; the plant goes on"));
    items.push(cand("wait_until", Kind::Builtin, "wait_until(condition, timeout=30 * s): True when it held, False at the timeout"));
    items.extend(UNITS.iter().map(|(n, ..)| cand(n, Kind::Unit, "a unit: 2 * s, 150 * mm")));
    items.extend(KEYWORDS.iter().map(|k| cand(k, Kind::Keyword, "keyword")));
    items
}

/// Names assigned or taken as parameters anywhere in the code (locals of functions), read line by line.
fn local_names(code: &str) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for line in code.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("def ")
            && let Some((_, params)) = rest.split_once('(')
        {
            for p in params.split([',', ')']) {
                let name: String = p.trim().chars().take_while(|c| is_word(*c)).collect();
                out.insert(name);
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("for ") {
            for p in rest.split(" in ").next().unwrap_or("").split(',') {
                out.insert(p.trim().chars().take_while(|c| is_word(*c)).collect());
            }
            continue;
        }
        let name: String = t.chars().take_while(|c| is_word(*c)).collect();
        let rest = t[name.len()..].trim_start();
        if (rest.starts_with('=') && !rest.starts_with("==")) || rest.starts_with("+=") || rest.starts_with("-=") {
            out.insert(name);
        }
    }
    out.into_iter().filter(|n| !n.is_empty() && !n.chars().next().is_some_and(|c| c.is_ascii_digit())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{Member, ObjectInfo};

    fn model() -> ModelInfo {
        let signal = |n: &str, w: bool| Member { name: n.into(), kind: MemberKind::Signal { writable: w }, help: "a signal".into() };
        let value = |n: &str| Member { name: n.into(), kind: MemberKind::Value, help: "a value".into() };
        ModelInfo {
            objects: vec![
                ObjectInfo {
                    name: "Roller3".into(),
                    id: "r3".into(),
                    kind: "conveyor.roller".into(),
                    members: vec![signal("run", true), signal("occupied", false), value("length")],
                    address: "Seccion1.Roller3".into(),
                    ..Default::default()
                },
                ObjectInfo {
                    name: "Routing".into(),
                    id: "c1".into(),
                    kind: crate::CODE_BLOCK.into(),
                    address: "Routing".into(),
                    ..Default::default()
                },
            ],
        }
    }

    fn labels(c: &Completion) -> Vec<&str> {
        c.items.iter().map(|i| i.label.as_str()).collect()
    }

    fn at_end(code: &str) -> Completion {
        complete(code, code.chars().count(), &model(), &[Block { object: 1, code: "routed = 0\ndef pick(kind):\n    return None\n" }])
    }

    #[test]
    fn after_an_object_and_a_dot_come_its_members() {
        let c = at_end("Roller3.");
        assert_eq!(labels(&c), ["occupied", "run", "length"]);
        assert_eq!(c.items[1].kind, Kind::Signal);
        assert!(c.items[1].help.contains("read and write"));
        assert_eq!(labels(&at_end("Roller3.r")), ["run"]);
    }

    #[test]
    fn a_section_offers_its_objects_and_an_address_its_members() {
        assert_eq!(labels(&at_end("Seccion1.")), ["Roller3"]);
        assert_eq!(labels(&at_end("Seccion1.Roller3.le")), ["length"]);
    }

    #[test]
    fn events_modules_pallets_and_other_blocks() {
        assert!(labels(&at_end("@on_e")).contains(&"on_enter"));
        assert!(labels(&at_end("x = math.sq")).contains(&"sqrt"));
        assert!(labels(&at_end("    print(pallet.add")).contains(&"address"));
        assert_eq!(labels(&at_end("Routing.")), ["routed", "pick"]);
    }

    #[test]
    fn anywhere_the_names_that_start_with_what_is_typed() {
        let code = "counted = 0\n\ndef helper():\n    pass\n\n@on_start\ndef go():\n    hel";
        let c = complete(code, code.chars().count(), &model(), &[]);
        assert_eq!(labels(&c), ["helper"]);
        assert_eq!(c.start, code.chars().count() - 3);
        let c = at_end("x = 2 * mi");
        assert!(labels(&c).contains(&"minute") && labels(&c).contains(&"min"), "{:?}", labels(&c));
        assert!(labels(&at_end("Se")).contains(&"Seccion1"));
    }

    #[test]
    fn nothing_inside_comments_and_texts() {
        assert!(at_end("# Roller3.").items.is_empty());
        assert!(at_end("print(\"Roller3.").items.is_empty());
    }

    #[test]
    fn top_level_names_are_read_without_parsing() {
        let names = top_level_names("a = 1\nb: int = 2\ndef f(x):\n    c = 3\nif x\n@on_start\ndef g(\n");
        assert_eq!(names, [("a".into(), Kind::Own), ("b".into(), Kind::Own), ("f".into(), Kind::Function), ("g".into(), Kind::Function)]);
    }
}
