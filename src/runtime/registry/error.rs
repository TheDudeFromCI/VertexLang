use std::error::Error;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Function `{0}` already exists")]
    FunctionAlreadyExists(String),

    #[error("Cannot use unresolved data types in function metadata")]
    UnresolvedDataType,
}


impl RegistryError {
    /// Boxes this error and returns it as a result.
    pub fn err<T>(self) -> Result<T, Box<dyn Error>> {
        Err(Box::new(self))
    }
}
