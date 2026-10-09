use super::{
    reader::Reader,
    tokenizer::{Token, Tokenizer},
};
use crate::{
    mapping::Mapping,
    operators::Op,
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
                            let expr = self.expr(false)?;
                            match self.tokenizer.peek()? {
                                // =>
                                Some(MapsTo) => {
                                    self.tokenizer.skip();
                                    let template = self.expr(false)?;
                                    self.end_of_line()?;
                                    let m = Mapping::new(expr, template)?;
                                    return Ok(Some(LetMap(m)));
                                }
                                // :=
                                Some(SaveTo) => {
                                    self.tokenizer.skip();
                                    if let Expr::Literal(key) = expr {
                                        let expr = self.expr(false)?;
                                        self.end_of_line()?;
                                        return Ok(Some(EvalTo(key, expr)));
                                    } else {
                                        bail!("left hand side of := needs to be a literal")
                                    }
                                }
                                // {
                                Some(LBracket) => {
                                    self.tokenizer.skip();
                                    while let Some(t) = self.tokenizer.peek()? {
                                        if let Newline = t {
                                            self.tokenizer.skip();
                                            continue;
                                        }

                                        let tail = self.expr(false)?;
                                        self.expect(MapsTo)?;
                                        let rhs = self.expr(false)?;
                                        let lhs = Cons(Box::new(expr.clone()), Box::new(tail));
                                        let m = Mapping::new(lhs, rhs)?;
                                        self.buffer.push_back(LetMap(m));

                                        if let Some(Semicolon) = self.tokenizer.peek()? {
                                            self.tokenizer.skip();
                                        }
                                        while let Some(t) = self.tokenizer.peek()? {
                                            match t {
                                                Newline => self.tokenizer.skip(),
                                                RBracket => {
                                                    self.tokenizer.skip();
                                                    self.end_of_line()?;
                                                    return Ok(self.buffer.pop_front());
                                                }
                                                _ => break,
                                            }
                                        }
                                    }
                                    bail!("unexpected end of input")
                                }
                                // boolean let
                                _ => {
                                    self.end_of_line()?;
                                    return Ok(Some(Let(expr)));
                                }
                            }
                        }
                        "eval" => {
                            let expr = self.expr(false)?;
                            self.end_of_line()?;
                            return Ok(Some(Eval(expr)));
                        }
                        "assert" => {
                            let expr = self.expr(false)?;
                            self.end_of_line()?;
                            return Ok(Some(Assert(expr)));
                        }
                        "prefix" => {
                            let name = next!(Keyword);
                            let prec = u8::from_str(&next!(Number))?;
                            self.end_of_line()?;
                            return Ok(Some(Op::new_prefix(name.to_string(), prec)));
                        }
                        "infix" => {
                            let name = next!(Keyword);
                            let prec = u8::from_str(&next!(Number))?;
                            self.end_of_line()?;
                            return Ok(Some(Op::new_infix(name.to_string(), prec)));
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
        let mut lhs = if let Some(Keyword(s)) = self.tokenizer.peek()?
            && let Some((op, bp)) = Op::get_prefix(s)
        {
            self.tokenizer.skip();
            // TODO: is this needed? Prefix operator always has precedence (?)
            let rhs = self.expr_bp(bp, ignore_newline)?;
            Expr::Prefix(op, Box::new(rhs))
        } else {
            self.cons()?
        };

        loop {
            let (op, bp) = match self.tokenizer.peek()? {
                Some(Keyword(k)) if let Some(v) = Op::get_infix(k) => v,
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
                Keyword(s) => {
                    return match s.as_str() {
                        "true" => Ok(Expr::Bool(true)),
                        "false" => Ok(Expr::Bool(false)),
                        _ => Ok(Expr::Literal(s)),
                    };
                }
                Number(s) => return Ok(Expr::Literal(s)),
                Variable(s) => return Ok(Expr::Variable(s)),
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
            && !matches!(t, Token::Newline | Token::Semicolon)
        {
            bail!("unexpected token: {}", t)
        }
        Ok(())
    }
}

impl Token {
    fn is_infix(&self) -> bool {
        if let Keyword(s) = self {
            return Op::get_infix(s).is_some();
        }
        false
    }

    fn is_boundary(&self) -> bool {
        use Token::*;
        // "(" is parsed as delimiter of expression
        matches!(
            self,
            Newline | Semicolon | LBracket | RBracket | RParenth | MapsTo | SaveTo
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
    use crate::mapping::Mapping;

    use super::{
        Expr::*,
        Op, Parser,
        Statement::{self, *},
    };
    use anyhow::Result;
    use test_case::test_case;

    #[test_case("let a -> b and c", vec![
        Let(Infix(
            Op::Imply,
            Box::new(Literal("a".into())),
            Box::new(Infix(
                Op::And,
                Box::new(Literal("b".into())),
                Box::new(Literal("c".into())),
            )))),
     ];
     "simple let")]
    #[test_case("let $a => $a", vec![
        LetMap(Mapping::new(
            Variable("a".to_string()),
            Variable("a".to_string())).unwrap()),
     ];
     "simple mapping")]
    fn parse(input: &str, expected: Vec<Statement>) {
        let parser = Parser::from(input);
        let result = parser.into_iter().collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(result, expected)
    }
}
