pub mod ast;
pub mod parser;
use std::fmt;

pub fn compile(source: &str) -> Result<(), CompilerError> {
    let _root_node = parser::parse(source)?;

    Ok(())
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
