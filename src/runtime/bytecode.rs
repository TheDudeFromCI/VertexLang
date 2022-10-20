//! The lowest level of a Vertex program representation. All data not relevant
//! to loading or executing the program are discarded and no further
//! optimizations or debug data are maintained.

use crate::compiler;
use crate::compiler::ir::{IRContext, IRFuncCall, IRNodeInput};
use crate::runtime::data::{Data, VertexFunction};
use crate::runtime::registry::FunctionRegistry;
use std::error::Error;
use std::sync::Arc;


/// A pointer to an external function that can be called from within Vertex.
pub(crate) struct ExternalFunction {
    name:     String,
    function: VertexFunction,
}

impl ExternalFunction {
    /// Creates a new external function instance for the given function.
    pub fn new(name: String, function: VertexFunction) -> Self {
        ExternalFunction {
            name,
            function,
        }
    }


    /// Gets the name of this function within the function registry.
    pub fn get_function_name(&self) -> &str {
        &self.name
    }


    /// Gets the executable function pointer within Rust.
    pub fn get_function_exec(&self) -> &VertexFunction {
        &self.function
    }
}


/// A pointer to a function within the bytecode to be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionCall {
    /// Points to the internal function at the given index.
    Internal(usize),

    /// Points to the external function at the given index.
    External(usize),

    /// Points to the constant data value at the given index.
    Constant(usize),
}


/// A pointer to a node that provides the input to another function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationInput {
    /// Points to the function input parameter at the given index.
    Param(usize),

    /// Points to the another operation within the same function at the given
    /// index.
    Hidden(usize),
}


/// A single executable instruction to be performed within Vertex.
#[derive(Debug, Clone)]
pub(crate) struct Operation {
    function_call: FunctionCall,
    inputs:        Vec<OperationInput>,
}

impl Operation {
    /// Creates a new operation instance with the given function and operation
    /// input pointers.
    pub fn new(function_call: FunctionCall, inputs: Vec<OperationInput>) -> Self {
        Operation {
            function_call,
            inputs,
        }
    }


    /// Gets the function pointer this operation needs to evaluate.
    pub fn get_function(&self) -> &FunctionCall {
        &self.function_call
    }


    /// Gets the list of input pointers for this operation.
    pub fn get_inputs(&self) -> &Vec<OperationInput> {
        &self.inputs
    }
}


/// A container for a set of operation instructions that need to be executed in
/// order to evaluation this function's output value based on a set of inputs.
pub(crate) struct InternalFunction {
    operations: Vec<Operation>,
}

impl InternalFunction {
    /// Creates a new, empty internal function instance.
    pub fn new() -> Self {
        InternalFunction {
            operations: vec![],
        }
    }


    /// Appends a new operation to this function.
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }


    /// Gets the list of operations within this function to be executed.
    pub fn get_operations(&self) -> &Vec<Operation> {
        &self.operations
    }
}

impl Default for InternalFunction {
    fn default() -> Self {
        Self::new()
    }
}


/// A bytecode container for an executable Vertex program.
pub struct VertexBytecode {
    external_functions: Vec<ExternalFunction>,
    internal_functions: Vec<InternalFunction>,
    constants:          Vec<Arc<Data>>,
}

impl VertexBytecode {
    /// Creates a new, empty Vertex Bytecode instance.
    pub fn new() -> Self {
        VertexBytecode {
            external_functions: vec![],
            internal_functions: vec![],
            constants:          vec![],
        }
    }


    /// Compiles the given Vertex source code into executable byte code.
    pub fn from_source(source: &str, registry: &FunctionRegistry) -> Result<Self, Box<dyn Error>> {
        let context_node = compiler::grammar::parse(source)?;
        let ir = compiler::ir::compile_context(context_node, registry)?;
        let bytecode = bytecode_from_ir(ir, registry);
        Ok(bytecode)
    }


    /// Adds a new external function to this bytecode.
    pub(crate) fn add_external_function(&mut self, function: ExternalFunction) {
        self.external_functions.push(function);
    }


    /// Gets a list of all external functions within this bytecode.
    pub(crate) fn get_external_functions(&self) -> &Vec<ExternalFunction> {
        &self.external_functions
    }


    /// Adds a new internal function to this bytecode.
    pub(crate) fn add_internal_function(&mut self, function: InternalFunction) {
        self.internal_functions.push(function);
    }


    /// Gets a list of all internal functions within this bytecode.
    pub(crate) fn get_internal_functions(&self) -> &Vec<InternalFunction> {
        &self.internal_functions
    }


    /// Adds a new constant data value to this bytecode.
    pub(crate) fn add_constant(&mut self, constant: Data) {
        self.constants.push(Arc::new(constant));
    }


    /// Gets a list of all constant values within this bytecode.
    pub(crate) fn get_constants(&self) -> &Vec<Arc<Data>> {
        &self.constants
    }
}

impl Default for VertexBytecode {
    fn default() -> Self {
        Self::new()
    }
}


/// Creates a new VertexBytecode instance based on the given IRContext.
///
/// This method will panic if the intermediate representation is not properly
/// loaded or generated.
fn bytecode_from_ir(context: IRContext, registry: &FunctionRegistry) -> VertexBytecode {
    let mut bytecode = VertexBytecode::new();

    for function in context.get_functions() {
        let mut int_func = InternalFunction::new();
        for statement in function.get_statements() {
            let func = match statement.get_function() {
                IRFuncCall::External(f) => add_ext_func(&mut bytecode, f, registry),
                IRFuncCall::Internal(f) => FunctionCall::Internal(*f),
                IRFuncCall::IntConstant(v) => add_const(&mut bytecode, Data::Int(*v)),
                IRFuncCall::FloatConstant(v) => add_const(&mut bytecode, Data::Float(*v)),
                IRFuncCall::StringConstant(v) => add_const(&mut bytecode, Data::String(v.clone())),
                IRFuncCall::CharConstant(v) => add_const(&mut bytecode, Data::Char(*v)),
                IRFuncCall::BoolConstant(v) => add_const(&mut bytecode, Data::Bool(*v)),
                IRFuncCall::Unresolved(_) => {
                    panic!("Cannot load bytecode from unresolved functions!")
                },
            };

            let mut inputs = vec![];
            for input in statement.get_inputs() {
                let input = match input {
                    IRNodeInput::FunctionParam(i) => OperationInput::Param(*i as usize),
                    IRNodeInput::HiddenNode(i) => OperationInput::Hidden(*i as usize),
                };
                inputs.push(input);
            }

            let operation = Operation::new(func, inputs);
            int_func.add_operation(operation);
        }

        bytecode.add_internal_function(int_func);
    }

    bytecode
}


fn add_const(bytecode: &mut VertexBytecode, constant: Data) -> FunctionCall {
    if let Some(index) = bytecode.get_constants().iter().position(|c| **c == constant) {
        FunctionCall::Constant(index)
    } else {
        bytecode.add_constant(constant);
        FunctionCall::Constant(bytecode.get_constants().len() - 1)
    }
}


fn add_ext_func(
    bytecode: &mut VertexBytecode, function: &str, registry: &FunctionRegistry,
) -> FunctionCall {
    if let Some(index) = bytecode
        .get_external_functions()
        .iter()
        .position(|f| f.get_function_name().eq(function))
    {
        FunctionCall::External(index)
    } else if let Some(func_meta) = registry.get_function(function) {
        let func = func_meta.get_func();
        bytecode.add_external_function(ExternalFunction::new(function.to_owned(), func));
        FunctionCall::External(bytecode.get_external_functions().len() - 1)
    } else {
        panic!("Unknown function: {}", function);
    }
}
