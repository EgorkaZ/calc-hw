use std::fmt::Display;

use tokens::{Number, Operation, Paren, Token};

pub mod parser;
pub mod tokenizer;
pub mod tokens;

pub trait TokenVisitor {
    fn visit_paren(&mut self, paren: Paren);
    fn visit_op(&mut self, op: Operation);
    fn visit_num(&mut self, num: Number);

    fn visit_token(&mut self, tok: Token) {
        match tok {
            Token::Number(num) => self.visit_num(num),
            Token::Paren(paren) => self.visit_paren(paren),
            Token::Oper(op) => self.visit_op(op),
        }
    }
}

#[derive(Debug)]
struct Calculator {
    stack: Vec<i64>,
}

impl Calculator {
    fn new() -> Self {
        Self { stack: vec![] }
    }

    fn calculate<I: Iterator<Item = Token>>(&mut self, mut iter: I) -> i64 {
        while let Some(tok) = iter.next() {
            self.visit_token(tok);
        }
        if self.stack.is_empty() {
            0
        } else if self.stack.len() > 1 {
            panic!("Not all arguments have corresponding operators")
        } else {
            self.stack.pop().unwrap()
        }
    }
}

pub fn calculate<I: Iterator<Item = Token>>(iter: I) -> i64 {
    Calculator::new().calculate(iter)
}

impl TokenVisitor for Calculator {
    fn visit_paren(&mut self, _par: Paren) {
        panic!("Calculator should not face parens, use pareser first")
    }

    fn visit_op(&mut self, op: Operation) {
        if let (Some(rhs), Some(lhs)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(op.apply(lhs, rhs))
        } else {
            panic!("Insufficient arguments for operation")
        }
    }

    fn visit_num(&mut self, Number(num): Number) {
        self.stack.push(num);
    }
}

#[derive(Debug)]
pub struct Printer<'t>(pub &'t[Token]);

impl Display for Printer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let toks = self.0;

        if toks.is_empty() {
            return Ok(());
        }

        let (fst, rest) = (toks[0], &toks[1..]);
        f.write_fmt(format_args!("{fst}"))?;
        for tok in rest {
            f.write_fmt(format_args!(" {tok}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        calculate,
        parser::{ParseError, Parser},
        tokenizer::{tokenize, TokenizeError},
        tokens::{Number, Operation, Paren, Token},
    };

    fn num(num: i64) -> Token {
        Token::Number(Number(num))
    }

    fn op(op: char) -> Token {
        let op = match op {
            '+' => Operation::Add,
            '-' => Operation::Sub,
            '*' => Operation::Mul,
            '/' => Operation::Div,
            _ => panic!("Unsupported oper shortcut"),
        };

        Token::Oper(op)
    }

    const LPAR: Token = Token::Paren(Paren::Left);
    const RPAR: Token = Token::Paren(Paren::Right);

    #[test]
    fn one_op() {
        test(
            "1 + 2",
            vec![num(1), op('+'), num(2)],
            vec![num(1), num(2), op('+')],
            3,
        );
        test(
            "1 * 2",
            vec![num(1), op('*'), num(2)],
            vec![num(1), num(2), op('*')],
            2,
        );
    }

    #[test]
    fn more_ops() {
        test(
            "1 + 2 * 3",
            vec![num(1), op('+'), num(2), op('*'), num(3)],
            vec![num(1), num(2), num(3), op('*'), op('+')],
            7,
        );
        test(
            "1 * 2 - 3 * 4",
            vec![num(1), op('*'), num(2), op('-'), num(3), op('*'), num(4)],
            vec![num(1), num(2), op('*'), num(3), num(4), op('*'), op('-')],
            -10,
        );
        test(
            "1 * 2 / 5 - 3 * 4",
            vec![
                num(1),
                op('*'),
                num(2),
                op('/'),
                num(5),
                op('-'),
                num(3),
                op('*'),
                num(4),
            ],
            vec![
                num(1),
                num(2),
                op('*'),
                num(5),
                op('/'),
                num(3),
                num(4),
                op('*'),
                op('-'),
            ],
            -12,
        );
    }

    #[test]
    fn parens() {
        test(
            "1 * (2 + 3)",
            vec![num(1), op('*'), LPAR, num(2), op('+'), num(3), RPAR],
            vec![num(1), num(2), num(3), op('+'), op('*')],
            5,
        );
        test(
            "1 + 2 * 3 / (4 - 5) * 6",
            vec![
                num(1),
                op('+'),
                num(2),
                op('*'),
                num(3),
                op('/'),
                LPAR,
                num(4),
                op('-'),
                num(5),
                RPAR,
                op('*'),
                num(6),
            ],
            vec![
                num(1),
                num(2),
                num(3),
                op('*'),
                num(4),
                num(5),
                op('-'),
                op('/'),
                num(6),
                op('*'),
                op('+'),
            ],
            -35,
        )
    }

    #[test]
    fn failures() {
        test_fallible(
            "1 2",
            vec![num(1), num(2)],
            Err(ParseError::NotEnoughOps),
            0,
        );
        test_fallible(
            "1 +- 2",
            vec![num(1), op('+'), op('-'), num(2)],
            Err(ParseError::NotEnoughArgs),
            0,
        );
        test_fallible(
            "(1 + 2))",
            vec![LPAR, num(1), op('+'), num(2), RPAR, RPAR],
            Err(ParseError::UnmatchedParens),
            0,
        );
        test_fallible(
            "((1 + 2",
            vec![LPAR, LPAR, num(1), op('+'), num(2)],
            Err(ParseError::UnmatchedParens),
            0,
        );
        test_fallible(
            "a + b",
            vec![],
            Err(ParseError::Tokenization(TokenizeError::invalid_symbol(0))),
            0,
        );
    }

    fn test(input: &str, after_tokenize: Vec<Token>, after_parse: Vec<Token>, expected: i64) {
        test_fallible(input, after_tokenize, Ok(after_parse), expected);
    }

    fn test_fallible(
        input: &str,
        after_tokenize: Vec<Token>,
        after_parse: Result<Vec<Token>, ParseError>,
        expected: i64,
    ) {
        let collected: Vec<_> = match tokenize(input).collect() {
            Ok(collected) => collected,
            Err(_) => vec![],
        };

        assert_eq!(collected, after_tokenize);

        let collected = match (
            Parser::new(tokenize(input)).collect::<Result<Vec<_>, _>>(),
            after_parse,
        ) {
            (Ok(collected), Ok(expected)) => {
                assert_eq!(collected, expected);
                collected
            }
            (Err(parse_err), Err(expected)) => {
                assert_eq!(parse_err, expected);
                return;
            }
            (Ok(collected), Err(expected)) => {
                panic!("Unexpectedly parsed \"{input}\": {collected:?}, expected err: {expected}")
            }
            (Err(parse_err), Ok(expected)) => {
                panic!("Couldn't parse \"{input}\": {parse_err}, expected: {expected:?}")
            }
        };

        assert_eq!(calculate(collected.into_iter()), expected);
    }
}
