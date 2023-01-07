use tokens::{Brace, Operation, Number};

pub mod tokens;
pub mod tokenizer;
pub mod parser;

pub trait TokenVisitor {
    fn visit_brace(&mut self, brace: Brace);
    fn visit_op(&mut self, op: Operation);
    fn visit_num(&mut self, num: Number);
}
