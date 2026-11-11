//! What the checker and the run report about code: a code (`syntax`, `unknownName`…), where, and what to do about it.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// The stable code of A17 §13 (`syntax`, `unknownName`, `waitInDecision`…).
    pub code: String,
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn error(code: &str, line: u32, col: u32, message: impl Into<String>) -> Diagnostic {
        Diagnostic { severity: Severity::Error, code: code.into(), line, col, message: message.into() }
    }

    pub fn warning(code: &str, line: u32, col: u32, message: impl Into<String>) -> Diagnostic {
        Diagnostic { severity: Severity::Warning, code: code.into(), line, col, message: message.into() }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_error() { "error" } else { "warning" };
        write!(f, "line {}, column {}: {kind} [{}] {}", self.line, self.col, self.code, self.message)
    }
}
