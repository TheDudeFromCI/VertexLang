use super::ast::*;
use super::Compile;
use super::CompilerError;

pub struct Interpreter;
pub struct Eval;

impl Compile for Interpreter {
    type Output = i64;

    fn eval_from_ast(ast: Vec<Node>) -> Result<Vec<Self::Output>, CompilerError> {
        let mut outputs: Vec<Self::Output> = vec![];
        let evaluator = Eval::new();

        for node in ast {
            outputs.push(evaluator.eval(&node)?);
        }

        return Ok(outputs);
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
