use std::{ops, fmt::{Display, Write}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paren {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    pub(crate) fn prio(&self) -> i32 {
        match self {
            Operation::Add | Operation::Sub => 1,
            Operation::Mul | Operation::Div => 2,
        }
    }

    fn as_fun<T>(self) -> fn(T, T) -> T
    where
        T: ops::Add<T, Output = T> + ops::Sub<T, Output = T>,
        T: ops::Mul<T, Output = T> + ops::Div<T, Output = T>,
    {
        match self {
            Operation::Add => ops::Add::add,
            Operation::Sub => ops::Sub::sub,
            Operation::Mul => ops::Mul::mul,
            Operation::Div => ops::Div::div,
        }
    }

    pub fn apply<T>(self, lhs: T, rhs: T) -> T
    where
        T: ops::Add<T, Output = T> + ops::Sub<T, Output = T>,
        T: ops::Mul<T, Output = T> + ops::Div<T, Output = T>,
    {
        self.as_fun()(lhs, rhs)
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Operation::Add => '+',
            Operation::Sub => '-',
            Operation::Mul => '*',
            Operation::Div => '/',
        };
        f.write_char(ch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Number(Number),
    Paren(Paren),
    Oper(Operation),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(Number(num)) => f.write_fmt(format_args!("{num}")),
            Token::Paren(Paren::Left) => f.write_char('('),
            Token::Paren(Paren::Right) => f.write_char(')'),
            Token::Oper(op) => f.write_fmt(format_args!("{op}")),
        }
    }
}
