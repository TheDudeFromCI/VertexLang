mod ast;
mod compiler_error;
mod parser;
use super::context::DataType;
pub use compiler_error::CompilerError;

pub fn compile(source: &str) -> Result<(), CompilerError> {
    let _root_node = parser::parse(source)?;

    Ok(())
}
