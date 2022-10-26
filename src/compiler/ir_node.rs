use super::IRDataType;


/// Defines the input data type for a statement node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IRNodeInput {
    /// The node input should come from the graph input parameter at the given
    /// index.
    FunctionParam(u32),

    /// The node input should come from another node within the same graph at
    /// the given index.
    HiddenNode(u32),
}


/// Defines a function call to a specific type of function and it's definition
/// location.
#[derive(Debug, Clone, PartialEq)]
pub enum IRFuncCall {
    /// Indicates the function is an external function with the given function
    /// name.
    External(String),

    /// Points to an internal function call at the given pointer within the
    /// context object.
    Internal(usize),

    /// This is an internal function with the given name, but it may or may not
    /// be loaded within the context yet.
    Unresolved(String),

    /// An empty function that takes no inputs and returns an integer constant
    /// value.
    IntConstant(i64),

    /// An empty function that takes no inputs and returns a float constant
    /// value.
    FloatConstant(f64),

    /// An empty function that takes no inputs and returns a string constant
    /// value.
    StringConstant(String),

    /// An empty function that takes no inputs and returns a character constant
    /// value.
    CharConstant(char),

    /// An empty function that takes no inputs and returns a boolean constant
    /// value.
    BoolConstant(bool),
}


/// A node is a function call within a function graph that takes in a set of
/// inputs and outputs a given data type.
#[derive(Debug, Clone, PartialEq)]
pub struct IRNode {
    function: IRFuncCall,
    inputs:   Vec<IRNodeInput>,
    output:   IRDataType,
}

impl IRNode {
    /// Creates a new function call node instance.
    pub fn new(function: IRFuncCall, inputs: Vec<IRNodeInput>, output: IRDataType) -> Self {
        Self {
            function,
            inputs,
            output,
        }
    }


    /// Gets a list of all inputs, in order, for this node.
    pub fn get_inputs(&self) -> &Vec<IRNodeInput> {
        &self.inputs
    }


    /// Gets the output data type of this node.
    pub fn get_output(&self) -> &IRDataType {
        &self.output
    }


    /// Gets the function that is executed by this node.
    pub fn get_function(&self) -> &IRFuncCall {
        &self.function
    }
}
