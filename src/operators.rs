use crate::types::Statement;
use std::sync::{LazyLock, Mutex};

static PREFIX: LazyLock<Mutex<Vec<(String, u8)>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static INFIX: LazyLock<Mutex<Vec<(String, u8)>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Equals,
    Not,
    And,
    Or,
    Imply,
    Prefix(usize),
    Infix(usize),
}

impl Op {
    pub(crate) fn get_prefix(name: &str) -> Option<(Op, u8)> {
        use Op::*;
        let val = match name {
            "not" => (Not, 200),
            _ => {
                return PREFIX
                    .lock()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .find_map(|(i, (n, p))| {
                        if n == name {
                            Some((Op::Prefix(i), *p))
                        } else {
                            None
                        }
                    });
            }
        };
        Some(val)
    }

    pub(crate) fn get_infix(name: &str) -> Option<(Op, u8)> {
        use Op::*;
        let val = match name {
            "=" => (Equals, 0),
            "->" => (Imply, 10),
            "or" => (Or, 20),
            "and" => (And, 30),
            _ => {
                return INFIX
                    .lock()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .find_map(|(i, (n, p))| {
                        if n == name {
                            Some((Op::Infix(i), *p))
                        } else {
                            None
                        }
                    });
            }
        };
        Some(val)
    }

    /// Register a new prefix operator (or alter precedence) and return a parsed statement for bookkeeping
    pub(crate) fn new_prefix(name: String, precedence: u8) -> Statement {
        if let Ok(ref mut ops) = PREFIX.lock() {
            if let Some((_, p)) = ops.iter_mut().find(|(n, _)| n == &name) {
                *p = precedence;
            } else {
                ops.push((name.to_owned(), precedence));
            }
            Statement::NewPrefix(name, precedence)
        } else {
            unreachable!()
        }
    }

    /// Register a new infix operator (or alter precedence) and return a parsed statement for bookkeeping
    pub(crate) fn new_infix(name: String, precedence: u8) -> Statement {
        if let Ok(ref mut ops) = INFIX.lock() {
            if let Some((_, p)) = ops.iter_mut().find(|(n, _)| n == &name) {
                *p = precedence;
            } else {
                ops.push((name.to_owned(), precedence));
            }
            Statement::NewInfix(name, precedence)
        } else {
            unreachable!()
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Op::*;
        match self {
            Equals => write!(f, "="),
            Not => write!(f, "not"),
            And => write!(f, "and"),
            Or => write!(f, "or"),
            Imply => write!(f, "->"),
            Prefix(i) => {
                write!(f, "{}", PREFIX.lock().unwrap()[*i].0)
            }
            Infix(i) => {
                write!(f, "{}", INFIX.lock().unwrap()[*i].0)
            }
        }
    }
}
