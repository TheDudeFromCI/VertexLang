//! The function registry for Vertex.


use crate::compiler::ir::IRDataType;
use crate::runtime::data::VertexFunction;
use crate::runtime::registry::error::RegistryError;
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
        if self.get_function(&function.name).is_some() {
            return RegistryError::FunctionAlreadyExists(function.name).err();
        }

        self.functions.push(function);
        Ok(())
    }


    /// Gets the function meta data for the given function name.
    ///
    /// If there is no function with the given name, then None is returned.
    pub fn get_function(&self, name: &str) -> Option<&FuncMeta> {
        let index = self.functions.iter().position(|f| f.name == name);
        if let Some(id) = index {
            Some(&self.functions[id])
        } else {
            None
        }
    }
}


impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
