use crate::types::Expr;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashSet},
    rc::Rc,
};

#[derive(Eq)]
struct Candidate {
    value: Rc<Expr>,
    complexity: usize,
    // order is used for making the queue deterministic
    order: usize,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        (self.complexity, self.order) == (other.complexity, other.order)
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.complexity, self.order).cmp(&(other.complexity, other.order))
    }
}

#[derive(Default)]
pub(crate) struct Queue {
    items: BinaryHeap<Reverse<Candidate>>,
    seen: HashSet<Rc<Expr>>,
    count: usize,
}

impl Queue {
    pub(crate) fn insert(&mut self, value: Expr) {
        if self.seen.contains(&value) {
            return;
        }
        let value = Rc::new(value);
        self.seen.insert(value.clone());
        self.items.push(Reverse(Candidate {
            complexity: value.complexity(),
            order: self.count,
            value,
        }));
        self.count += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<Rc<Expr>> {
        self.items.pop().map(|Reverse(c)| c.value)
    }
}
