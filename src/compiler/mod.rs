pub mod ast;
pub mod interpreter;
pub mod parser;
use ast::Node;
use std::fmt;

pub trait Compile {
    type Output;

    fn eval_from_ast(ast: Node) -> Result<Self::Output, CompilerError>;

    fn eval_from_source(source: &str) -> Result<Self::Output, CompilerError> {
        return Self::eval_from_ast(Self::compile_from_source(source)?);
    }

    fn compile_from_source(source: &str) -> Result<Node, CompilerError> {
        return parser::parse(source);
    }
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    message: String,
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to compile!\n{}", self.message)
    }
}
