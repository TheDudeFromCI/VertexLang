use super::errors::IRError;
use super::{IRDataType, IRPathElement};
use anyhow::Result;


/// Represents an intermediate-level Vertex representation of a program data
/// structure.
///
/// This structure data is only used for compilation type checking, and is not
/// included in the resulting Vertex bytecode.
#[derive(Debug, Clone, PartialEq)]
pub struct IRStruct {
    path:   Vec<IRPathElement>,
    fields: Vec<(String, IRDataType)>,
}

impl IRStruct {
    /// Gets the full identifier pathname of this structure.
    pub fn path(&self) -> &Vec<IRPathElement> {
        &self.path
    }


    /// Adds a new field to this struct with the given name and data type.
    ///
    /// If the is already another field within this struct with the given name,
    /// then an error is returned.
    pub fn add_field(&mut self, name: String, dtype: IRDataType) -> Result<(), IRError> {
        if self.get_field(&name).is_none() {
            self.fields.push((name, dtype));
            Ok(())
        } else {
            Err(IRError::IdentifierAlreadyExists(name))
        }
    }


    /// Gets the data type of the indicated field within this struct, as
    /// specified by the given field name.
    ///
    /// If there is no field within this struct with the given name, then None
    /// is returned.
    pub fn get_field(&self, name: &str) -> Option<&IRDataType> {
        for (field_name, dtype) in &self.fields {
            if *field_name == name {
                return Some(dtype);
            }
        }

        None
    }
}
