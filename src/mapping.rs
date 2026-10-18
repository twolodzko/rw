use crate::{operators::Op, types::Expr};
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapping {
    pattern: Expr,
    template: Expr,
}

impl Mapping {
    pub(crate) fn new(pattern: Expr, template: Expr) -> Result<Mapping> {
        validate(&pattern, &template)?;
        Ok(Mapping { pattern, template })
    }

    pub(crate) fn reversed(&self) -> Result<Mapping> {
        Mapping::new(self.template.clone(), self.pattern.clone())
    }

    pub(crate) fn apply(&self, expr: &Expr) -> Vec<Expr> {
        let mut acc = Vec::new();

        let mut vars = HashMap::new();
        if is_match(&self.pattern, expr, &mut vars) {
            if vars.is_empty() {
                acc.push(self.template.to_owned());
            } else {
                acc.push(render(&self.template, &vars));
            }
        }

        use Expr::*;
        match expr {
            Cons(head, tail) => {
                for s in self
                    .apply(head)
                    .into_iter()
                    .map(|v| Cons(Box::new(v), tail.to_owned()))
                {
                    acc.push(s)
                }
                for s in self
                    .apply(tail)
                    .into_iter()
                    .map(|v| Cons(head.to_owned(), Box::new(v)))
                {
                    acc.push(s)
                }
            }
            Prefix(op, val) => {
                for s in self
                    .apply(val)
                    .into_iter()
                    .map(|v| Prefix(*op, Box::new(v)))
                {
                    acc.push(s)
                }
            }
            Infix(op, lhs, rhs) => {
                for s in self
                    .apply(lhs)
                    .into_iter()
                    .map(|v| Infix(*op, Box::new(v), rhs.to_owned()))
                {
                    acc.push(s)
                }
                for s in self
                    .apply(rhs)
                    .into_iter()
                    .map(|v| Infix(*op, lhs.to_owned(), Box::new(v)))
                {
                    acc.push(s)
                }
            }
            _ => {}
        }
        acc
    }

    /// Literal to literal mapping
    pub(crate) fn literal_edge(&self) -> Option<(&str, &str)> {
        if let (Expr::Literal(p), Expr::Literal(t)) = (&self.pattern, &self.template) {
            Some((p.as_str(), t.as_str()))
        } else {
            None
        }
    }

    pub(crate) fn constant(&self) -> Option<(Expr, Expr)> {
        if self.pattern.is_constant() && self.template.is_constant() {
            Some((self.pattern.clone(), self.template.clone()))
        } else {
            None
        }
    }
}

fn is_match(pattern: &Expr, value: &Expr, vars: &mut HashMap<String, Expr>) -> bool {
    use Expr::*;
    match (pattern, value) {
        (Wildcard, _) | (_, Wildcard) => true,
        (Bool(l), Bool(r)) => l == r,
        (Literal(l), Literal(r)) => l == r,
        (Cons(lh, lt), Cons(rh, rt)) => is_match(lh, rh, vars) && is_match(lt, rt, vars),
        (Variable(k), v) => {
            debug_assert!(!matches!(v, Variable(_)));
            if let Some(old) = vars.insert(k.to_owned(), v.clone()) {
                old == *v
            } else {
                true
            }
        }
        (Prefix(lo, lv), Prefix(ro, rv)) => lo == ro && is_match(lv, rv, vars),
        (Infix(lo, la, lb), Infix(ro, ra, rb)) => {
            lo == ro && is_match(la, ra, vars) && is_match(lb, rb, vars)
        }
        _ => false,
    }
}

fn render(template: &Expr, vars: &HashMap<String, Expr>) -> Expr {
    use Expr::*;
    match template {
        // unwrap is ok because we already check for uninitialized variables in Mapping::new
        Variable(k) => vars.get(k).unwrap().clone(),
        Cons(head, tail) => {
            let head = render(head, vars);
            let tail = render(tail, vars);
            Cons(Box::new(head), Box::new(tail))
        }
        Prefix(op, val) => {
            let val = render(val, vars);
            Prefix(*op, Box::new(val))
        }
        Infix(op, lhs, rhs) => {
            let lhs = render(lhs, vars);
            let rhs = render(rhs, vars);
            Infix(*op, Box::new(lhs), Box::new(rhs))
        }
        _ => template.clone(),
    }
}

fn validate(pattern: &Expr, template: &Expr) -> Result<()> {
    use Expr::*;

    fn collect(expr: &Expr, seen: &mut HashSet<String>) {
        match expr {
            Variable(k) => {
                let _ = seen.insert(k.to_string());
            }
            Cons(head, tail) => {
                collect(head, seen);
                collect(tail, seen);
            }
            Prefix(_, val) => collect(val, seen),
            Infix(_, lhs, rhs) => {
                collect(lhs, seen);
                collect(rhs, seen);
            }
            _ => {}
        }
    }

    fn check(expr: &Expr, seen: &HashSet<String>) -> Result<()> {
        match expr {
            v @ Variable(k) => {
                if !seen.contains(k) {
                    bail!("unknown key: {}", v)
                }
            }
            Cons(head, tail) => {
                check(head, seen)?;
                return check(tail, seen);
            }
            Prefix(_, val) => check(val, seen)?,
            Infix(_, lhs, rhs) => {
                check(lhs, seen)?;
                return check(rhs, seen);
            }
            _ => {}
        }
        Ok(())
    }

    let mut seen = HashSet::new();
    collect(pattern, &mut seen);
    check(template, &seen)
}

impl TryFrom<Expr> for Mapping {
    type Error = anyhow::Error;

    fn try_from(value: Expr) -> Result<Self> {
        use Expr::*;
        match value {
            Prefix(Op::Not, expr) => {
                expr.validate()?;
                Mapping::new(*expr, Bool(false))
            }
            Infix(Op::Equals, lhs, rhs) => Mapping::new(*lhs, *rhs),
            _ => {
                value.validate()?;
                Mapping::new(value, Bool(true))
            }
        }
    }
}

impl std::fmt::Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.pattern, self.template)
    }
}
