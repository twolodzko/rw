use crate::operators;
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    NewOp(String, u8),
    LetEquals(Expr, Expr),
    LetMap(Expr, Expr),
    Rewrite(Expr),
    Load(String),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            NewOp(s, p) => write!(f, "infix {} {}", s, p),
            LetEquals(l, r) => write!(f, "let {} <=> {}", l, r),
            LetMap(l, r) => write!(f, "let {} => {}", l, r),
            Rewrite(e) => write!(f, "rewrite {}", e),
            Load(s) => write!(f, "load {}", s),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Wildcard,
    Not(Box<Expr>),
    Literal(String),
    Cons(Box<Expr>, Box<Expr>),
    Infix(usize, Box<Expr>, Box<Expr>),
    Variable(String, Box<Expr>),
}

impl Expr {
    pub(crate) fn complexity(&self) -> usize {
        use Expr::*;
        match self {
            Wildcard | Literal(_) | Variable(_, _) => 1,
            Not(inner) => 1 + inner.complexity(),
            Cons(head, tail) => head.complexity() + tail.complexity(),
            Infix(_, lhs, rhs) => 1 + lhs.complexity() + rhs.complexity(),
        }
    }

    pub(crate) fn is_constant(&self) -> bool {
        use Expr::*;
        match self {
            Literal(_) | Wildcard => true,
            Variable(_, _) => false,
            Not(inner) => inner.is_constant(),
            Cons(head, tail) => head.is_constant() && tail.is_constant(),
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

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;
        match self {
            Wildcard => write!(f, "_"),
            Not(v) if is_atom(v) => write!(f, "~{}", v),
            Not(v) => write!(f, "~({})", v),
            Literal(v) => write!(f, "{}", v),
            Cons(h, t) => write!(f, "{} {}", h, t),
            Infix(op, a, b) => {
                let op = operators::name(*op);
                match (is_atom(a), is_atom(b)) {
                    (true, false) => write!(f, "{} {} ({})", a, op, b),
                    (false, true) => write!(f, "({}) {} {}", a, op, b),
                    (false, false) => write!(f, "({}) {} ({})", a, op, b),
                    (true, true) => write!(f, "{} {} {}", a, op, b),
                }
            }
            Variable(v, c) if let Wildcard = c.as_ref() => write!(f, "${}", v),
            Variable(v, c) if is_atom(c) => write!(f, "${}:({})", v, c),
            Variable(v, c) => write!(f, "${}:{}", v, c),
        }
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;
        match self {
            Wildcard => write!(f, "_"),
            Not(v) if is_atom(v) => write!(f, "~{:?}", v),
            Not(v) => write!(f, "~({:?})", v),
            Literal(v) => write!(f, "{}", v),
            Cons(h, t) => write!(f, "({:?} {:?})", h, t),
            Infix(op, a, b) => {
                let op = operators::name(*op);
                match (is_atom(a), is_atom(b)) {
                    (true, false) => write!(f, "{:?} {} ({:?})", a, op, b),
                    (false, true) => write!(f, "({:?}) {} {:?}", a, op, b),
                    (false, false) => write!(f, "({:?}) {} ({:?})", a, op, b),
                    (true, true) => write!(f, "{:?} {} {:?}", a, op, b),
                }
            }
            Variable(v, c) if let Wildcard = c.as_ref() => write!(f, "${}", v),
            Variable(v, c) if is_atom(c) => write!(f, "${}:({:?})", v, c),
            Variable(v, c) => write!(f, "${}:{:?}", v, c),
        }
    }
}

fn is_atom(expr: &Expr) -> bool {
    use Expr::*;
    matches!(expr, Literal(_) | Variable(_, _) | Wildcard | Not(_))
}
