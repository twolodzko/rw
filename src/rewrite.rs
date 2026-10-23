use crate::{Context, queue::Queue, types::Expr};
use std::rc::Rc;

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
}
