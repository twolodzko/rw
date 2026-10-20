use crate::{
    mapping::Mapping,
    queue::Queue,
    types::Expr::{self, Literal},
};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Debug, Default, Clone)]
pub struct Context {
    rules: Vec<Mapping>,
    incoming: HashMap<String, HashSet<String>>,
    outgoing: HashMap<String, HashSet<String>>,
}

impl Context {
    pub(crate) fn rewrite(&mut self, expr: Expr) -> Expr {
        let mut queue = Queue::default();
        let mut this = Rc::new(expr.clone());
        let mut best: Option<Rc<Expr>> = None;

        loop {
            if self.is_final(&this) {
                return Rc::unwrap_or_clone(this);
            }

            let mut succeeded = false;
            for m in &self.rules {
                for s in m.apply(&this) {
                    succeeded = true;
                    queue.insert(s);
                }
            }

            if !succeeded
                && best
                    .as_ref()
                    .is_none_or(|b| this.complexity() < b.complexity())
            {
                best = Some(Rc::clone(&this));
            }

            if let Some(s) = queue.pop() {
                this = s;
            } else {
                return best.map_or(expr, Rc::unwrap_or_clone);
            }
        }
    }

    fn is_final(&self, expr: &Expr) -> bool {
        if let Expr::Literal(key) = expr
            && let Some(out) = self.outgoing.get(key)
        {
            out.is_empty()
        } else {
            false
        }
    }

    pub(crate) fn insert_mapping(&mut self, m: Mapping) -> Result<()> {
        if !self.rules.contains(&m) {
            self.rules.push(m.clone());
            if let Some((lhs, rhs)) = m.literal_edge() {
                self.update_edges(lhs, rhs)?;
            }
        }
        Ok(())
    }

    fn update_edges(&mut self, lhs: &str, rhs: &str) -> Result<()> {
        // update edges for the mapping
        self.outgoing
            .entry(lhs.to_string())
            .or_default()
            .insert(rhs.to_string());
        self.incoming
            .entry(rhs.to_string())
            .or_default()
            .insert(lhs.to_string());

        // all the incoming edges get a new outgoing edge
        if self.incoming.contains_key(lhs) {
            for inc in self.incoming.get(lhs).cloned().unwrap() {
                if inc != lhs && inc != rhs {
                    self.insert_mapping(Mapping::new(
                        Literal(inc.to_string()),
                        Literal(rhs.to_string()),
                    )?)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::Context;
    // use crate::{eval, parser::Parser, types::Statement};
    // use anyhow::Result;

    // fn solve_all(source: &str) -> Result<Vec<String>> {
    //     let mut rules = Context::default();
    //     let mut acc = Vec::new();

    //     for stmt in Parser::from(source) {
    //         match stmt? {
    //             Statement::Rewrite(expr) => {
    //                 acc.push(rules.rewrite(expr).to_string());
    //             }
    //             stmt => eval(stmt, &mut rules)?,
    //         }
    //     }
    //     Ok(acc)
    // }

    // #[test]
    // fn apply_all_reductions() {
    //     let source = r"infix + 100
    //                    let $A + 0 => $A
    //                    let $A + $B => $B + $A
    //                    rewrite (0 + 5) + 0";
    //     assert_eq!(solve_all(source).unwrap(), vec!["5".to_string()]);
    // }

    // #[test]
    // fn rewrite_is_deterministic() {
    //     let source = r"infix + 100
    //                    let $A + 0 => $A
    //                    let $A + $B => $B + $A
    //                    rewrite (0 + 5) + 0";
    //     let first = solve_all(source).unwrap();
    //     for _ in 0..20 {
    //         assert_eq!(solve_all(source).unwrap(), first);
    //     }
    // }

    // #[test]
    // fn rewrite_applies_mapping() {
    //     let source = r"let foo => bar
    //                    rewrite foo";
    //     assert_eq!(solve_all(source).unwrap(), vec!["bar".to_string()]);
    // }

    // #[test]
    // fn literal_mappings() {
    //     let source = r"let a => b
    //                    rewrite a = b
    //                    let b => c
    //                    rewrite a = c";
    //     assert_eq!(
    //         solve_all(source).unwrap(),
    //         vec!["b = b".to_string(), "c = c".to_string()]
    //     );
    // }

    // #[test]
    // fn mappings_are_unidirectional() {
    //     let source = r"let a => b
    //                    let a => c
    //                    rewrite b = c";
    //     assert_eq!(solve_all(source).unwrap(), vec!["b = c".to_string()]);
    // }

    // #[test]
    // fn validate_facts() {
    //     let source = r"let a
    //                    let not b
    //                    let a and (b and c)";
    //     assert!(solve_all(source).is_err())
    // }
}
