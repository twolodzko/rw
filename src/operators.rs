use crate::types::Statement;
use std::sync::{LazyLock, Mutex};

/// Name and complexity of the defined operators
static OPERATORS: LazyLock<Mutex<Vec<(String, u8)>>> = LazyLock::new(Default::default);

pub(crate) fn get(name: &str) -> Option<(usize, u8)> {
    OPERATORS
        .lock()
        .unwrap()
        .iter()
        .enumerate()
        .find_map(|(i, (n, p))| if n == name { Some((i, *p)) } else { None })
}

/// Register a new infix operator (or alter precedence) and return a parsed statement for bookkeeping
pub(crate) fn new(name: String, precedence: u8) -> Statement {
    let mut op = OPERATORS.lock().unwrap();
    if let Some((_, p)) = &mut op.iter_mut().find(|(n, _)| n == &name) {
        *p = precedence;
    } else {
        op.push((name.to_owned(), precedence));
    }
    Statement::NewOp(name, precedence)
}

pub(crate) fn name(id: usize) -> String {
    OPERATORS.lock().unwrap()[id].0.to_string()
}
