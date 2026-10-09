use anyhow::{Result, bail};
use rustyline::{Config, DefaultEditor, error::ReadlineError};
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    iter::Peekable,
    path::PathBuf,
    vec::IntoIter,
};

pub trait Reader {
    fn next(&mut self) -> Result<Option<char>>;
    fn peek(&mut self) -> Result<Option<char>>;

    fn skip(&mut self) {
        self.next().unwrap();
    }
}

pub struct StringReader(Peekable<IntoIter<char>>);

impl StringReader {
    fn empty() -> Self {
        StringReader::from("")
    }
}

impl From<&str> for StringReader {
    fn from(value: &str) -> Self {
        StringReader(value.chars().collect::<Vec<char>>().into_iter().peekable())
    }
}

impl From<&String> for StringReader {
    fn from(value: &String) -> Self {
        StringReader(value.chars().collect::<Vec<char>>().into_iter().peekable())
    }
}

impl Reader for StringReader {
    fn next(&mut self) -> Result<Option<char>> {
        Ok(self.0.next())
    }

    fn peek(&mut self) -> Result<Option<char>> {
        Ok(self.0.peek().cloned())
    }
}

pub struct FileReader {
    file: Lines<BufReader<File>>,
    buffer: StringReader,
}

impl TryFrom<&str> for FileReader {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let file = BufReader::new(File::open(value)?).lines();
        let chars = StringReader::from("");
        Ok(FileReader {
            file,
            buffer: chars,
        })
    }
}

impl TryFrom<&PathBuf> for FileReader {
    type Error = anyhow::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let file = BufReader::new(File::open(value)?).lines();
        let chars = StringReader::from("");
        Ok(FileReader {
            file,
            buffer: chars,
        })
    }
}

impl Reader for FileReader {
    fn next(&mut self) -> Result<Option<char>> {
        loop {
            if let c @ Some(_) = self.buffer.next()? {
                return Ok(c);
            }
            if !self.next_line()? {
                return Ok(None);
            }
        }
    }

    fn peek(&mut self) -> Result<Option<char>> {
        loop {
            if let c @ Some(_) = self.buffer.peek()? {
                return Ok(c);
            }
            if !self.next_line()? {
                return Ok(None);
            }
        }
    }
}

impl FileReader {
    fn next_line(&mut self) -> Result<bool> {
        if let Some(res) = self.file.next() {
            let mut line = res?;
            line.push('\n');
            self.buffer = StringReader::from(line.as_str());
            return Ok(true);
        }
        Ok(false)
    }
}

pub struct StdinReader {
    reader: DefaultEditor,
    buffer: StringReader,
    brackets: usize,
    parentheses: usize,
}

impl StdinReader {
    pub fn new() -> Result<Self> {
        let config = Config::builder().auto_add_history(true).build();
        let reader = DefaultEditor::with_config(config)?;
        let reader = StdinReader {
            reader,
            buffer: StringReader::empty(),
            brackets: 0,
            parentheses: 0,
        };
        Ok(reader)
    }

    fn next_line(&mut self) -> Result<bool> {
        let prompt = if self.brackets > 0 || self.parentheses > 0 {
            "  "
        } else {
            "> "
        };

        match self.reader.readline(prompt) {
            Ok(line) => {
                for c in line.chars() {
                    self.parentheses = match c {
                        '(' => self.parentheses + 1,
                        ')' => self.parentheses.saturating_sub(1),
                        '{' => self.brackets + 1,
                        '}' => self.brackets.saturating_sub(1),
                        _ => continue,
                    };
                }
                self.buffer = StringReader::from(&format!("{}\n", line));
                Ok(true)
            }
            Err(ReadlineError::Eof) => Ok(false),
            Err(err) => bail!(err),
        }
    }
}

impl Reader for StdinReader {
    fn next(&mut self) -> Result<Option<char>> {
        loop {
            if let c @ Some(_) = self.buffer.next()? {
                return Ok(c);
            }
            if !self.next_line()? {
                return Ok(None);
            }
        }
    }

    fn peek(&mut self) -> Result<Option<char>> {
        loop {
            if let c @ Some(_) = self.buffer.peek()? {
                return Ok(c);
            }
            if !self.next_line()? {
                return Ok(None);
            }
        }
    }
}
