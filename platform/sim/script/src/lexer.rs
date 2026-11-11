//! Splits code into tokens, as Python does: indentation opens and closes blocks, a line ends a statement unless a
//! bracket is open or it ends with `\`, `#` starts a comment, and texts may be `'…'`, `"…"`, triple-quoted or f-strings.

use crate::diag::Diagnostic;

/// The longest code a block may hold, in characters (S21).
pub const MAX_SOURCE: usize = 200_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Name(String),
    Int(i64),
    Float(f64),
    Str(String),
    /// An f-string: its literal parts and the expressions between braces.
    FStr(Vec<FPiece>),
    Op(&'static str),
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FPiece {
    Lit(String),
    /// The code between the braces, where it starts, and the text after `:` (`.2f`).
    Expr {
        code: String,
        line: u32,
        col: u32,
        spec: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub col: u32,
}

/// Operators and punctuation, longest first.
const OPS: [&str; 38] = [
    "**=", "//=", "...", "->", "**", "//", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "<>", ":=", "+", "-", "*", "/", "%", "<",
    ">", "=", "(", ")", "[", "]", "{", "}", ",", ":", ".", "@", ";", "!", "&",
];

pub fn tokenize(src: &str) -> Result<Vec<Token>, Diagnostic> {
    if src.chars().count() > MAX_SOURCE {
        return Err(Diagnostic::error("tooLarge", 1, 1, format!("the code is longer than {MAX_SOURCE} characters")));
    }
    Lexer { chars: src.chars().collect(), pos: 0, line: 1, col: 1, out: Vec::new(), indents: vec![0], depth: 0 }.run()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
    out: Vec<Token>,
    indents: Vec<usize>,
    /// Open brackets: newlines inside them don't end the statement.
    depth: usize,
}

impl Lexer {
    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.pos + k).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn push(&mut self, tok: Tok, line: u32, col: u32) {
        self.out.push(Token { tok, line, col });
    }

    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::error("syntax", self.line, self.col, msg)
    }

    fn run(mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut at_line_start = true;
        loop {
            if at_line_start && self.depth == 0 {
                // A blank or comment line leaves the next one at a line start too.
                if self.indentation()? {
                    continue;
                }
                at_line_start = false;
            }
            let Some(c) = self.peek(0) else { break };
            let (line, col) = (self.line, self.col);
            match c {
                '\n' => {
                    self.bump();
                    if self.depth == 0 {
                        self.push(Tok::Newline, line, col);
                        at_line_start = true;
                    }
                }
                ' ' | '\r' => {
                    self.bump();
                }
                '\t' => {
                    if self.depth == 0 && self.out.last().is_some_and(|t| t.tok == Tok::Newline) {
                        return Err(self.err("indent with spaces, not tabs"));
                    }
                    self.bump();
                }
                '#' => {
                    while self.peek(0).is_some_and(|c| c != '\n') {
                        self.bump();
                    }
                }
                '\\' if self.peek(1) == Some('\n') || (self.peek(1) == Some('\r') && self.peek(2) == Some('\n')) => {
                    self.bump();
                    if self.peek(0) == Some('\r') {
                        self.bump();
                    }
                    self.bump();
                }
                c if c.is_ascii_digit() || (c == '.' && self.peek(1).is_some_and(|d| d.is_ascii_digit())) => {
                    let t = self.number()?;
                    self.push(t, line, col);
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut name = String::new();
                    while let Some(c) = self.peek(0).filter(|c| c.is_alphanumeric() || *c == '_') {
                        name.push(c);
                        self.bump();
                    }
                    let prefix = name.to_ascii_lowercase();
                    if matches!(prefix.as_str(), "f" | "r" | "rf" | "fr" | "b" | "u") && matches!(self.peek(0), Some('"' | '\'')) {
                        if prefix == "b" {
                            return Err(self.err("bytes are not part of IntraScript; write a normal text"));
                        }
                        let t = self.string(prefix.contains('f'), prefix.contains('r'))?;
                        self.push(t, line, col);
                    } else {
                        self.push(Tok::Name(name), line, col);
                    }
                }
                '"' | '\'' => {
                    let t = self.string(false, false)?;
                    self.push(t, line, col);
                }
                _ => {
                    let Some(op) = OPS.iter().copied().find(|op| op.chars().enumerate().all(|(i, oc)| self.peek(i) == Some(oc))) else {
                        return Err(self.err(format!("unexpected character {c:?}")));
                    };
                    for _ in 0..op.chars().count() {
                        self.bump();
                    }
                    match op {
                        "(" | "[" | "{" => self.depth += 1,
                        ")" | "]" | "}" => {
                            if self.depth == 0 {
                                return Err(Diagnostic::error("syntax", line, col, format!("{op} closes nothing")));
                            }
                            self.depth -= 1;
                        }
                        _ => {}
                    }
                    self.push(Tok::Op(op), line, col);
                }
            }
        }
        let (line, col) = (self.line, self.col);
        if self.depth > 0 {
            return Err(self.err("a bracket is still open at the end of the code"));
        }
        if self.out.last().is_some_and(|t| t.tok != Tok::Newline) {
            self.push(Tok::Newline, line, col);
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push(Tok::Dedent, line, col);
        }
        self.push(Tok::Eof, line, col);
        Ok(self.out)
    }

    /// Reads a line's indentation and emits Indent or Dedents; true when the line is blank or a comment (skipped).
    fn indentation(&mut self) -> Result<bool, Diagnostic> {
        let mut width = 0;
        loop {
            match self.peek(0) {
                Some(' ') => width += 1,
                Some('\t') => return Err(self.err("indent with spaces, not tabs")),
                Some('\r') => {}
                _ => break,
            }
            self.bump();
        }
        match self.peek(0) {
            None => return Ok(false),
            Some('\n') => {
                self.bump();
                return Ok(true);
            }
            Some('#') => {
                while self.peek(0).is_some_and(|c| c != '\n') {
                    self.bump();
                }
                if self.peek(0) == Some('\n') {
                    self.bump();
                }
                return Ok(true);
            }
            _ => {}
        }
        let current = self.indents.last().copied().unwrap_or(0);
        let (line, col) = (self.line, self.col);
        if width > current {
            self.indents.push(width);
            self.push(Tok::Indent, line, col);
        } else {
            while width < self.indents.last().copied().unwrap_or(0) {
                self.indents.pop();
                self.push(Tok::Dedent, line, col);
            }
            if width != self.indents.last().copied().unwrap_or(0) {
                return Err(Diagnostic::error("syntax", line, col, "this line's indentation matches no block above it"));
            }
        }
        Ok(false)
    }

    fn number(&mut self) -> Result<Tok, Diagnostic> {
        if self.peek(0) == Some('0') && matches!(self.peek(1), Some('x' | 'X' | 'o' | 'O' | 'b' | 'B')) {
            self.bump();
            let radix = match self.bump() {
                Some('x' | 'X') => 16,
                Some('o' | 'O') => 8,
                _ => 2,
            };
            let mut digits = String::new();
            while let Some(c) = self.peek(0).filter(|c| c.is_ascii_alphanumeric() || *c == '_') {
                if c != '_' {
                    digits.push(c);
                }
                self.bump();
            }
            return i64::from_str_radix(&digits, radix).map(Tok::Int).map_err(|_| self.err("not a valid whole number"));
        }
        let mut text = String::new();
        let mut float = false;
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    text.push(c);
                }
            } else if c == '.' && !float && self.peek(1) != Some('.') {
                float = true;
                text.push(c);
            } else if (c == 'e' || c == 'E')
                && (self.peek(1).is_some_and(|d| d.is_ascii_digit())
                    || (matches!(self.peek(1), Some('+' | '-')) && self.peek(2).is_some_and(|d| d.is_ascii_digit())))
            {
                float = true;
                text.push('e');
                self.bump();
                if let Some(s) = self.peek(0).filter(|s| *s == '+' || *s == '-') {
                    text.push(s);
                    self.bump();
                }
                continue;
            } else {
                break;
            }
            self.bump();
        }
        if self.peek(0).is_some_and(|c| c.is_alphabetic()) {
            return Err(self.err("a number can't run into a name: write `150 * mm`, with `*`, for a quantity"));
        }
        if float {
            text.parse::<f64>().map(Tok::Float).map_err(|_| self.err("not a valid number"))
        } else {
            text.parse::<i64>().map(Tok::Int).map_err(|_| self.err("this whole number is too large"))
        }
    }

    fn string(&mut self, f: bool, raw: bool) -> Result<Tok, Diagnostic> {
        let (line, col) = (self.line, self.col);
        let Some(q) = self.bump() else { return Err(self.err("text without quotes")) };
        let triple = self.peek(0) == Some(q) && self.peek(1) == Some(q);
        if triple {
            self.bump();
            self.bump();
        }
        let mut pieces: Vec<FPiece> = Vec::new();
        let mut lit = String::new();
        loop {
            let Some(c) = self.peek(0) else {
                return Err(Diagnostic::error("syntax", line, col, "this text has no closing quote"));
            };
            if c == q && (!triple || (self.peek(1) == Some(q) && self.peek(2) == Some(q))) {
                self.bump();
                if triple {
                    self.bump();
                    self.bump();
                }
                break;
            }
            if c == '\n' && !triple {
                return Err(Diagnostic::error("syntax", line, col, "this text has no closing quote on its line"));
            }
            if c == '\\' && !raw {
                self.bump();
                let e = self.bump().ok_or_else(|| self.err("text ends in \\"))?;
                match e {
                    'n' => lit.push('\n'),
                    't' => lit.push('\t'),
                    'r' => lit.push('\r'),
                    '0' => lit.push('\0'),
                    '\\' => lit.push('\\'),
                    '\'' => lit.push('\''),
                    '"' => lit.push('"'),
                    '\n' => {}
                    'x' | 'u' => {
                        let n = if e == 'x' { 2 } else { 4 };
                        let mut hex = String::new();
                        for _ in 0..n {
                            hex.push(self.bump().ok_or_else(|| self.err("an escape is cut short"))?);
                        }
                        let ch =
                            u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32).ok_or_else(|| self.err("not a valid escape"))?;
                        lit.push(ch);
                    }
                    other => {
                        lit.push('\\');
                        lit.push(other);
                    }
                }
                continue;
            }
            if f && c == '{' {
                self.bump();
                if self.peek(0) == Some('{') {
                    self.bump();
                    lit.push('{');
                    continue;
                }
                if !lit.is_empty() {
                    pieces.push(FPiece::Lit(std::mem::take(&mut lit)));
                }
                pieces.push(self.f_expr()?);
                continue;
            }
            if f && c == '}' {
                self.bump();
                if self.peek(0) == Some('}') {
                    self.bump();
                }
                lit.push('}');
                continue;
            }
            self.bump();
            lit.push(c);
        }
        if !f {
            return Ok(Tok::Str(lit));
        }
        if !lit.is_empty() {
            pieces.push(FPiece::Lit(lit));
        }
        Ok(Tok::FStr(pieces))
    }

    /// The expression of an f-string up to its closing brace, with the format after `:` (outside brackets).
    fn f_expr(&mut self) -> Result<FPiece, Diagnostic> {
        let (line, col) = (self.line, self.col);
        let mut code = String::new();
        let mut depth = 0usize;
        let mut quote: Option<char> = None;
        loop {
            let c = self.peek(0).ok_or_else(|| Diagnostic::error("syntax", line, col, "an f-string's { is never closed"))?;
            if c == '\n' {
                return Err(Diagnostic::error("syntax", line, col, "an f-string's { is never closed on its line"));
            }
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                }
                code.push(c);
                self.bump();
                continue;
            }
            match c {
                '\'' | '"' => quote = Some(c),
                '(' | '[' | '{' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                '}' if depth > 0 => depth -= 1,
                '}' => {
                    self.bump();
                    return Ok(FPiece::Expr { code, line, col, spec: None });
                }
                ':' if depth == 0 => {
                    self.bump();
                    let mut spec = String::new();
                    while let Some(s) = self.peek(0).filter(|s| *s != '}' && *s != '\n') {
                        spec.push(s);
                        self.bump();
                    }
                    if self.bump() != Some('}') {
                        return Err(Diagnostic::error("syntax", line, col, "an f-string's { is never closed"));
                    }
                    return Ok(FPiece::Expr { code, line, col, spec: Some(spec) });
                }
                _ => {}
            }
            code.push(c);
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn indentation_opens_and_closes_blocks() {
        let t = toks("if x:\n    y = 1\nz = 2\n");
        assert!(t.contains(&Tok::Indent));
        assert!(t.contains(&Tok::Dedent));
        assert_eq!(t.iter().filter(|t| **t == Tok::Newline).count(), 3);
    }

    #[test]
    fn blank_lines_comments_and_brackets_dont_end_statements() {
        let t = toks("x = [1,\n  2]  # two\n\n# only a comment\ny = 3\n");
        assert_eq!(t.iter().filter(|t| **t == Tok::Newline).count(), 2);
    }

    #[test]
    fn reads_numbers_texts_and_f_strings() {
        assert_eq!(toks("1_000 0.25 1e3 0x1F")[..4], [Tok::Int(1000), Tok::Float(0.25), Tok::Float(1000.0), Tok::Int(31)]);
        assert_eq!(toks("'a\\n' \"b\"")[..2], [Tok::Str("a\n".into()), Tok::Str("b".into())]);
        let Tok::FStr(p) = &toks("f\"n = {n:.2f} {{x}}\"")[0] else { panic!() };
        assert_eq!(p.len(), 3);
        assert!(matches!(&p[1], FPiece::Expr { code, spec: Some(s), .. } if code == "n" && s == ".2f"));
        assert_eq!(p[2], FPiece::Lit(" {x}".into()));
    }

    #[test]
    fn errors_say_where_and_what() {
        let e = tokenize("x = 'open\n").unwrap_err();
        assert_eq!((e.line, e.code.as_str()), (1, "syntax"));
        assert!(tokenize("x = 150mm\n").unwrap_err().message.contains("150 * mm"));
        assert!(tokenize("if x:\n\ty = 1\n").unwrap_err().message.contains("spaces"));
        assert!(tokenize("if x:\n    y = 1\n  z = 2\n").is_err());
    }
}
