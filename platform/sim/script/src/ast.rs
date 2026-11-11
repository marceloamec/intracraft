//! The program as the parser reads it: statements and expressions with the line and column they start at.

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    /// `a = b = value`: every target gets the value.
    Assign {
        targets: Vec<Expr>,
        value: Expr,
    },
    AugAssign {
        target: Expr,
        op: BinOp,
        value: Expr,
    },
    /// `x: int = 3`; the annotation documents, it is not checked.
    AnnAssign {
        target: Expr,
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        target: Expr,
        iter: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Pass,
    Return(Option<Expr>),
    Global(Vec<String>),
    Def(Box<FuncDef>),
    /// `from intrascript import *`, `import math`: only the built-in modules.
    Import {
        module: String,
    },
    Assert {
        cond: Expr,
        msg: Option<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    /// The text of a first-line string, shown by the editor as the function's help.
    pub doc: Option<String>,
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Str(String),
    FStr(Vec<FPart>),
    Bool(bool),
    None,
    Name(String),
    Attr(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Slice(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        kwargs: Vec<(String, Expr)>,
    },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `a < b <= c`: the first operand and each comparison after it.
    Compare(Box<Expr>, Vec<(CmpOp, Expr)>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    IfExp {
        cond: Box<Expr>,
        then: Box<Expr>,
        orelse: Box<Expr>,
    },
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    ListComp {
        elt: Box<Expr>,
        target: Box<Expr>,
        iter: Box<Expr>,
        cond: Option<Box<Expr>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum FPart {
    Lit(String),
    Expr(Expr, Option<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::FloorDiv => "//",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
    Is,
    IsNot,
}

impl Expr {
    pub fn new(kind: ExprKind, line: u32, col: u32) -> Expr {
        Expr { kind, line, col }
    }

    /// `a.b.c` as `["a", "b", "c"]`, when the expression is only names and dots.
    pub fn dotted(&self) -> Option<Vec<String>> {
        match &self.kind {
            ExprKind::Name(n) => Some(vec![n.clone()]),
            ExprKind::Attr(base, name) => {
                let mut v = base.dotted()?;
                v.push(name.clone());
                Some(v)
            }
            _ => None,
        }
    }
}
