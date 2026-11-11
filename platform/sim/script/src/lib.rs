//! **IntraScript**, IntraCraft's own programming language for simulation (A17 IntraScript Language, ADR-0018,
//! ADR-0027). Its code is written with Python's syntax, so it reads like Python and every Python editor colours it, and
//! it adds what a plant simulation needs:
//!
//! - **events as decorators**: `@on_enter(Roller3)`, `@on_rise(Photoeye3.detected)`, `@every(10 * minute)`,
//!   `@choose_exit(Turntable1)`;
//! - **waiting inside a sequence**: `wait(2 * s)`, `wait_until(Roller4.occupied, timeout=30 * s)`;
//! - **physical units**: `150 * mm`, `0.25 * m / s`; adding a length to a time is an error, not a wrong number;
//! - **the layout's objects by name**, with their signals, parameters and statistics, checked before the run.
//!
//! It is its own small language, not CPython: the interpreter is this crate, written in safe Rust with no dependencies
//! and no `eval`. Code can't reach files, the network or the clock, every random number comes from the block's seeded
//! stream, and limits stop a runaway loop instead of freezing the window, so a run stays deterministic and safe.
//!
//! Pipeline: [`lexer`] → [`parser`] → [`compile`] (checks, then instructions) → [`vm::Runtime`], which an engine
//! drives through [`host::Host`]. The engines are not known here: the 2D run implements `Host` in `intracraft-des`.

pub mod ast;
pub mod builtins;
pub mod compile;
pub mod complete;
pub mod diag;
pub mod host;
pub mod lexer;
pub mod ops;
pub mod parser;
pub mod units;
pub mod value;
pub mod vm;

pub use compile::{Source, check, compile_blocks};
pub use diag::{Diagnostic, Severity};
pub use host::{Host, Member, MemberKind, ModelInfo, ObjectInfo};
pub use ops::Fault;
pub use value::Value;
pub use vm::{ConsoleLine, Event, RunError, Runtime};

/// The library definition of a code block (S21).
pub const CODE_BLOCK: &str = "info.code";
/// The version of the language a block's code is written in; stored with the code so later versions can migrate it.
pub const LANGUAGE_VERSION: u32 = 1;

/// A name that code can use: a letter or `_` first, then letters, digits or `_`, and not a reserved word.
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
        && !parser::KEYWORDS.contains(&name)
}

/// The name code uses for an object of the layout (A17 §10): its own name when that is an identifier, otherwise the
/// name with every other character made `_` (`Roller conveyor 100023` → `Roller_conveyor_100023`).
pub fn code_name(name: &str) -> String {
    if is_identifier(name) {
        return name.to_string();
    }
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_alphanumeric() || c == '_' {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let out = out.trim_matches('_').to_string();
    match out.chars().next() {
        None => "_".into(),
        Some(c) if c.is_ascii_digit() => format!("_{out}"),
        _ if parser::KEYWORDS.contains(&out.as_str()) => format!("{out}_"),
        _ => out,
    }
}

/// A library key in Python's style: `nominalSpeed` → `nominal_speed`.
pub fn snake_case(key: &str) -> String {
    let mut out = String::new();
    for (i, c) in key.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests;
