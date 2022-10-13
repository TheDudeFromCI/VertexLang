//! The function registry for Vertex.


use crate::compiler::ir_nodes::IRDataType;
use crate::data::VertexFunction;
use crate::registry::error::RegistryError;
use std::error::Error;


/// Contains function meta data for functions that have been specified within
/// the function registry.
#[derive(Clone)]
pub struct FuncMeta {
    name:       String,
    func:       VertexFunction,
    input_args: Vec<IRDataType>,
    output:     IRDataType,
}


impl FuncMeta {
    /// Creates a new function meta data container for use with the function
    /// registry.
    pub fn new(
        name: String, func: VertexFunction, input_args: Vec<IRDataType>, output: IRDataType,
    ) -> Result<Self, RegistryError> {
        if input_args.iter().any(|d| !d.is_resolved()) || !output.is_resolved() {
            return Err(RegistryError::UnresolvedDataType);
        }

        Ok(FuncMeta {
            name,
            func,
            input_args,
            output,
        })
    }


    /// Gets the name of this function.
    pub fn get_name(&self) -> &str {
        &self.name
    }


    /// Gets the Rust function pointer.
    pub fn get_func(&self) -> VertexFunction {
        self.func
    }


    /// Gets the input argument types for this function.
    pub fn get_inputs(&self) -> &Vec<IRDataType> {
        &self.input_args
    }


    /// Gets the output argument type for this function.
    pub fn get_output(&self) -> &IRDataType {
        &self.output
    }
}


/// The function registry allows for external Rust functions to be categorized
/// for compilation and usage within Vertex source code.
pub struct FunctionRegistry {
    functions: Vec<FuncMeta>,
}


impl FunctionRegistry {
    /// Creates a new function registry.
    pub fn new() -> Self {
        FunctionRegistry {
            functions: vec![],
        }
    }


    /// Registers a new function into this registry.
    ///
    /// If there is already a function in this registry with the same path name,
    /// then an error is returned.
    pub fn register(&mut self, function: FuncMeta) -> Result<(), Box<dyn Error>> {
        if self.find_function(&function.name).is_some() {
            return RegistryError::FunctionAlreadyExists(function.name).err();
        }

        self.functions.push(function);
        Ok(())
    }


    /// Tries to find the function id of the registered function with the given
    /// name.
    ///
    /// If there is no registered function with the given name, then None is
    /// returned. Function IDs are not promised to identical between multiple
    /// application executions. As such, it is recommended to store function
    /// path names instead of IDs within bytecode files, and find the function
    /// ID at startup.
    ///
    /// Functions are also not guaranteed to be loaded within the registry
    /// between application executions if the underlying libraries or
    /// plugins providing these runtime functions are changed or removed.
    pub fn find_function(&self, name: &str) -> Option<usize> {
        self.functions.iter().position(|f| f.name == name)
    }


    /// Gets the function meta data for the given function ID.
    pub fn get_function(&self, id: usize) -> FuncMeta {
        self.functions[id].clone()
    }
}


impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
