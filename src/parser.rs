use std::fmt::{Debug, Display};

use crate::{tokens, TokenVisitor, tokenizer::{TokenizeError, Tokenizer, tokenize}};

pub struct Parser<I> {
    inner: I,
    stack: Vec<Stacked>,
    curr: Option<tokens::Token>,
    state: State,
    /// +1 on argument, -1 on operator, can't be out of [0, 1] for valid infix string
    arg_balance: i8,
    par_balance: i32,
}

pub fn parse(input: &str) -> Parser<Tokenizer> {
    Parser::new(tokenize(input))
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Tokenization(TokenizeError),
    UnmatchedParens,
    NotEnoughArgs,
    NotEnoughOps,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Tokenization(tok_err) => f.write_fmt(format_args!("TokenizeError: {tok_err}")),
            ParseError::UnmatchedParens => f.write_str("unmatched parens"),
            ParseError::NotEnoughArgs => f.write_str("got operators without arguments"),
            ParseError::NotEnoughOps => f.write_str("got arguments without operator"),
        }
    }
}

impl std::error::Error for ParseError {}

impl<I> Parser<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            stack: vec![],
            curr: None,
            state: State::Skip,
            arg_balance: 0,
            par_balance: 0,
        }
    }

    fn token_from_state(&mut self) -> Option<Result<tokens::Token, ParseError>> {
        let mb_token = match self.state {
            State::PopParenLevel => match self.stack.pop() {
                Some(Stacked::LBrace) => { self.curr = None; self.state = State::Skip; None },
                Some(Stacked::Op(op)) => {
                    Some(tokens::Token::Oper(op))
                },
                None => return Some(Err(ParseError::UnmatchedParens)),
            },
            State::PopOp => match self.stack.pop() {
                Some(Stacked::Op(op)) => {
                    Some(tokens::Token::Oper(op))
                },
                _ => unreachable!(),
            },
            State::Skip => { self.curr = None; None },
            State::CurrToOut => {
                self.curr.take()
            },
        };

        if let Some(tokens::Token::Oper(_)) = self.curr {
            // will be reused on next turn, have to keep arg balance
            self.arg_balance += 1;
        }

        mb_token.map(Ok)
        // mb_token.map(|token| if self.arg_balance <= 0 {
        //     Err(ParseError::NotEnoughArgs)
        // } else {
        //     Ok(token)
        // })
    }
}

impl<I> Debug for Parser<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("stack", &self.stack)
            .field("curr", &self.curr)
            .field("state", &self.state)
            .field("arg_balance", &self.arg_balance)
            .field("par_balance", &self.par_balance)
            .finish()
    }
}

#[derive(Debug)]
enum Stacked {
    Op(tokens::Operation),
    LBrace,
}

#[derive(Debug)]
enum State {
    PopParenLevel,
    PopOp,
    Skip,
    CurrToOut,
}

impl<I> Iterator for Parser<I>
where
    I: Iterator<Item = Result<tokens::Token, TokenizeError>>,
{
    type Item = Result<tokens::Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr = {
                match self.curr {
                    Some(curr) => curr,
                    None => match self.inner.next() {
                        Some(Err(tok_err)) => return Some(Err(ParseError::Tokenization(tok_err))),
                        Some(Ok(curr)) => {
                            self.curr = Some(curr);
                            curr
                        }
                        None if self.stack.is_empty() => {
                            if self.par_balance != 0 {
                                // panic!("Unmatched parens, state: {self:?}");
                                return Some(Err(ParseError::UnmatchedParens));
                            }
                            return None
                        },
                        None => {
                            self.state = State::PopParenLevel;
                            if self.par_balance != 0 {
                                // panic!("Non-matched parens, state: {self:?}");
                                return Some(Err(ParseError::UnmatchedParens))
                            }
                            return self.token_from_state();
                        }
                    },
                }
            };

            self.visit_token(curr);
            if self.par_balance < 0 {
                // panic!("Went under 0 paren balance: {self:?}");
                return Some(Err(ParseError::UnmatchedParens))
            } else if self.arg_balance < 0 {
                return Some(Err(ParseError::NotEnoughArgs))
            } else if self.arg_balance > 1 {
                return Some(Err(ParseError::NotEnoughOps))
            }

            if let Some(token) = self.token_from_state() {
                break Some(token);
            }
        }
    }
}

impl<I> TokenVisitor for Parser<I> {
    fn visit_paren(&mut self, paren: tokens::Paren) {
        match paren {
            tokens::Paren::Left => {
                self.stack.push(Stacked::LBrace);
                self.state = State::Skip;
                self.par_balance += 1;
            }
            tokens::Paren::Right => match self.state {
                State::PopParenLevel => (),
                _ => {
                    self.par_balance -= 1;
                    self.state = State::PopParenLevel;
                },
            }
        }
    }

    fn visit_op(&mut self, op: tokens::Operation) {
        self.arg_balance -= 1;
        match self.stack.last() {
            Some(Stacked::LBrace) | None => {
                self.stack.push(Stacked::Op(op));
                self.state = State::Skip;
            }
            Some(Stacked::Op(stack_op)) if stack_op.prio() < op.prio() => {
                self.stack.push(Stacked::Op(op));
                self.state = State::Skip;
            }
            Some(Stacked::Op(_)) => {
                self.state = State::PopOp;
            }
        }
    }

    fn visit_num(&mut self, _num: tokens::Number) {
        self.arg_balance += 1;
        self.state = State::CurrToOut;
    }
}
