use crate::types::Expr;
use anyhow::{Ok, Result, bail};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapping {
    pub(crate) pattern: Expr,
    pub(crate) template: Expr,
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
                    .map(|v| Cons(Rc::new(v), tail.to_owned()))
                {
                    acc.push(s)
                }
                for s in self
                    .apply(tail)
                    .into_iter()
                    .map(|v| Cons(head.to_owned(), Rc::new(v)))
                {
                    acc.push(s)
                }
            }
            Infix(op, lhs, rhs) => {
                for s in self
                    .apply(lhs)
                    .into_iter()
                    .map(|v| Infix(*op, Rc::new(v), rhs.to_owned()))
                {
                    acc.push(s)
                }
                for s in self
                    .apply(rhs)
                    .into_iter()
                    .map(|v| Infix(*op, lhs.to_owned(), Rc::new(v)))
                {
                    acc.push(s)
                }
            }
            Not(expr) => {
                for e in self.apply(expr) {
                    acc.push(Not(Rc::new(e)))
                }
            }
            Or(set) => {
                for (i, e) in set.iter().enumerate() {
                    for v in self.apply(e) {
                        let mut copy = set.clone();
                        copy[i] = v;
                        acc.push(Expr::new_or(copy));
                    }
                }
            }
            Wildcard | Literal(_) | Variable(_, _) => {}
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
                // merge sets
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
                (this, Or(other)) => {
                    let mut acc = vec![this.clone()];
                    acc.extend_from_slice(other);
                    self.pattern = Expr::new_or(acc)
                }
                // create new set
                _ => self.pattern = Expr::new_or(vec![self.pattern.clone(), other.pattern.clone()]),
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
            Cons(Rc::new(head), Rc::new(tail))
        }
        Infix(op, lhs, rhs) => {
            let lhs = render(lhs, vars);
            let rhs = render(rhs, vars);
            Infix(*op, Rc::new(lhs), Rc::new(rhs))
        }
        Or(set) => Expr::new_or(set.iter().map(|e| render(e, vars)).collect()),
        Not(expr) => Not(Rc::new(render(expr, vars))),
        Wildcard | Literal(_) => template.clone(),
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
            Not(expr) => collect(expr, seen),
            Or(set) => set.iter().for_each(|e| collect(e, seen)),
            Wildcard | Literal(_) => {}
        }
    }

    fn check(expr: &Expr, seen: &HashSet<String>) -> Result<()> {
        match expr {
            v @ Variable(k, c) => {
                if !seen.contains(k) {
                    bail!("unknown key: {}", v)
                }
                check(c, seen)
            }
            Infix(_, lhs, rhs) | Cons(lhs, rhs) => {
                check(lhs, seen)?;
                check(rhs, seen)
            }
            Not(expr) => check(expr, seen),
            Wildcard | Literal(_) => Ok(()),
            // because or running Expr::expand_or in Context::insert_mapping
            // OR patterns would not appear in templates
            Or(_) => unreachable!(),
        }
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
