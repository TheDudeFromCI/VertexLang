use std::fmt;

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to compile!\n{}", self.message)
    }
}
