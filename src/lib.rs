use tokens::{Paren, Number, Operation};

pub mod parser;
pub mod tokenizer;
pub mod tokens;

pub trait TokenVisitor {
    fn visit_paren(&mut self, brace: Paren);
    fn visit_op(&mut self, op: Operation);
    fn visit_num(&mut self, num: Number);
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{Parser, ParseError},
        tokenizer::{tokenize, TokenizeError},
        tokens::{Paren, Number, Operation, Token},
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
        );
        test(
            "1 * 2",
            vec![num(1), op('*'), num(2)],
            vec![num(1), num(2), op('*')],
        );
    }

    #[test]
    fn more_ops() {
        test(
            "1 + 2 * 3",
            vec![num(1), op('+'), num(2), op('*'), num(3)],
            vec![num(1), num(2), num(3), op('*'), op('+')],
        );
        test(
            "1 * 2 - 3 * 4",
            vec![num(1), op('*'), num(2), op('-'), num(3), op('*'), num(4)],
            vec![num(1), num(2), op('*'), num(3), num(4), op('*'), op('-')],
        );
        test(
            "1 * 2 / 5 - 3 * 4",
            vec![num(1), op('*'), num(2), op('/'), num(5), op('-'), num(3), op('*'), num(4)],
            vec![num(1), num(2), op('*'), num(5), op('/'), num(3), num(4), op('*'), op('-')],
        );
    }

    #[test]
    fn parens() {
        test(
            "1 * (2 + 3)",
            vec![num(1), op('*'), LPAR, num(2), op('+'), num(3), RPAR],
            vec![num(1), num(2), num(3), op('+'), op('*')],
        );
        test(
            "1 + 2 * 3 / (4 - 5) * 6",
            vec![num(1), op('+'), num(2), op('*'), num(3), op('/'), LPAR, num(4), op('-'), num(5), RPAR, op('*'), num(6)],
            vec![num(1), num(2), num(3), op('*'), num(4), num(5), op('-'), op('/'), num(6), op('*'), op('+')]
        )
    }

    #[test]
    fn failures() {
        test_fallible("1 2", vec![num(1), num(2)], Err(ParseError::NotEnoughOps));
        test_fallible("1 +- 2", vec![num(1), op('+'), op('-'), num(2)], Err(ParseError::NotEnoughArgs));
        test_fallible("(1 + 2))", vec![LPAR, num(1), op('+'), num(2), RPAR, RPAR], Err(ParseError::UnmatchedParens));
        test_fallible("((1 + 2", vec![LPAR, LPAR, num(1), op('+'), num(2)], Err(ParseError::UnmatchedParens));
        test_fallible("a + b", vec![], Err(ParseError::Tokenization(TokenizeError::invalid_symbol(0))));
    }

    fn test(input: &str, after_tokenize: Vec<Token>, after_parse: Vec<Token>) {
        test_fallible(input, after_tokenize, Ok(after_parse));
    }

    fn test_fallible(input: &str, after_tokenize: Vec<Token>, after_parse: Result<Vec<Token>, ParseError>) {
        let collected: Vec<_> = match tokenize(input).collect() {
            Ok(collected) => collected,
            Err(_) => vec![],
        };

        assert_eq!(collected, after_tokenize);

        match (Parser::new(tokenize(input)).collect::<Result<Vec<_>, _>>(), after_parse) {
            (Ok(collected), Ok(expected)) => assert_eq!(collected, expected),
            (Err(parse_err), Err(expected)) => assert_eq!(parse_err, expected),
            (Ok(collected), Err(expected)) => panic!("Unexpectedly parsed \"{input}\": {collected:?}, expected err: {expected}"),
            (Err(parse_err), Ok(expected)) => panic!("Couldn't parse \"{input}\": {parse_err}, expected: {expected:?}"),
        };
    }
}
