use crate::parser::nodes::NodePosition;
use std::fmt;
use std::fmt::Display;
use thiserror::Error as ThisError;


#[derive(ThisError, Debug)]
pub struct CompilerError {
    source:   IRError,
    position: NodePosition,
}

impl CompilerError {
    pub(crate) fn new(position: NodePosition, source: IRError) -> Self {
        CompilerError {
            source,
            position,
        }
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error: {}, at {}:{}",
            self.source, self.position.line, self.position.col
        )
    }
}


#[derive(ThisError, Debug)]
pub enum IRError {
    #[error("An element with the name '{0}' already exists")]
    IdentifierAlreadyExists(String),

    #[error("Cannot find element '{0}' within the current scope")]
    UnknownIdentifier(String),
}
