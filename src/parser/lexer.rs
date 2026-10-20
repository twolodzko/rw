use super::{
    reader::Reader,
    tokenizer::{Token, Tokenizer},
};
use crate::{
    operators,
    parser::tokenizer::Token::Keyword,
    types::{
        Expr::{self, Cons},
        Statement,
    },
};
use anyhow::{Result, bail};
use std::{collections::VecDeque, str::FromStr};

pub struct Parser<R: Reader> {
    tokenizer: Tokenizer<R>,
    buffer: VecDeque<Statement>,
}

impl<R: Reader> Parser<R> {
    pub(crate) fn new(tokenizer: Tokenizer<R>) -> Parser<R> {
        Parser {
            tokenizer,
            buffer: VecDeque::new(),
        }
    }

    fn statement(&mut self) -> Result<Option<Statement>> {
        if let Some(s) = self.buffer.pop_front() {
            return Ok(Some(s));
        }

        macro_rules! next {
            ( $t:ident ) => {
                if let Some(t) = self.tokenizer.next()? {
                    if let $t(s) = t {
                        s
                    } else {
                        bail!("unexpected token: {}", t)
                    }
                } else {
                    bail!("unexpected end of input")
                }
            };
        }

        use Statement::*;
        use Token::*;
        loop {
            if let Some(t) = self.tokenizer.next()? {
                if t == Newline {
                    continue;
                }
                if let Keyword(s) = &t {
                    match s.as_str() {
                        "let" => {
                            let lhs = self.expr(false)?;
                            let Some(sep) = self.tokenizer.next()? else {
                                bail!("unexpected end of input")
                            };
                            match sep {
                                // =>
                                MapsTo => {
                                    let template = self.expr(false)?;
                                    self.end_of_line()?;
                                    return Ok(Some(LetMap(lhs, template)));
                                }
                                // <=>
                                Equals => {
                                    let rhs = self.expr(false)?;
                                    return Ok(Some(LetEquals(lhs, rhs)));
                                }
                                // {
                                LBracket => {
                                    while let Some(t) = self.tokenizer.peek()? {
                                        match t {
                                            Newline => {
                                                self.tokenizer.skip();
                                                continue;
                                            }
                                            RBracket => {
                                                self.tokenizer.skip();
                                                self.end_of_line()?;
                                                return Ok(self.buffer.pop_front());
                                            }
                                            _ => {}
                                        }

                                        let tail = self.expr(false)?;
                                        let lhs = Cons(Box::new(lhs.clone()), Box::new(tail));

                                        let Some(t) = self.tokenizer.next()? else {
                                            bail!("unexpected end of input")
                                        };
                                        match t {
                                            MapsTo => {
                                                let rhs = self.expr(false)?;
                                                self.buffer.push_back(LetMap(lhs, rhs));
                                            }
                                            Equals => {
                                                let rhs = self.expr(false)?;
                                                self.buffer.push_back(LetEquals(lhs, rhs));
                                            }
                                            _ => bail!("unexpected token: {}", t),
                                        }
                                    }
                                    bail!("unexpected end of input")
                                }
                                // invalid
                                other => bail!("unexpected token: {}", other),
                            }
                        }
                        "rewrite" => {
                            let expr = self.expr(false)?;
                            self.end_of_line()?;
                            return Ok(Some(Rewrite(expr)));
                        }
                        "infix" => {
                            let name = next!(Keyword);
                            let prec = u8::from_str(&next!(Number))?;
                            self.end_of_line()?;
                            return Ok(Some(operators::new(name.to_string(), prec)));
                        }
                        "load" => {
                            if let Some(String(path)) = self.tokenizer.next()? {
                                return Ok(Some(Load(path)));
                            } else {
                                bail!("invalid path")
                            }
                        }
                        _ => {}
                    }
                }
                bail!("unexpected token: {}", t)
            } else {
                return Ok(None);
            }
        }
    }

    fn expr(&mut self, ignore_newline: bool) -> Result<Expr> {
        self.expr_bp(0, ignore_newline)
    }

    fn expr_bp(&mut self, min_bp: u8, ignore_newline: bool) -> Result<Expr> {
        use Token::*;
        // Pratt parser
        // https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
        let mut lhs = self.cons()?;

        loop {
            let (op, bp) = match self.tokenizer.peek()? {
                Some(Keyword(k)) if let Some(v) = operators::get(k) => v,
                Some(Newline) if ignore_newline => continue,
                _ => break,
            };
            if bp < min_bp {
                break;
            }
            self.tokenizer.skip();

            let rhs = self.expr_bp(bp, ignore_newline)?;
            lhs = Expr::Infix(op, Box::new(lhs), Box::new(rhs));
        }

        Ok(lhs)
    }

    fn cons(&mut self) -> Result<Expr> {
        let head = self.atom()?;
        if let Some(tail) = self.tokenizer.peek()?
            && !tail.is_infix()
            && !tail.is_boundary()
        {
            let tail = self.cons()?;
            return Ok(Cons(Box::new(head), Box::new(tail)));
        }
        Ok(head)
    }

    fn atom(&mut self) -> Result<Expr> {
        use Token::*;
        if let Some(t) = self.tokenizer.next()? {
            match t {
                Keyword(s) | Number(s) | String(s) => return Ok(Expr::Literal(s)),
                Wildcard => return Ok(Expr::Wildcard),
                Variable(s) => {
                    let cond = if let Some(Colon) = self.tokenizer.peek()? {
                        self.tokenizer.skip();
                        self.atom()?
                    } else {
                        Expr::Wildcard
                    };
                    return Ok(Expr::Variable(s, Box::new(cond)));
                }
                Not => {
                    let expr = self.atom()?;
                    if let Expr::Not(inner) = expr {
                        return Ok(*inner);
                    }
                    return Ok(Expr::Not(Box::new(expr)));
                }
                LParenth => {
                    let expr = self.expr(true)?;
                    self.expect(RParenth)?;
                    return Ok(expr);
                }
                _ => {}
            }
        }
        bail!("unexpected end of input")
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.tokenizer.next()?.is_some_and(|t| t == expected) {
            return Ok(());
        }
        bail!("missing {}", expected)
    }

    fn end_of_line(&mut self) -> Result<()> {
        if let Some(t) = self.tokenizer.next()?
            && !matches!(t, Token::Newline)
        {
            bail!("unexpected token: {}", t)
        }
        Ok(())
    }
}

impl Token {
    fn is_infix(&self) -> bool {
        if let Keyword(s) = self {
            return operators::get(s).is_some();
        }
        false
    }

    fn is_boundary(&self) -> bool {
        use Token::*;
        // "(" is parsed as delimiter of expression
        matches!(
            self,
            Newline | LBracket | RBracket | RParenth | MapsTo | Equals
        )
    }
}

impl<R: Reader> Iterator for Parser<R> {
    type Item = Result<Statement>;

    fn next(&mut self) -> Option<Self::Item> {
        self.statement().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Expr::*,
        Parser,
        Statement::{self, *},
    };
    use anyhow::Result;
    use test_case::test_case;

    #[test_case("let $a => $a", vec![
        LetMap(
            Variable("a".to_string(), Box::new(Wildcard)),
            Variable("a".to_string(), Box::new(Wildcard))),
     ];
     "simple mapping")]
    fn parse(input: &str, expected: Vec<Statement>) {
        let parser = Parser::from(input);
        let result = parser.into_iter().collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(result, expected)
    }
}
