mod evaluate;
mod mapping;
mod operators;
mod parser;
mod queue;
mod rewrite;
mod types;

use std::{cell::LazyCell, path::PathBuf};

pub use evaluate::{eval, eval_file};
pub use parser::Parser;
pub use rewrite::Context;

pub static mut WORKING_DIRECTORY: LazyCell<Vec<PathBuf>> = LazyCell::new(Vec::new);
