use super::reader::Reader;
use anyhow::{Result, bail};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    /// alphanumerical keyword or operator
    Keyword(String),
    /// numerical keyword
    Number(String),
    /// string
    String(String),
    /// variable name
    Variable(String),
    /// =>
    MapsTo,
    /// :=
    SaveTo,
    /// {
    LBracket,
    /// }
    RBracket,
    /// (
    LParenth,
    /// )
    RParenth,
    /// \n
    Newline,
    /// ;
    Semicolon,
}

pub(crate) struct Tokenizer<R: Reader> {
    reader: R,
    cache: Option<Token>,
}

impl<R: Reader> Tokenizer<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            cache: None,
        }
    }

    pub(crate) fn peek(&mut self) -> Result<Option<&Token>> {
        if self.cache.is_none() {
            self.cache = self.read_token()?;
        }
        Ok(self.cache.as_ref())
    }

    pub(crate) fn next(&mut self) -> Result<Option<Token>> {
        if self.cache.is_some() {
            let t = self.cache.clone();
            self.cache = None;
            return Ok(t);
        }
        self.read_token()
    }

    pub(crate) fn skip(&mut self) {
        let _ = self.next();
    }

    pub(crate) fn read_token(&mut self) -> Result<Option<Token>> {
        use Token::*;
        while let Some(c) = self.reader.peek()? {
            match c {
                // newline
                '\n' => {
                    self.reader.skip();
                    return Ok(Some(Newline));
                }
                ';' => {
                    self.reader.skip();
                    return Ok(Some(Semicolon));
                }
                // skip whitespace
                c if c.is_whitespace() => self.reader.skip(),
                // skip comment
                '#' => loop {
                    if let None | Some('\n') = self.reader.next()? {
                        break;
                    }
                },
                // proper tokens
                '(' => {
                    self.reader.skip();
                    return Ok(Some(LParenth));
                }
                ')' => {
                    self.reader.skip();
                    return Ok(Some(RParenth));
                }
                '{' => {
                    self.reader.skip();
                    return Ok(Some(LBracket));
                }
                '}' => {
                    self.reader.skip();
                    return Ok(Some(RBracket));
                }
                '"' => {
                    return Ok(Some(String(self.read_string()?)));
                }
                '$' => {
                    self.reader.skip();
                    return Ok(Some(Variable(self.read_name()?)));
                }
                c if c.is_alphabetic() || c == '_' => {
                    return Ok(Some(Keyword(self.read_name()?)));
                }
                c if c.is_numeric() => {
                    return Ok(Some(Number(self.read_number()?)));
                }
                _ => {
                    let s = self.read_symbol_name()?;
                    return Ok(Some(match s.as_str() {
                        "=>" => MapsTo,
                        ":=" => SaveTo,
                        _ => Keyword(s),
                    }));
                }
            }
        }
        Ok(None)
    }

    fn read_name(&mut self) -> Result<String> {
        let mut acc = String::new();
        while let Some(c) = self.reader.peek()? {
            if c.is_alphanumeric() || c == '_' {
                self.reader.skip();
                acc.push(c);
            } else {
                break;
            }
        }
        // allow for primes at the end of the name
        while let Some(c @ '\'') = self.reader.peek()? {
            self.reader.skip();
            acc.push(c);
        }
        debug_assert!(!acc.is_empty());
        Ok(acc)
    }

    fn read_number(&mut self) -> Result<String> {
        let mut acc = String::new();
        while let Some(c) = self.reader.peek()? {
            if c.is_numeric() {
                self.reader.skip();
                acc.push(c);
            } else {
                break;
            }
        }
        debug_assert!(!acc.is_empty());
        Ok(acc)
    }

    fn read_symbol_name(&mut self) -> Result<String> {
        let mut acc = String::new();
        while let Some(c) = self.reader.peek()? {
            if c.is_whitespace() || is_special(c) || c.is_alphanumeric() {
                break;
            } else {
                self.reader.skip();
                acc.push(c);
            }
        }
        debug_assert!(!acc.is_empty());
        Ok(acc)
    }

    fn read_string(&mut self) -> Result<String> {
        let mut acc = String::new();
        self.reader.skip();

        while let Some(c) = self.reader.next()? {
            match c {
                '\\' => {
                    let Some(c) = self.reader.next()? else {
                        break;
                    };
                    match c {
                        '\\' | '"' => acc.push(c),
                        _ => {
                            acc.push('\\');
                            acc.push(c);
                        }
                    }
                }
                '"' => return Ok(acc),
                _ => acc.push(c),
            }
        }
        bail!("missing \" to mark end of the string")
    }
}

fn is_special(c: char) -> bool {
    matches!(c, '(' | ')' | '{' | '}' | '$' | '#' | ';')
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Keyword(s) | Number(s) => write!(f, "{}", s),
            Variable(s) => write!(f, "${}", s),
            String(s) => write!(f, "\"{}\"", s),
            MapsTo => write!(f, "=>"),
            SaveTo => write!(f, ":="),
            LBracket => write!(f, "{{"),
            RBracket => write!(f, "}}"),
            LParenth => write!(f, "("),
            RParenth => write!(f, ")"),
            Newline => write!(f, r"\n"),
            Semicolon => write!(f, ";"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Token::{self, *},
        Tokenizer,
    };
    use crate::parser::reader::StringReader;
    use test_case::test_case;

    #[test_case(
        "\nnot this",
        Newline;
        "newline"
    )]
    #[test_case(
        "(2 + 2)",
        LParenth;
        "left parenthesis"
    )]
    #[test_case(
        "foo+bar",
        Keyword("foo".to_string());
        "keyword"
    )]
    #[test_case(
        "=/=Bar",
        Keyword("=/=".to_string());
        "operator"
    )]
    fn tokenize_one(input: &str, expected: Token) {
        let reader = StringReader::from(input);
        let tokenizer = &mut Tokenizer::<StringReader>::new(reader);
        assert_eq!(tokenizer.read_token().unwrap().unwrap(), expected)
    }

    #[test]
    fn tokenize_many() {
        let reader = StringReader::from("let 2 + 2 = 4\neval 2 + 2");
        let tokenizer = &mut Tokenizer::<StringReader>::new(reader);
        let mut result = Vec::new();
        while let Some(val) = tokenizer.next().unwrap() {
            result.push(val);
        }
        let expected = vec![
            Keyword("let".to_string()),
            Number("2".to_string()),
            Keyword("+".to_string()),
            Number("2".to_string()),
            Keyword("=".to_string()),
            Number("4".to_string()),
            Newline,
            Keyword("eval".to_string()),
            Number("2".to_string()),
            Keyword("+".to_string()),
            Number("2".to_string()),
        ];
        assert_eq!(result, expected)
    }
}
