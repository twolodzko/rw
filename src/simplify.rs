use crate::{mapping::Mapping, operators::Op, queue::Queue, types::Expr};
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone)]
pub struct Context {
    /// mappings to be applied
    rules: Vec<Mapping>,
    /// things that are known to be true or false
    facts: HashMap<Expr, bool>,
    /// list of literals this literal maps to
    adjacency: HashMap<String, Vec<String>>,
    /// literal nodes reachable from this literal
    reachable: HashMap<String, HashSet<String>>,
}

impl Context {
    pub(crate) fn simplify(&mut self, expr: Expr) -> Expr {
        let mut queue = Queue::default();
        let mut this = expr.clone();
        let mut best: Option<Expr> = None;

        loop {
            if self.is_true(&this) {
                return Expr::Bool(true);
            }

            let mut succeeded = false;
            for m in &self.rules {
                for s in m.apply(&this) {
                    if let Expr::Bool(_) = s {
                        return s;
                    }
                    succeeded = true;
                    queue.insert(s);
                }
            }

            if !succeeded
                && best
                    .as_ref()
                    .is_none_or(|b| this.complexity() < b.complexity())
            {
                best = Some(this.clone());
            }

            if let Some(s) = queue.pop() {
                this = s;
            } else {
                return best.unwrap_or(expr);
            }
        }
    }

    fn is_true(&mut self, expr: &Expr) -> bool {
        if let Some(v) = self.facts.get(expr) {
            return *v;
        }
        use Expr::*;
        match expr {
            Windcard => true,
            Bool(v) => *v,
            Prefix(Op::Not, val) if let Windcard = **val => true,
            Prefix(Op::Not, val) => !self.is_true(val),
            Infix(Op::Equals, lhs, rhs) => self.equal(lhs, rhs),
            Infix(Op::And, lhs, rhs) => self.is_true(lhs) && self.is_true(rhs),
            Infix(Op::Or, lhs, rhs) => self.is_true(lhs) || self.is_true(rhs),
            Infix(Op::Imply, lhs, rhs) => !self.is_true(lhs) || self.is_true(rhs),
            _ => false,
        }
    }

    fn equal(&mut self, lhs: &Expr, rhs: &Expr) -> bool {
        use Expr::*;
        match (lhs, rhs) {
            (Windcard, _) | (_, Windcard) => true,
            (Bool(l), Bool(r)) => l == r,
            (Literal(l), Literal(r)) => self.literals_equal(l, r),
            (Cons(lh, lt), Cons(rh, rt)) => self.equal(lh, rh) && self.equal(lt, rt),
            (Prefix(lo, lv), Prefix(ro, rv)) if lo == ro => self.equal(lv, rv),
            (Infix(lo, la, lb), Infix(ro, ra, rb)) if lo == ro => {
                self.equal(la, ra) && self.equal(lb, rb)
            }
            _ => false,
        }
    }

    fn literals_equal(&mut self, lhs: &str, rhs: &str) -> bool {
        if lhs == rhs {
            return true;
        }
        if !self.reachable.contains_key(lhs) {
            self.build_reachable(lhs);
        }
        self.reachable[lhs].contains(rhs)
    }

    fn build_reachable(&mut self, lhs: &str) {
        let mut visited = HashSet::new();
        let mut queue = vec![lhs];
        while let Some(node) = queue.pop() {
            if !visited.insert(node.to_string()) {
                continue;
            }
            if let Some(next) = self.adjacency.get(node) {
                for n in next {
                    if !visited.contains(n) {
                        queue.push(n);
                    }
                }
            }
        }
        visited.remove(lhs);
        self.reachable.insert(lhs.to_string(), visited);
    }

    pub(crate) fn insert_mapping(&mut self, m: Mapping) -> Result<()> {
        if !self.rules.contains(&m) {
            // Validate and save facts
            if let Some((key, val)) = m.constant() {
                if let Expr::Bool(val) = val {
                    self.insert_fact(key.clone(), val)?;
                } else if let Some(k) = self.facts.get(&key)
                    && let Some(v) = self.facts.get(&val)
                    && k != v
                {
                    bail!("{} is inconsistent with previous facts", m)
                }
            }
            // Update adjacency representation
            if let Some((lhs, rhs)) = m.literal_edge() {
                self.adjacency
                    .entry(lhs.to_owned())
                    .or_default()
                    .push(rhs.to_owned());
                self.reachable.clear();
            }
            // Save the rule
            self.rules.push(m.clone());
        }
        Ok(())
    }

    fn insert_fact(&mut self, key: Expr, val: bool) -> Result<()> {
        if self.is_fact_true(&key).is_some_and(|old| old != val) {
            bail!("{} is inconsistent with previous facts", key)
        }
        self.facts.insert(key.clone(), val);

        // propagate to adjacent facts
        if let Expr::Literal(ref k) = key
            && let Some(adj) = self.adjacency.get(k).cloned()
        {
            for key in adj {
                self.insert_fact(Expr::Literal(key), val)?;
            }
        }
        Ok(())
    }

    fn is_fact_true(&self, key: &Expr) -> Option<bool> {
        if let Some(val) = self.facts.get(key) {
            return Some(*val);
        }
        use Expr::*;
        match key {
            Bool(val) => Some(*val),
            Prefix(Op::Not, k) => self.is_fact_true(k).map(|v| !v),
            Infix(Op::Equals, lhs, rhs) => {
                let lhs = self.is_fact_true(lhs)?;
                let rhs = self.is_fact_true(rhs)?;
                Some(lhs == rhs)
            }
            Infix(Op::And, lhs, rhs) => {
                if !self.is_fact_true(lhs)? {
                    return Some(false);
                }
                self.is_fact_true(rhs)
            }
            Infix(Op::Or, lhs, rhs) => {
                if self.is_fact_true(lhs)? {
                    return Some(true);
                }
                self.is_fact_true(rhs)
            }
            Infix(Op::Imply, lhs, rhs) => {
                if !self.is_fact_true(lhs)? {
                    return Some(true);
                }
                self.is_fact_true(rhs)
            }
            _ => None,
        }
    }

    pub(crate) fn merge(&mut self, other: Context) -> Result<()> {
        for m in other.rules {
            self.insert_mapping(m)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::{eval, parser::Parser, types::Statement};
    use anyhow::Result;

    fn solve_all(source: &str) -> Result<Vec<String>> {
        let mut rules = Context::default();
        let mut acc = Vec::new();

        for stmt in Parser::from(source) {
            match stmt? {
                Statement::Eval(expr) => {
                    acc.push(rules.simplify(expr).to_string());
                }
                stmt => eval(stmt, &mut rules)?,
            }
        }
        Ok(acc)
    }

    #[test]
    fn apply_all_reductions() {
        let source = r"infix + 100
                       let $A + 0 => $A
                       let $A + $B => $B + $A
                       eval (0 + 5) + 0";
        assert_eq!(solve_all(source).unwrap(), vec!["5".to_string()]);
    }

    #[test]
    fn simplify_is_deterministic() {
        let source = r"infix + 100
                       let $A + 0 => $A
                       let $A + $B => $B + $A
                       eval (0 + 5) + 0";
        let first = solve_all(source).unwrap();
        for _ in 0..20 {
            assert_eq!(solve_all(source).unwrap(), first);
        }
    }

    #[test]
    fn simplify_applies_mapping() {
        let source = r"let foo => bar
                       eval foo";
        assert_eq!(solve_all(source).unwrap(), vec!["bar".to_string()]);
    }

    #[test]
    fn literal_mappings() {
        let source = r"let a => b
                       eval a = b
                       let b => c
                       eval a = c";
        assert_eq!(
            solve_all(source).unwrap(),
            vec!["true".to_string(), "true".to_string()]
        );
    }

    #[test]
    fn mappings_are_unidirectional() {
        let source = r"let a => b
                       let a => c
                       eval b = c";
        assert_eq!(solve_all(source).unwrap(), vec!["b = c".to_string()]);
    }

    #[test]
    fn validate_facts() {
        let source = r"let a
                       let not b
                       let a and (b and c)";
        assert!(solve_all(source).is_err())
    }
}
