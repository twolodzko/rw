use crate::types::Statement;
use std::cell::LazyCell;

/// Name and complexity of the defined operators
pub(crate) static mut OPERATORS: LazyCell<Vec<(String, u8)>> = LazyCell::new(Vec::new);

pub(crate) fn get(name: &str) -> Option<(usize, u8)> {
    #[allow(static_mut_refs)]
    unsafe {
        OPERATORS
            .iter()
            .enumerate()
            .find_map(|(i, (n, p))| if n == name { Some((i, *p)) } else { None })
    }
}

/// Register a new infix operator (or alter precedence) and return a parsed statement for bookkeeping
pub(crate) fn new(name: String, precedence: u8) -> Statement {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some((_, p)) = &mut OPERATORS.iter_mut().find(|(n, _)| n == &name) {
            *p = precedence;
        } else {
            OPERATORS.push((name.to_owned(), precedence));
        }
    }
    Statement::NewOp(name, precedence)
}

pub(crate) fn name<'a>(id: usize) -> &'a str {
    unsafe { &OPERATORS[id].0 }
}
