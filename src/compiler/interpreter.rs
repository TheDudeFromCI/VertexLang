use super::ast::*;
use super::Compile;
use super::CompilerError;

pub struct Interpreter;
pub struct Eval;

impl Compile for Interpreter {
    type Output = i64;

    fn eval_from_ast(node: Node) -> Result<Self::Output, CompilerError> {
        let evaluator = Eval::new();
        Ok(evaluator.eval(&node)?)
    }
}

impl Eval {
    pub fn new() -> Self {
        return Self;
    }

    pub fn eval(&self, _node: &Node) -> Result<i64, CompilerError> {
        todo!();
    }
}
