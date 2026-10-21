use crate::{Context, MAX_ITERS, MAX_ITERS_KEY, VERBOSE, queue::Queue, types::Expr};
use std::rc::Rc;

impl Context {
    pub(crate) fn rewrite(&mut self, expr: Expr) -> Expr {
        let mut queue = Queue::default();
        let mut this = Rc::new(expr.clone());
        let mut best: Option<Rc<Expr>> = None;

        let verbose = *VERBOSE;
        let max_iters = *MAX_ITERS;
        let mut iter = 1;

        loop {
            if verbose {
                println!("↳ {iter:}\t[queue: {}]\t{}", queue.len(), this);
            }

            if self.is_final(&this) {
                return Rc::unwrap_or_clone(this);
            }

            for m in &self.rules {
                for s in m.apply(&this) {
                    queue.insert(s);
                }
            }

            if best
                .as_ref()
                .is_none_or(|b| this.complexity() < b.complexity())
            {
                best = Some(Rc::clone(&this));
            }

            if iter >= max_iters {
                eprintln!(
                    "Warning: stopping early because of reaching maximum number of iterations ({MAX_ITERS_KEY}={})",
                    max_iters
                );
                return best.map_or(expr, Rc::unwrap_or_clone);
            }
            iter += 1;

            if let Some(s) = queue.pop() {
                this = s;
            } else {
                return best.map_or(expr, Rc::unwrap_or_clone);
            }
        }
    }

    fn is_final(&self, expr: &Expr) -> bool {
        if let Expr::Literal(key) = expr {
            self.outgoing.get(key).is_none_or(|out| out.is_empty())
        } else {
            false
        }
    }
}
