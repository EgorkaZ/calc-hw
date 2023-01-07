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
            Operation::Add|Operation::Sub => 1,
            Operation::Mul|Operation::Div => 2,
        }
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
