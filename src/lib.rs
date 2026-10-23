mod context;
mod evaluate;
mod mapping;
mod operators;
mod parser;
mod queue;
mod rewrite;
mod types;

use std::str::FromStr;
use std::sync::LazyLock;

pub use context::Context;
pub use evaluate::{eval, eval_file};
pub use parser::Parser;

pub(crate) const MAX_ITERS_KEY: &str = "RW_MAX_ITERS";

pub(crate) static MAX_ITERS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var(MAX_ITERS_KEY)
        .ok()
        .and_then(|s| usize::from_str(s.as_str()).ok())
        .unwrap_or(100_000)
});

pub(crate) static VERBOSE: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("RW_VERBOSE")
        .ok()
        .and_then(|s| bool::from_str(s.as_str()).ok())
        .unwrap_or_default()
});
