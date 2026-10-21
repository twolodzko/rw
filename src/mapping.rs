use crate::types::Expr;
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

    /// If it is possible to merge the new mapping with this mapping do so and return a success status
    pub(crate) fn try_merge(&mut self, other: &Mapping) -> bool {
        if !other.pattern.is_constant() || !other.template.is_constant() {
            return false;
        }
        if self.template == other.template {
            use Expr::*;
            match (&mut self.pattern, &other.pattern) {
                (Wildcard, _) => {}
                (_, Wildcard) => self.pattern = Wildcard,
                (Or(this), Or(other)) => {
                    for p in other {
                        if !this.contains(p) {
                            this.push(p.clone());
                        }
                    }
                }
                (Or(this), other) => {
                    if !this.contains(other) {
                        this.push(other.clone());
                    }
                }
                _ => self.pattern = Or(vec![self.pattern.clone(), other.pattern.clone()]),
            }
            return true;
        }
        false
    }
}

fn is_match(pattern: &Expr, value: &Expr, vars: &mut HashMap<String, Expr>) -> bool {
    use Expr::*;
    match (pattern, value) {
        (Wildcard, _) | (_, Wildcard) => true,
        (Not(n), v) | (v, Not(n)) => !is_match(n, v, &mut vars.clone()),
        (Literal(l), Literal(r)) => l == r,
        (Cons(lh, lt), Cons(rh, rt)) => is_match(lh, rh, vars) && is_match(lt, rt, vars),
        (Variable(k, c), v) if let Wildcard = c.as_ref() => {
            debug_assert!(!matches!(v, Variable(_, _)));
            if let Some(old) = vars.insert(k.to_owned(), v.clone()) {
                old == *v
            } else {
                true
            }
        }
        (Variable(k, c), v) => {
            debug_assert!(!matches!(v, Variable(_, _)));
            if !is_match(c.as_ref(), v, vars) {
                return false;
            }
            if let Some(old) = vars.insert(k.to_owned(), v.clone()) {
                old == *v
            } else {
                true
            }
        }
        (Infix(lo, la, lb), Infix(ro, ra, rb)) => {
            lo == ro && is_match(la, ra, vars) && is_match(lb, rb, vars)
        }
        (Or(set), _) => {
            for p in set {
                let copy = vars.clone();
                if is_match(p, value, vars) {
                    return true;
                }
                *vars = copy;
            }
            false
        }
        _ => false,
    }
}

fn render(template: &Expr, vars: &HashMap<String, Expr>) -> Expr {
    use Expr::*;
    match template {
        // unwrap is ok because we already check for uninitialized variables in Mapping::new
        Variable(k, _) => vars.get(k).unwrap().clone(),
        Cons(head, tail) => {
            let head = render(head, vars);
            let tail = render(tail, vars);
            Cons(Box::new(head), Box::new(tail))
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
            Variable(k, c) => {
                collect(c, seen);
                let _ = seen.insert(k.to_string());
            }
            Infix(_, lhs, rhs) | Cons(lhs, rhs) => {
                collect(lhs, seen);
                collect(rhs, seen);
            }
            _ => {}
        }
    }

    fn check(expr: &Expr, seen: &HashSet<String>) -> Result<()> {
        match expr {
            v @ Variable(k, c) => {
                if !seen.contains(k) {
                    bail!("unknown key: {}", v)
                }
                check(c, seen)?;
            }
            Infix(_, lhs, rhs) | Cons(lhs, rhs) => {
                check(lhs, seen)?;
                return check(rhs, seen);
            }
            Or(_) => bail!("OR patterns are not allowed in templates"),
            _ => {}
        }
        Ok(())
    }

    let mut seen = HashSet::new();
    collect(pattern, &mut seen);
    check(template, &seen)
}

impl std::fmt::Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.pattern, self.template)
    }
}
