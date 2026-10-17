use crate::{mapping::Mapping, operators::Op};
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    NewPrefix(String, u8),
    NewInfix(String, u8),
    Let(Expr),
    LetMap(Mapping),
    Eval(Expr),
    EvalTo(String, Expr),
    Assert(Expr),
    Load(String),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Bool(bool),
    Literal(String),
    Cons(Box<Expr>, Box<Expr>),
    Prefix(Op, Box<Expr>),
    Infix(Op, Box<Expr>, Box<Expr>),
    Variable(String),
}

impl Expr {
    pub(crate) fn complexity(&self) -> usize {
        use Expr::*;
        match self {
            Bool(_) | Literal(_) | Variable(_) => 1,
            Cons(head, tail) => head.complexity() + tail.complexity(),
            Prefix(_, val) => 1 + val.complexity(),
            Infix(_, lhs, rhs) => 1 + lhs.complexity() + rhs.complexity(),
        }
    }

    pub(crate) fn is_constant(&self) -> bool {
        use Expr::*;
        match self {
            Bool(_) | Literal(_) => true,
            Variable(_) => false,
            Cons(head, tail) => head.is_constant() && tail.is_constant(),
            Prefix(_, val) => val.is_constant(),
            Infix(_, lhs, rhs) => lhs.is_constant() && rhs.is_constant(),
        }
    }

    /// Use of variables is disallowed outside of mappings
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.is_constant() {
            bail!("variables are not allowed outside of mappings")
        }
        Ok(())
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            NewPrefix(s, p) => write!(f, "prefix {} {}", s, p),
            NewInfix(s, p) => write!(f, "infix {} {}", s, p),
            Let(e) => write!(f, "let {}", e),
            LetMap(m) => write!(f, "let {}", m),
            Eval(e) => write!(f, "eval {}", e),
            EvalTo(k, e) => write!(f, "let {} := {}", k, e),
            Assert(e) => write!(f, "assert {}", e),
            Load(s) => write!(f, "load {}", s),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;
        fn is_atom(expr: &Expr) -> bool {
            matches!(expr, Literal(_) | Bool(_) | Variable(_))
        }

        match self {
            Bool(v) => write!(f, "{}", v),
            Literal(v) => write!(f, "{}", v),
            Cons(h, t) => write!(f, "{} {}", h, t),
            Prefix(op, e) if is_atom(e) => write!(f, "{} {}", op, e),
            Prefix(op, e) => write!(f, "{} ({})", op, e),
            Infix(op, a, b) => match (is_atom(a), is_atom(b)) {
                (true, false) => write!(f, "{} {} ({})", a, op, b),
                (false, true) => write!(f, "({}) {} {}", a, op, b),
                (false, false) => write!(f, "({}) {} ({})", a, op, b),
                (true, true) => write!(f, "{} {} {}", a, op, b),
            },
            Variable(v) => write!(f, "${}", v),
        }
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn is_atom(expr: &Expr) -> bool {
            use Expr::*;
            matches!(expr, Literal(_) | Bool(_) | Variable(_))
        }

        use Expr::*;
        match self {
            Bool(v) => write!(f, "{}", v),
            Literal(v) => write!(f, "{}", v),
            Cons(h, t) => write!(f, "({:?} {:?})", h, t),
            Prefix(op, e) if is_atom(e) => write!(f, "{} {:?}", op, e),
            Prefix(op, e) => write!(f, "{} ({:?})", op, e),
            Infix(op, a, b) => match (is_atom(a), is_atom(b)) {
                (true, false) => write!(f, "{:?} {} ({:?})", a, op, b),
                (false, true) => write!(f, "({:?}) {} {:?}", a, op, b),
                (false, false) => write!(f, "({:?}) {} ({:?})", a, op, b),
                (true, true) => write!(f, "{:?} {} {:?}", a, op, b),
            },
            Variable(v) => write!(f, "${}", v),
        }
    }
}
