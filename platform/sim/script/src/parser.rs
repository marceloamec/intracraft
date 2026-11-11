//! Reads tokens into statements and expressions with Python's grammar and precedence. Python's features that
//! IntraScript leaves out (classes, lambdas, exceptions, generators…) are syntax errors that say so.

use crate::ast::{BinOp, CmpOp, Expr, ExprKind, FPart, FuncDef, Param, Stmt, StmtKind, UnOp};
use crate::diag::Diagnostic;
use crate::lexer::{FPiece, Tok, Token, tokenize};

/// Python's keywords: none of them can name a variable or function.
pub const KEYWORDS: [&str; 35] = [
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

/// How deep expressions and blocks may nest before the parser stops (hostile input can't overflow the stack).
const MAX_NESTING: usize = 100;

pub fn parse(src: &str) -> Result<Vec<Stmt>, Diagnostic> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0, nesting: 0 };
    let mut body = Vec::new();
    while !p.at(&Tok::Eof) {
        if p.eat(&Tok::Newline) {
            continue;
        }
        body.extend(p.statement()?);
    }
    Ok(body)
}

/// One expression on its own (an f-string's braces).
pub fn parse_expression(src: &str, line: u32, col: u32) -> Result<Expr, Diagnostic> {
    let mut tokens = tokenize(src).map_err(|e| Diagnostic::error("syntax", line, col, e.message))?;
    for t in &mut tokens {
        t.line = line;
        t.col = col;
    }
    let mut p = Parser { tokens, pos: 0, nesting: 0 };
    let e = p.expr_list()?;
    p.eat(&Tok::Newline);
    if !p.at(&Tok::Eof) {
        return Err(Diagnostic::error("syntax", line, col, "only one expression fits between an f-string's braces"));
    }
    Ok(e)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    nesting: usize,
}

impl Parser {
    fn tok(&self) -> &Tok {
        self.tokens.get(self.pos).map_or(&Tok::Eof, |t| &t.tok)
    }

    fn here(&self) -> (u32, u32) {
        self.tokens.get(self.pos).or(self.tokens.last()).map_or((1, 1), |t| (t.line, t.col))
    }

    fn at(&self, t: &Tok) -> bool {
        self.tok() == t
    }

    fn at_op(&self, op: &str) -> bool {
        matches!(self.tok(), Tok::Op(o) if *o == op)
    }

    fn at_kw(&self, kw: &str) -> bool {
        matches!(self.tok(), Tok::Name(n) if n == kw)
    }

    fn advance(&mut self) -> Tok {
        let t = self.tok().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.at(t) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_op(&mut self, op: &str) -> bool {
        if self.at_op(op) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.at_kw(kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        let (line, col) = self.here();
        Diagnostic::error("syntax", line, col, msg)
    }

    fn expect_op(&mut self, op: &str) -> Result<(), Diagnostic> {
        if self.eat_op(op) { Ok(()) } else { Err(self.err(format!("expected `{op}` here, found {}", describe(self.tok())))) }
    }

    fn name(&mut self) -> Result<String, Diagnostic> {
        match self.tok().clone() {
            Tok::Name(n) if KEYWORDS.contains(&n.as_str()) => Err(self.err(format!("`{n}` is a reserved word and can't be a name"))),
            Tok::Name(n) => {
                self.advance();
                Ok(n)
            }
            t => Err(self.err(format!("expected a name here, found {}", describe(&t)))),
        }
    }

    fn deeper(&mut self) -> Result<(), Diagnostic> {
        self.nesting += 1;
        if self.nesting > MAX_NESTING { Err(self.err("the code nests too deeply")) } else { Ok(()) }
    }

    // ---- statements ----

    /// A line of simple statements (`a = 1; b = 2`) or one compound statement.
    fn statement(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let (line, col) = self.here();
        if self.at_op("@") || self.at_kw("def") {
            return Ok(vec![self.def()?]);
        }
        if self.at_kw("if") {
            self.advance();
            return Ok(vec![self.if_rest(line, col)?]);
        }
        if self.eat_kw("while") {
            let cond = self.expr()?;
            let body = self.block()?;
            if self.at_kw("else") {
                return Err(self.err("`while … else` is not part of IntraScript"));
            }
            return Ok(vec![Stmt { kind: StmtKind::While { cond, body }, line, col }]);
        }
        if self.eat_kw("for") {
            let target = self.target_list()?;
            if !self.eat_kw("in") {
                return Err(self.err("expected `in` after the loop variable"));
            }
            let iter = self.expr_list()?;
            let body = self.block()?;
            if self.at_kw("else") {
                return Err(self.err("`for … else` is not part of IntraScript"));
            }
            return Ok(vec![Stmt { kind: StmtKind::For { target, iter, body }, line, col }]);
        }
        for (kw, why) in UNSUPPORTED {
            if self.at_kw(kw) {
                return Err(self.err(why));
            }
        }
        let mut out = vec![self.simple()?];
        while self.eat_op(";") {
            if self.at(&Tok::Newline) {
                break;
            }
            out.push(self.simple()?);
        }
        if !self.eat(&Tok::Newline) && !self.at(&Tok::Eof) && !self.at(&Tok::Dedent) {
            return Err(self.err(format!("expected the end of the line, found {}", describe(self.tok()))));
        }
        Ok(out)
    }

    fn if_rest(&mut self, line: u32, col: u32) -> Result<Stmt, Diagnostic> {
        let cond = self.expr()?;
        let body = self.block()?;
        let orelse = if self.at_kw("elif") {
            let (l, c) = self.here();
            self.advance();
            vec![self.if_rest(l, c)?]
        } else if self.eat_kw("else") {
            self.block()?
        } else {
            Vec::new()
        };
        Ok(Stmt { kind: StmtKind::If { cond, body, orelse }, line, col })
    }

    /// `:` and the indented block, or simple statements on the same line.
    fn block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.expect_op(":")?;
        self.deeper()?;
        let mut body = Vec::new();
        if self.eat(&Tok::Newline) {
            if !self.eat(&Tok::Indent) {
                return Err(self.err("expected an indented block after `:`"));
            }
            while !self.eat(&Tok::Dedent) && !self.at(&Tok::Eof) {
                if self.eat(&Tok::Newline) {
                    continue;
                }
                body.extend(self.statement()?);
            }
        } else {
            body = self.statement()?;
        }
        self.nesting -= 1;
        Ok(body)
    }

    fn def(&mut self) -> Result<Stmt, Diagnostic> {
        let mut decorators = Vec::new();
        while self.eat_op("@") {
            decorators.push(self.expr()?);
            if !self.eat(&Tok::Newline) {
                return Err(self.err("a decorator ends its line"));
            }
        }
        let (line, col) = self.here();
        if !self.eat_kw("def") {
            return Err(self.err("expected `def` after the decorators"));
        }
        let name = self.name()?;
        self.expect_op("(")?;
        let mut params: Vec<Param> = Vec::new();
        while !self.at_op(")") {
            if self.at_op("*") || self.at_op("**") {
                return Err(self.err("`*args` and `**kwargs` are not part of IntraScript"));
            }
            let pname = self.name()?;
            if self.eat_op(":") {
                self.expr()?;
            }
            let default = if self.eat_op("=") { Some(self.expr()?) } else { None };
            if default.is_none() && params.iter().any(|p| p.default.is_some()) {
                return Err(self.err("parameters without a default value come before those with one"));
            }
            if params.iter().any(|p| p.name == pname) {
                return Err(self.err(format!("parameter `{pname}` is named twice")));
            }
            params.push(Param { name: pname, default });
            if !self.eat_op(",") {
                break;
            }
        }
        self.expect_op(")")?;
        if self.eat_op("->") {
            self.expr()?;
        }
        let mut body = self.block()?;
        let doc = match body.first() {
            Some(Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Str(s), .. }), .. }) => Some(s.trim().to_string()),
            _ => None,
        };
        if doc.is_some() && body.len() > 1 {
            body.remove(0);
        }
        Ok(Stmt { kind: StmtKind::Def(Box::new(FuncDef { name, params, body, decorators, doc, line, col })), line, col })
    }

    fn simple(&mut self) -> Result<Stmt, Diagnostic> {
        let (line, col) = self.here();
        let s = |kind| Ok(Stmt { kind, line, col });
        if self.eat_kw("pass") {
            return s(StmtKind::Pass);
        }
        if self.eat_kw("break") {
            return s(StmtKind::Break);
        }
        if self.eat_kw("continue") {
            return s(StmtKind::Continue);
        }
        if self.eat_kw("return") {
            let v = if self.at(&Tok::Newline) || self.at_op(";") || self.at(&Tok::Eof) { None } else { Some(self.expr_list()?) };
            return s(StmtKind::Return(v));
        }
        if self.eat_kw("global") {
            let mut names = vec![self.name()?];
            while self.eat_op(",") {
                names.push(self.name()?);
            }
            return s(StmtKind::Global(names));
        }
        if self.eat_kw("assert") {
            let cond = self.expr()?;
            let msg = if self.eat_op(",") { Some(self.expr()?) } else { None };
            return s(StmtKind::Assert { cond, msg });
        }
        if self.eat_kw("import") {
            let module = self.dotted_name()?;
            if self.at_kw("as") {
                return Err(self.err("`import … as …` is not part of IntraScript"));
            }
            return s(StmtKind::Import { module });
        }
        if self.eat_kw("from") {
            let module = self.dotted_name()?;
            if !self.eat_kw("import") {
                return Err(self.err("expected `import` after `from …`"));
            }
            if !self.eat_op("*") {
                self.name()?;
                while self.eat_op(",") {
                    self.name()?;
                }
            }
            return s(StmtKind::Import { module });
        }
        let first = self.expr_list()?;
        if self.eat_op(":") {
            check_target(&first)?;
            self.expr()?;
            let value = if self.eat_op("=") { Some(self.expr_list()?) } else { None };
            return s(StmtKind::AnnAssign { target: first, value });
        }
        for (op, bin) in AUG_OPS {
            if self.eat_op(op) {
                check_target(&first)?;
                if matches!(first.kind, ExprKind::Tuple(_)) {
                    return Err(Diagnostic::error("syntax", line, col, format!("`{op}` takes one target")));
                }
                let value = self.expr_list()?;
                return s(StmtKind::AugAssign { target: first, op: bin, value });
            }
        }
        if self.at_op("=") {
            let mut targets = vec![first];
            let mut value = None;
            while self.eat_op("=") {
                let e = self.expr_list()?;
                if let Some(prev) = value.replace(e) {
                    targets.push(prev);
                }
            }
            for t in &targets {
                check_target(t)?;
            }
            let Some(value) = value else { return Err(self.err("expected a value after `=`")) };
            return s(StmtKind::Assign { targets, value });
        }
        if self.at_op(":=") {
            return Err(self.err("`:=` is not part of IntraScript; assign with `=` on its own line"));
        }
        s(StmtKind::Expr(first))
    }

    fn dotted_name(&mut self) -> Result<String, Diagnostic> {
        let mut n = self.name()?;
        while self.eat_op(".") {
            n.push('.');
            n.push_str(&self.name()?);
        }
        Ok(n)
    }

    /// A loop's targets: `x` or `i, x`.
    fn target_list(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let first = self.primary()?;
        if !self.at_op(",") {
            check_target(&first)?;
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.at_kw("in") {
                break;
            }
            items.push(self.primary()?);
        }
        let t = Expr::new(ExprKind::Tuple(items), line, col);
        check_target(&t)?;
        Ok(t)
    }

    // ---- expressions ----

    /// Expressions separated by commas make a tuple: `return a, b`.
    fn expr_list(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let first = self.expr()?;
        if !self.at_op(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.at(&Tok::Newline) || self.at_op("=") || self.at_op(")") || self.at_op(":") || self.at(&Tok::Eof) {
                break;
            }
            items.push(self.expr()?);
        }
        Ok(Expr::new(ExprKind::Tuple(items), line, col))
    }

    fn expr(&mut self) -> Result<Expr, Diagnostic> {
        self.deeper()?;
        let (line, col) = self.here();
        if self.at_kw("lambda") {
            return Err(self.err("`lambda` is not part of IntraScript; write a `def` function and use its name"));
        }
        let e = self.or_expr()?;
        let e = if self.eat_kw("if") {
            let cond = self.or_expr()?;
            if !self.eat_kw("else") {
                return Err(self.err("expected `else` in `a if condition else b`"));
            }
            let orelse = self.expr()?;
            Expr::new(ExprKind::IfExp { cond: Box::new(cond), then: Box::new(e), orelse: Box::new(orelse) }, line, col)
        } else {
            e
        };
        self.nesting -= 1;
        Ok(e)
    }

    fn or_expr(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.and_expr()?;
        while self.at_kw("or") {
            let (line, col) = self.here();
            self.advance();
            let r = self.and_expr()?;
            e = Expr::new(ExprKind::Or(Box::new(e), Box::new(r)), line, col);
        }
        Ok(e)
    }

    fn and_expr(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.not_expr()?;
        while self.at_kw("and") {
            let (line, col) = self.here();
            self.advance();
            let r = self.not_expr()?;
            e = Expr::new(ExprKind::And(Box::new(e), Box::new(r)), line, col);
        }
        Ok(e)
    }

    fn not_expr(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        if self.eat_kw("not") {
            self.deeper()?;
            let e = self.not_expr()?;
            self.nesting -= 1;
            return Ok(Expr::new(ExprKind::Unary(UnOp::Not, Box::new(e)), line, col));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let first = self.arith()?;
        let mut ops = Vec::new();
        loop {
            let op = if self.eat_op("==") {
                CmpOp::Eq
            } else if self.eat_op("!=") {
                CmpOp::Ne
            } else if self.at_op("<>") {
                return Err(self.err("write `!=` for \"not equal\""));
            } else if self.eat_op("<=") {
                CmpOp::Le
            } else if self.eat_op(">=") {
                CmpOp::Ge
            } else if self.eat_op("<") {
                CmpOp::Lt
            } else if self.eat_op(">") {
                CmpOp::Gt
            } else if self.eat_kw("in") {
                CmpOp::In
            } else if self.at_kw("not") && matches!(self.tokens.get(self.pos + 1).map(|t| &t.tok), Some(Tok::Name(n)) if n == "in") {
                self.advance();
                self.advance();
                CmpOp::NotIn
            } else if self.eat_kw("is") {
                if self.eat_kw("not") { CmpOp::IsNot } else { CmpOp::Is }
            } else {
                break;
            };
            ops.push((op, self.arith()?));
        }
        if ops.is_empty() { Ok(first) } else { Ok(Expr::new(ExprKind::Compare(Box::new(first), ops), line, col)) }
    }

    fn arith(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.term()?;
        loop {
            let (line, col) = self.here();
            let op = if self.eat_op("+") {
                BinOp::Add
            } else if self.eat_op("-") {
                BinOp::Sub
            } else {
                break;
            };
            let r = self.term()?;
            e = Expr::new(ExprKind::Binary(op, Box::new(e), Box::new(r)), line, col);
        }
        Ok(e)
    }

    fn term(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.factor()?;
        loop {
            let (line, col) = self.here();
            let op = if self.eat_op("*") {
                BinOp::Mul
            } else if self.eat_op("/") {
                BinOp::Div
            } else if self.eat_op("//") {
                BinOp::FloorDiv
            } else if self.eat_op("%") {
                BinOp::Mod
            } else {
                break;
            };
            let r = self.factor()?;
            e = Expr::new(ExprKind::Binary(op, Box::new(e), Box::new(r)), line, col);
        }
        Ok(e)
    }

    fn factor(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let op = if self.eat_op("-") {
            UnOp::Neg
        } else if self.eat_op("+") {
            UnOp::Pos
        } else {
            return self.power();
        };
        self.deeper()?;
        let e = self.factor()?;
        self.nesting -= 1;
        Ok(Expr::new(ExprKind::Unary(op, Box::new(e)), line, col))
    }

    fn power(&mut self) -> Result<Expr, Diagnostic> {
        let base = self.primary()?;
        let (line, col) = self.here();
        if self.eat_op("**") {
            self.deeper()?;
            // Right-associative, and binds tighter than a minus on its left: -2 ** 2 is -4.
            let exp = self.factor()?;
            self.nesting -= 1;
            return Ok(Expr::new(ExprKind::Binary(BinOp::Pow, Box::new(base), Box::new(exp)), line, col));
        }
        Ok(base)
    }

    /// An atom and what follows it: `.name`, `[index]`, `(arguments)`.
    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.atom()?;
        loop {
            let (line, col) = self.here();
            if self.eat_op(".") {
                let name = self.name()?;
                e = Expr::new(ExprKind::Attr(Box::new(e), name), line, col);
            } else if self.eat_op("[") {
                let index = self.subscript()?;
                self.expect_op("]")?;
                e = Expr::new(ExprKind::Index(Box::new(e), Box::new(index)), line, col);
            } else if self.eat_op("(") {
                let (args, kwargs) = self.arguments()?;
                e = Expr::new(ExprKind::Call { func: Box::new(e), args, kwargs }, line, col);
            } else {
                return Ok(e);
            }
        }
    }

    fn subscript(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let start = if self.at_op(":") { None } else { Some(self.expr()?) };
        if !self.eat_op(":") {
            let Some(s) = start else { return Err(self.err("expected an index")) };
            if self.at_op(",") {
                return Err(self.err("one index per bracket: write `table[row][column]`"));
            }
            return Ok(s);
        }
        let stop = if self.at_op("]") || self.at_op(":") { None } else { Some(Box::new(self.expr()?)) };
        let step = if self.eat_op(":") && !self.at_op("]") { Some(Box::new(self.expr()?)) } else { None };
        Ok(Expr::new(ExprKind::Slice(start.map(Box::new), stop, step), line, col))
    }

    fn arguments(&mut self) -> Result<Arguments, Diagnostic> {
        let mut args = Vec::new();
        let mut kwargs: Vec<(String, Expr)> = Vec::new();
        while !self.at_op(")") {
            if self.at_op("*") || self.at_op("**") {
                return Err(self.err("`*` and `**` in calls are not part of IntraScript"));
            }
            let is_kw = matches!(self.tok(), Tok::Name(_)) && matches!(self.tokens.get(self.pos + 1).map(|t| &t.tok), Some(Tok::Op("=")));
            if is_kw {
                let name = self.name()?;
                self.advance();
                if kwargs.iter().any(|(k, _)| *k == name) {
                    return Err(self.err(format!("argument `{name}` is given twice")));
                }
                kwargs.push((name, self.expr()?));
            } else {
                if !kwargs.is_empty() {
                    return Err(self.err("arguments by position come before arguments by name"));
                }
                args.push(self.expr()?);
            }
            if !self.eat_op(",") {
                break;
            }
        }
        self.expect_op(")")?;
        Ok((args, kwargs))
    }

    fn atom(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.here();
        let e = |kind| Ok(Expr::new(kind, line, col));
        match self.tok().clone() {
            Tok::Int(i) => {
                self.advance();
                e(ExprKind::Int(i))
            }
            Tok::Float(x) => {
                self.advance();
                e(ExprKind::Float(x))
            }
            Tok::Str(s) => {
                self.advance();
                let mut text = s;
                // Adjacent texts join, as in Python.
                while let Tok::Str(more) = self.tok().clone() {
                    self.advance();
                    text.push_str(&more);
                }
                e(ExprKind::Str(text))
            }
            Tok::FStr(pieces) => {
                self.advance();
                let mut parts = Vec::new();
                for p in pieces {
                    parts.push(match p {
                        FPiece::Lit(s) => FPart::Lit(s),
                        FPiece::Expr { code, line, col, spec } => FPart::Expr(parse_expression(&code, line, col)?, spec),
                    });
                }
                e(ExprKind::FStr(parts))
            }
            Tok::Name(n) => match n.as_str() {
                "True" => {
                    self.advance();
                    e(ExprKind::Bool(true))
                }
                "False" => {
                    self.advance();
                    e(ExprKind::Bool(false))
                }
                "None" => {
                    self.advance();
                    e(ExprKind::None)
                }
                _ => {
                    let name = self.name()?;
                    e(ExprKind::Name(name))
                }
            },
            Tok::Op("(") => {
                self.advance();
                if self.eat_op(")") {
                    return e(ExprKind::Tuple(Vec::new()));
                }
                self.deeper()?;
                let first = self.expr()?;
                let out = if self.at_op(",") {
                    let mut items = vec![first];
                    while self.eat_op(",") {
                        if self.at_op(")") {
                            break;
                        }
                        items.push(self.expr()?);
                    }
                    Expr::new(ExprKind::Tuple(items), line, col)
                } else {
                    first
                };
                self.expect_op(")")?;
                self.nesting -= 1;
                Ok(out)
            }
            Tok::Op("[") => {
                self.advance();
                self.deeper()?;
                let mut items = Vec::new();
                if !self.at_op("]") {
                    let first = self.expr()?;
                    if self.eat_kw("for") {
                        let target = self.target_list()?;
                        if !self.eat_kw("in") {
                            return Err(self.err("expected `in` in the list comprehension"));
                        }
                        let iter = self.or_expr()?;
                        let cond = if self.eat_kw("if") { Some(Box::new(self.or_expr()?)) } else { None };
                        if self.at_kw("for") {
                            return Err(self.err("one `for` per list comprehension; write a loop for more"));
                        }
                        self.expect_op("]")?;
                        self.nesting -= 1;
                        let kind = ExprKind::ListComp { elt: Box::new(first), target: Box::new(target), iter: Box::new(iter), cond };
                        return e(kind);
                    }
                    items.push(first);
                    while self.eat_op(",") {
                        if self.at_op("]") {
                            break;
                        }
                        items.push(self.expr()?);
                    }
                }
                self.expect_op("]")?;
                self.nesting -= 1;
                e(ExprKind::List(items))
            }
            Tok::Op("{") => {
                self.advance();
                self.deeper()?;
                let mut pairs = Vec::new();
                while !self.at_op("}") {
                    let k = self.expr()?;
                    if !self.eat_op(":") {
                        return Err(self.err("sets are not part of IntraScript; a dictionary needs `key: value`"));
                    }
                    let v = self.expr()?;
                    pairs.push((k, v));
                    if !self.eat_op(",") {
                        break;
                    }
                }
                self.expect_op("}")?;
                self.nesting -= 1;
                e(ExprKind::Dict(pairs))
            }
            t => Err(self.err(format!("expected a value here, found {}", describe(&t)))),
        }
    }
}

/// A call's arguments: by position, and by name.
type Arguments = (Vec<Expr>, Vec<(String, Expr)>);

const AUG_OPS: [(&str, BinOp); 7] = [
    ("+=", BinOp::Add),
    ("-=", BinOp::Sub),
    ("*=", BinOp::Mul),
    ("/=", BinOp::Div),
    ("//=", BinOp::FloorDiv),
    ("%=", BinOp::Mod),
    ("**=", BinOp::Pow),
];

const UNSUPPORTED: [(&str, &str); 12] = [
    ("class", "classes are not part of IntraScript 1.0; use dictionaries and functions"),
    ("try", "`try` is not part of IntraScript: a run error stops the run and shows where"),
    ("except", "`except` is not part of IntraScript"),
    ("finally", "`finally` is not part of IntraScript"),
    ("raise", "`raise` is not part of IntraScript; call `error(\"message\")` to stop the run"),
    ("with", "`with` is not part of IntraScript"),
    ("async", "`async` is not part of IntraScript; `wait()` already pauses a handler"),
    ("await", "`await` is not part of IntraScript; write `wait(2 * s)`"),
    ("yield", "`yield` is not part of IntraScript; write `wait(2 * s)`"),
    ("nonlocal", "`nonlocal` is not part of IntraScript: functions don't nest"),
    ("del", "`del` is not part of IntraScript; use `list.pop(i)` or `dict.pop(key)`"),
    ("else", "`else` without an `if`, `elif` or `for` above it"),
];

/// Only names, attributes, items and tuples of them can be assigned to.
fn check_target(e: &Expr) -> Result<(), Diagnostic> {
    match &e.kind {
        ExprKind::Name(_) | ExprKind::Attr(..) | ExprKind::Index(..) => Ok(()),
        ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().try_for_each(check_target),
        _ => Err(Diagnostic::error("syntax", e.line, e.col, "can't assign to this; only to a name, an attribute or an item")),
    }
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Name(n) => format!("`{n}`"),
        Tok::Int(i) => format!("the number {i}"),
        Tok::Float(x) => format!("the number {x}"),
        Tok::Str(_) | Tok::FStr(_) => "a text".into(),
        Tok::Op(o) => format!("`{o}`"),
        Tok::Newline => "the end of the line".into(),
        Tok::Indent => "an indented line".into(),
        Tok::Dedent => "the end of the block".into(),
        Tok::Eof => "the end of the code".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(src: &str) -> StmtKind {
        parse(src).unwrap().remove(0).kind
    }

    #[test]
    fn reads_python_statements() {
        let body =
            parse("x = 1\nif x > 0:\n    x += 1\nelif x == 0: x = 5\nelse:\n    pass\nfor i, v in enumerate(l):\n    print(i)\n").unwrap();
        assert_eq!(body.len(), 3);
        assert!(matches!(&body[1].kind, StmtKind::If { orelse, .. } if matches!(orelse[0].kind, StmtKind::If { .. })));
        assert!(matches!(&body[2].kind, StmtKind::For { target: Expr { kind: ExprKind::Tuple(t), .. }, .. } if t.len() == 2));
    }

    #[test]
    fn reads_decorated_functions_with_their_help() {
        let StmtKind::Def(f) = one("@on_enter(Roller3, priority=5)\ndef count(pallet, n=2):\n    \"\"\"Counts.\"\"\"\n    return n\n")
        else {
            panic!()
        };
        assert_eq!((f.name.as_str(), f.params.len(), f.decorators.len(), f.doc.as_deref()), ("count", 2, 1, Some("Counts.")));
        assert_eq!(f.body.len(), 1);
    }

    #[test]
    fn follows_pythons_precedence() {
        let StmtKind::Expr(e) = one("-2 ** 2\n") else { panic!() };
        assert!(matches!(e.kind, ExprKind::Unary(UnOp::Neg, _)));
        let StmtKind::Expr(e) = one("0.25 * m / s\n") else { panic!() };
        assert!(matches!(e.kind, ExprKind::Binary(BinOp::Div, _, _)));
        let StmtKind::Expr(e) = one("not a and b\n") else { panic!() };
        assert!(matches!(e.kind, ExprKind::And(..)));
        let StmtKind::Expr(e) = one("1 < x <= 3\n") else { panic!() };
        assert!(matches!(e.kind, ExprKind::Compare(_, ops) if ops.len() == 2));
    }

    #[test]
    fn reads_comprehensions_slices_dicts_and_f_strings() {
        assert!(matches!(
            one("y = [z for z in zones if z.occupied]\n"),
            StmtKind::Assign { value: Expr { kind: ExprKind::ListComp { .. }, .. }, .. }
        ));
        assert!(matches!(one("y = l[1:-1]\n"), StmtKind::Assign { value: Expr { kind: ExprKind::Index(..), .. }, .. }));
        assert!(
            matches!(one("d = {'a': 1, 'b': 2}\n"), StmtKind::Assign { value: Expr { kind: ExprKind::Dict(p), .. }, .. } if p.len() == 2)
        );
        assert!(matches!(one("print(f\"{x + 1:.1f} mm\")\n"), StmtKind::Expr(_)));
    }

    #[test]
    fn says_what_python_features_are_left_out() {
        for (src, words) in [
            ("class A:\n    pass\n", "classes"),
            ("f = lambda x: x\n", "lambda"),
            ("try:\n    pass\n", "try"),
            ("x := 3\n", ":="),
            ("if a <> b:\n    pass\n", "!="),
        ] {
            let e = parse(src).unwrap_err();
            assert!(e.message.contains(words), "{src}: {}", e.message);
        }
        assert_eq!(parse("x = (1,\n").unwrap_err().code, "syntax");
        assert!(parse("1 = x\n").is_err());
    }

    #[test]
    fn deep_nesting_is_an_error_not_a_crash() {
        let src = format!("x = {}1{}\n", "(".repeat(5000), ")".repeat(5000));
        assert!(parse(&src).is_err());
    }
}
