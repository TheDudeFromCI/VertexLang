use super::{IRDataType, IRNode, IRPathElement};


/// Represents an intermediate-level Vertex representation of an executable
/// function node.
#[derive(Debug, Clone, PartialEq)]
pub struct IRFunction {
    path:       Vec<IRPathElement>,
    statements: Vec<IRNode>,
    inputs:     Vec<IRDataType>,
    output:     IRDataType,
}

impl IRFunction {
    /// Gets the identifier pathname of this function.
    pub fn path(&self) -> &Vec<IRPathElement> {
        &self.path
    }


    /// Gets a list of all statements within this function.
    ///
    /// These statements are ordered based on input dependencies.
    pub fn get_statements(&self) -> &Vec<IRNode> {
        &self.statements
    }


    /// Gets the input data types for this function.
    pub fn get_inputs(&self) -> &Vec<IRDataType> {
        &self.inputs
    }


    /// Gets the output data type for this function.
    pub fn get_output(&self) -> &IRDataType {
        &self.output
    }
}
