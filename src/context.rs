use crate::{
    mapping::Mapping,
    types::Expr::{self, Literal},
};
use anyhow::Result;
use itertools::Itertools;
use relative_path::RelativePath;
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    path::PathBuf,
    rc::Rc,
};

#[derive(Debug, Default, Clone)]
pub struct Context {
    pub(crate) rules: Vec<Mapping>,
    pub(crate) incoming: HashMap<String, HashSet<String>>,
    pub(crate) outgoing: HashMap<String, HashSet<String>>,
    workdir: Vec<PathBuf>,
    symmetries: Symmetries,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Symmetries {
    operators: HashSet<usize>,
    functions: HashSet<String>,
}

impl Context {
    pub(crate) fn insert_mapping(&mut self, mut lhs: Expr, rhs: Expr) -> Result<()> {
        lhs.expand_symmetries(&self.symmetries);
        for rhs in rhs.expand_or() {
            let m = Mapping::new(lhs.clone(), rhs)?;
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
        }
        Ok(())
    }

    pub(crate) fn update_symmetries(&mut self, expr: &Expr) {
        self.symmetries.insert(expr);
        for m in &mut self.rules {
            m.pattern.expand_symmetries(&self.symmetries);
        }
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
                    self.insert_mapping(Literal(inc.to_string()), Literal(rhs.to_string()))?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn update_path(&mut self, path: &str) -> Result<PathBuf> {
        let parent = if let Some(workdir) = self.workdir.last() {
            workdir
        } else {
            &current_dir()?
        };
        let path = RelativePath::new(&path).to_path(parent);
        let parent = path.parent().unwrap();
        self.workdir.push(parent.to_path_buf());
        Ok(path)
    }

    pub(crate) fn exit_path(&mut self) {
        self.workdir.pop();
    }
}

impl Symmetries {
    fn insert(&mut self, expr: &Expr) {
        use Expr::*;
        match expr {
            Cons(name, _) if let Literal(name) = name.as_ref() => {
                self.functions.insert(name.to_string());
            }
            Infix(op, _, _) => {
                self.operators.insert(*op);
            }
            _ => unreachable!(),
        }
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

impl Expr {
    /// Expand all the occurrences of OR pattern, e.g. `a => b|c` to `a => b` and `a => c`
    fn expand_or(&self) -> Vec<Expr> {
        use Expr::*;
        let mut acc = Vec::new();
        match self {
            Cons(lhs, rhs) => {
                for a in lhs.expand_or() {
                    for b in rhs.expand_or() {
                        acc.push(Cons(Rc::new(a.clone()), Rc::new(b.clone())));
                    }
                }
            }
            Infix(op, lhs, rhs) => {
                for a in lhs.expand_or() {
                    for b in rhs.expand_or() {
                        acc.push(Infix(*op, Rc::new(a.clone()), Rc::new(b.clone())));
                    }
                }
            }
            Not(expr) => {
                for e in expr.expand_or() {
                    acc.push(e);
                }
            }
            Or(set) => {
                for s in set
                    .iter()
                    .map(|e| e.expand_or().into_iter())
                    .multi_cartesian_product()
                {
                    for e in s {
                        acc.push(e);
                    }
                }
            }
            Wildcard | Literal(_) | Variable(_, _) => acc.push(self.clone()),
        }
        acc
    }

    fn expand_symmetries(&mut self, symmetries: &Symmetries) {
        use Expr::*;
        match self {
            Cons(head, tail)
                if let Literal(name) = head.as_ref()
                    && let Cons(lhs, rhs) = tail.as_ref() =>
            {
                if symmetries.functions.contains(name) {
                    *self = Or(vec![
                        Cons(
                            Rc::clone(head),
                            Rc::new(Cons(Rc::clone(lhs), Rc::clone(rhs))),
                        ),
                        Cons(
                            Rc::clone(head),
                            Rc::new(Cons(Rc::clone(rhs), Rc::clone(lhs))),
                        ),
                    ])
                }
            }
            Infix(op, lhs, rhs) => {
                if symmetries.operators.contains(op) {
                    *self = Or(vec![
                        Infix(*op, Rc::clone(lhs), Rc::clone(rhs)),
                        Infix(*op, Rc::clone(rhs), Rc::clone(lhs)),
                    ])
                }
            }
            Not(expr) => Rc::make_mut(expr).expand_symmetries(symmetries),
            Or(set) => {
                set.iter_mut().for_each(|e| e.expand_symmetries(symmetries));
                flatten_set(set);
            }
            _ => {}
        }
    }
}

fn flatten_set(set: &mut Vec<Expr>) {
    let mut i = 0;
    while i < set.len() {
        if let Expr::Or(inner) = &set[i] {
            for e in inner.clone() {
                if !set.contains(&e) {
                    set.push(e);
                }
            }
            set.remove(i);
        } else {
            i += 1;
        }
    }
}
