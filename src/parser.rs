use crate::{tokens, TokenVisitor};

#[derive(Debug)]
struct Parser<I> {
    inner: I,
    stack: Vec<Stacked>,
    curr: Option<tokens::Token>,
    state: State,
}

#[derive(Debug)]
enum Stacked {
    Op(tokens::Operation),
    LBrace,
}

#[derive(Debug)]
enum State {
    PopTillBrace,
    Accumulating,
    PopOp,
    Skip,
    CurrToOut,
}

impl<I> Iterator for Parser<I>
where
    I: Iterator<Item = tokens::Token>
{
    type Item = tokens::Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr = loop {
                match self.curr {
                    Some(curr) => break curr,
                    None => match self.inner.next() {
                        Some(curr) => self.curr = Some(curr),
                        None => return None,
                    },
                }
            };

            match curr {
                tokens::Token::Number(num) => self.visit_num(num),
                tokens::Token::Brace(br) => self.visit_brace(br),
                tokens::Token::Oper(op) => self.visit_op(op),
            };

            match self.state {
                State::PopTillBrace => match self.stack.pop() {
                    Some(Stacked::LBrace) => continue,
                    Some(Stacked::Op(op)) => break Some(tokens::Token::Oper(op)),
                },
                State::Accumulating => todo!(),
                State::PopOp => todo!(),
                State::Skip => todo!(),
                State::CurrToOut => todo!(),
            }
        }
    }
}

impl<I> TokenVisitor for Parser<I> {
    fn visit_brace(&mut self, brace: tokens::Brace) {
        match brace {
            tokens::Brace::Left => {
                self.stack.push(Stacked::LBrace);
                self.state = State::Skip;
            },
            tokens::Brace::Right => {
                self.state = State::PopTillBrace;
            },
        }
        self.curr = None;
    }

    fn visit_op(&mut self, op: tokens::Operation) {
        match self.stack.last() {
            Some(Stacked::LBrace)|None => {
                self.stack.push(Stacked::Op(op));
                self.state = State::Skip;
            }
            Some(Stacked::Op(stack_op)) if stack_op.prio() <= op.prio() => {
                self.stack.push(Stacked::Op(op));
                self.state = State::Skip;
            },
            Some(Stacked::Op(stack_op)) => {
                self.state = State::PopOp;
            }
        }
    }

    fn visit_num(&mut self, num: tokens::Number) {
        self.state = State::CurrToOut;
    }
}
