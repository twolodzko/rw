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
        for old in &mut self.rules {
            if m == *old {
                return Ok(());
            }

            if old.try_merge(&m) {
                if let Some((lhs, rhs)) = m.literal_edge() {
                    self.update_edges(lhs, rhs)?;
                }
                return Ok(());
            }
        }
        self.rules.push(m.clone());
        if let Some((lhs, rhs)) = m.literal_edge() {
            self.update_edges(lhs, rhs)?;
        }
        Ok(())
    }

    fn update_edges(&mut self, lhs: &str, rhs: &str) -> Result<()> {
        if !self
            .outgoing
            .entry(lhs.to_string())
            .or_default()
            .insert(rhs.to_string())
        {
            // the edge already exists
            return Ok(());
        }
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

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.rules
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
