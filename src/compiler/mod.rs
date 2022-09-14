mod ast;
mod compiler_error;
mod datatype;
mod parser;
pub use compiler_error::CompilerError;

pub fn compile(source: &str) -> Result<(), CompilerError> {
    let _root_node = parser::parse(source)?;

    Ok(())
}
