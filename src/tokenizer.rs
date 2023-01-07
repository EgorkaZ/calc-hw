use std::fmt::Display;

use crate::tokens;

#[derive(Debug)]
pub struct Tokenizer<'s> {
    input: &'s str,
    state: State,
}

pub fn tokenize(input: &str) -> Tokenizer {
    Tokenizer {
        input,
        state: State::General(GeneralState {}),
    }
}

#[derive(Debug)]
pub struct TokenizeError {
    msg: &'static str,
    at: usize,
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TokenizeError: {}", self.msg))
    }
}

impl std::error::Error for TokenizeError {}

impl Iterator for Tokenizer<'_> {
    type Item = Result<tokens::Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (outcome, to_skip) = self.state.process(self.input);
            self.input = &self.input[to_skip..];
            match outcome {
                Outcome::Token(tok) => break Some(Ok(tok)),
                Outcome::State(ns) => self.state = ns,
                Outcome::Done => break None,
                Outcome::Error(err) => break Some(Err(err)),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Number(NumberState),
    General(GeneralState),
}

#[derive(Debug)]
enum Outcome {
    Token(tokens::Token),
    State(State),
    Done,
    Error(TokenizeError),
}

fn try_parse_oper(part: char) -> Option<tokens::Operation> {
    match part {
        '+' => Some(tokens::Operation::Add),
        '-' => Some(tokens::Operation::Sub),
        '*' => Some(tokens::Operation::Mul),
        '/' => Some(tokens::Operation::Div),
        _ => None,
    }
}

fn is_oper(part: char) -> bool {
    try_parse_oper(part).is_some()
}

#[derive(Debug, Default, Clone, Copy)]
struct NumberState {}

trait ParseStep {
    fn process(&mut self, s: &str) -> (Outcome, usize);
}

impl ParseStep for NumberState {
    fn process(&mut self, s: &str) -> (Outcome, usize) {
        let mut chars = s.chars();
        let mut to_skip = 0;
        while let Some(ch) = chars.next() {
            if !ch.is_digit(10) {
                break;
            }
            to_skip += 1;
        }

        if to_skip != 0 {
            (
                Outcome::Token(tokens::Token::Number(tokens::Number(
                    s[..to_skip].parse().expect("Empty number somehow"),
                ))),
                to_skip,
            )
        } else {
            (Outcome::State(State::General(GeneralState {})), 0)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GeneralState {}

impl ParseStep for GeneralState {
    fn process(&mut self, s: &str) -> (Outcome, usize) {
        let mut chars = s.chars();
        let mut to_skip = 0;
        loop {
            match chars.next() {
                Some(ch) if ch.is_digit(10) => {
                    break (
                        Outcome::State(State::Number(NumberState::default())),
                        to_skip,
                    )
                }
                Some(ch) if ch.is_whitespace() => {
                    to_skip += 1;
                }
                Some(ch) if is_oper(ch) => {
                    break (
                        Outcome::Token(tokens::Token::Oper(try_parse_oper(ch).unwrap())),
                        to_skip + 1,
                    )
                }
                Some(ch) if ch == '(' || ch == ')' => {
                    break (
                        Outcome::Token(tokens::Token::Brace(if ch == '(' {
                            tokens::Brace::Left
                        } else {
                            tokens::Brace::Right
                        })),
                        to_skip + 1,
                    )
                }
                Some(_) => {
                    break (
                        Outcome::Error(TokenizeError {
                            msg: "invalid symbol",
                            at: to_skip,
                        }),
                        0,
                    )
                }
                None => break (Outcome::Done, to_skip),
            }
        }
    }
}

impl ParseStep for State {
    fn process(&mut self, s: &str) -> (Outcome, usize) {
        match self {
            State::Number(ns) => ns.process(s),
            State::General(gs) => gs.process(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens;

    use super::{tokenize, TokenizeError};

    #[test]
    fn number() -> Result<(), TokenizeError> {
        let s = "123";
        let res: Vec<_> = tokenize(s).collect::<Result<_, _>>()?;
        [tokens::Token::Number(tokens::Number(123))]
            .into_iter()
            .zip(res)
            .for_each(|(l, r)| assert_eq!(l, r));
        Ok(())
    }

    #[test]
    fn number_and_op() -> Result<(), TokenizeError> {
        let res: Vec<_> = tokenize("123 + (").collect::<Result<_, _>>()?;
        [
            tokens::Token::Number(tokens::Number(123)),
            tokens::Token::Oper(tokens::Operation::Add),
            tokens::Token::Brace(tokens::Brace::Left),
        ]
        .into_iter()
        .zip(res)
        .for_each(|(l, r)| assert_eq!(l, r));
        Ok(())
    }
}
