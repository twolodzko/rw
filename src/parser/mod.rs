mod lexer;
mod reader;
mod tokenizer;

use anyhow::Result;
pub use lexer::Parser;
use reader::{FileReader, StdinReader, StringReader};
use std::path::PathBuf;
use tokenizer::Tokenizer;

impl Parser<StdinReader> {
    pub fn from_stdin() -> Result<Self> {
        let reader = StdinReader::new()?;
        let tokenizer = Tokenizer::new(reader);
        Ok(Parser::new(tokenizer))
    }
}

impl From<&str> for Parser<StringReader> {
    fn from(value: &str) -> Self {
        let reader = StringReader::from(value);
        let tokenizer = Tokenizer::new(reader);
        Parser::new(tokenizer)
    }
}

impl TryFrom<&PathBuf> for Parser<FileReader> {
    type Error = anyhow::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let reader = FileReader::try_from(value)?;
        let tokenizer = Tokenizer::<FileReader>::new(reader);
        Ok(Parser::new(tokenizer))
    }
}

impl Parser<FileReader> {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let reader = FileReader::try_from(path)?;
        let tokenizer = Tokenizer::<FileReader>::new(reader);
        Ok(Parser::new(tokenizer))
    }
}
