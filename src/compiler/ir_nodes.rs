//! Intermediate Representation level program nodes.
//!
//! These nodes can be compiled from an abstract syntax tree in order to get a
//! lower-level code view of the program bytecode that will be generated. Nodes
//! within this module define the strict data structures that are defined for
//! compilation type-checking and optimization.

use super::errors::IRError;
use lazy_static::lazy_static;
use regex::Regex;


/// Represents an intermediate-level Vertex representation of a program context.
#[derive(Debug, Clone, PartialEq)]
pub struct IRContext {
    structs:   Vec<IRStruct>,
    functions: Vec<IRFunction>,
}

impl IRContext {
    /// Creates a new intermediate representation of a Vertex program context.
    pub(super) fn new() -> Self {
        Self {
            structs:   vec![],
            functions: vec![],
        }
    }


    /// Gets the structure within this context with the given identifier path.
    ///
    /// If there is no struct within this context with the given identifier
    /// path, then None is returned.
    pub fn get_struct(&self, path: &Vec<String>) -> Option<&IRStruct> {
        self.structs.iter().find(|&structure| structure.path().eq(path))
    }


    /// Gets the function within this context with the given identifier path.
    ///
    /// If there is no function within this context with the given identifier
    /// path, then None is returned.
    pub fn get_function(&self, path: &Vec<String>) -> Option<&IRFunction> {
        self.functions.iter().find(|&function| function.path().eq(path))
    }


    /// Adds a new structure to this program context.
    pub(super) fn add_struct(&mut self, structure: IRStruct) {
        self.structs.push(structure);
    }


    /// Adds a new function to this program context.
    pub(super) fn add_function(&mut self, function: IRFunction) {
        self.functions.push(function);
    }
}

impl Default for IRContext {
    fn default() -> Self {
        Self::new()
    }
}


/// Represents an intermediate-level Vertex representation of a program data
/// structure.
///
/// This structure data is only used for compilation type checking, and is not
/// included in the resulting Vertex bytecode.
#[derive(Debug, Clone, PartialEq)]
pub struct IRStruct {
    ident_path:    Vec<String>,
    accessability: u32,
    fields:        Vec<(String, IRDataType)>,
}

impl IRStruct {
    /// Creates a new intermediate representation of a structure with the given
    /// identifier path.
    pub(super) fn new(ident_path: Vec<String>, accessability: u32) -> Self {
        Self {
            ident_path,
            accessability,
            fields: vec![],
        }
    }


    /// Gets the full identifier pathname of this structure.
    ///
    /// Each string in the vector represents a nested module or function that is
    /// a child of the previous element, with the final string in the list being
    /// the local name of this structure.
    pub fn path(&self) -> &Vec<String> {
        &self.ident_path
    }


    /// Gets the accessability level of this struct.
    ///
    /// The accessability level of a node is defined by the minimum number of
    /// matching sections within the identifier path that must match the element
    /// that is trying to access this node.
    ///
    /// For example, an accessability level of 3 means that at least the first
    /// 3 elements within the path of the reader must match this node's path in
    /// order for the reader to have permission. If the reader has less than 3
    /// sections within it's identifier path, or one of the first 3 elements are
    /// not equal to this node's respective path sections, then the reader is
    /// blocked from reading this node.
    pub fn accessability(&self) -> u32 {
        self.accessability
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


/// Represents an intermediate-level Vertex representation of an executable
/// function node.
#[derive(Debug, Clone, PartialEq)]
pub struct IRFunction {
    ident_path:    Vec<String>,
    accessability: u32,
    statements:    Vec<IRNode>,
    inputs:        Vec<IRDataType>,
    output:        IRDataType,
}

impl IRFunction {
    /// Creates a new intermediate representation of an executable function node
    /// with the given identifier path.
    pub fn new(
        ident_path: Vec<String>, accessability: u32, inputs: Vec<IRDataType>, output: IRDataType,
        statements: Vec<IRNode>,
    ) -> Self {
        Self {
            ident_path,
            accessability,
            statements,
            inputs,
            output,
        }
    }


    /// Gets the full identifier pathname of this function.
    ///
    /// Each string in the vector represents a nested module or function that is
    /// a child of the previous element, with the final string in the list being
    /// the local name of this function.
    pub fn path(&self) -> &Vec<String> {
        &self.ident_path
    }


    /// Gets the accessability level of this function.
    ///
    /// The accessability level of a node is defined by the minimum number of
    /// matching sections within the identifier path that must match the element
    /// that is trying to access this node.
    ///
    /// For example, an accessability level of 3 means that at least the first
    /// 3 elements within the path of the reader must match this node's path in
    /// order for the reader to have permission. If the reader has less than 3
    /// sections within it's identifier path, or one of the first 3 elements are
    /// not equal to this node's respective path sections, then the reader is
    /// blocked from reading this node.
    pub fn accessability(&self) -> u32 {
        self.accessability
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


/// A intermediate-level representation of a data-type.
#[derive(Debug, Clone, PartialEq)]
pub enum IRDataType {
    /// Represents a data type that has a known type-name, but has not yet been
    /// resolved into an actual data type yet. This is usually a struct name
    /// that has not been loaded yet.
    Unresolved(String),

    /// Represents a data type that has no known type or type name. This is
    /// usually a data type that is returned from an unresolved function call,
    /// where it is assumed that some data type is returned, but is unclear what
    /// that data type is.
    Unknown,

    /// A 64-bit signed integer data type.
    Int,

    /// A 64-bit signed floating point data type.
    Float,

    /// A UTF-8 string data type..
    String,

    /// A single UTF-8 character data type.
    Char,

    /// A boolean data type.
    Bool,

    /// An error data type.
    Error,

    /// A null data type.
    Null,

    /// A resizable list of data elements of a given data type.
    List(Box<IRDataType>),

    /// A statically-sized array of data elements of a given data type.
    Array(Box<IRDataType>, u32),

    /// A data type that may or may not be defined.
    Option(Box<IRDataType>),

    /// A data type that may either be defined, or may be an error.
    Result(Box<IRDataType>),

    /// A statically defined ordered set of data types that make up this one.
    /// This is similar to a struct, but unnamed and short-lived.
    Tuple(Vec<IRDataType>),

    /// A dictionary, or map, of two data types. For each key, of the first data
    /// type, there is a corresponding value, or the second data type. Keys
    /// within the dictionary must be unique.
    Dictionary(Box<IRDataType>, Box<IRDataType>),

    /// A named structure data type that contains a set of named fields, each
    /// with a defined data type.
    Struct(String, Vec<(String, IRDataType)>),
}

impl IRDataType {
    /// Gets the corresponding data type from the given data type name.
    ///
    /// If the type is not a primitive type, then it is returned as unresolved.
    pub(super) fn from(name: &str) -> Self {
        match name {
            "Int" => IRDataType::Int,
            "Float" => IRDataType::Float,
            "String" => IRDataType::String,
            "Char" => IRDataType::Char,
            "Bool" => IRDataType::Bool,
            "Error" => IRDataType::Error,
            "Null" => IRDataType::Null,

            other => {
                lazy_static! {
                    static ref LIST_RE: Regex = Regex::new("(.+)\\[\\]").unwrap();
                    static ref ARRAY_RE: Regex = Regex::new("(.+)\\[(0-9)+\\]").unwrap();
                    static ref OPTION_RE: Regex = Regex::new("(.+)\\?").unwrap();
                    static ref RESULT_RE: Regex = Regex::new("(.+)!").unwrap();
                    static ref TUPLE_RE: Regex = Regex::new("\\(\\s*,?\\s*(.+)\\s*\\)").unwrap();
                    static ref DICTIONARY_RE: Regex =
                        Regex::new("\\{\\s*(.+)\\s*:\\s*(.+)\\s*\\}").unwrap();
                }

                if let Some(caps) = LIST_RE.captures(other) {
                    IRDataType::List(Box::new(IRDataType::from(&caps[0])))
                } else if let Some(caps) = ARRAY_RE.captures(other) {
                    let count = caps[1].parse::<u32>().unwrap();
                    IRDataType::Array(Box::new(IRDataType::from(&caps[0])), count)
                } else if let Some(caps) = OPTION_RE.captures(other) {
                    IRDataType::Option(Box::new(IRDataType::from(&caps[0])))
                } else if let Some(caps) = RESULT_RE.captures(other) {
                    IRDataType::Result(Box::new(IRDataType::from(&caps[0])))
                } else if let Some(caps) = TUPLE_RE.captures(other) {
                    let mut elements = vec![];
                    for cap in caps.iter() {
                        elements.push(IRDataType::from(cap.unwrap().as_str()));
                    }
                    IRDataType::Tuple(elements)
                } else if let Some(caps) = DICTIONARY_RE.captures(other) {
                    let key = Box::new(IRDataType::from(&caps[0]));
                    let value = Box::new(IRDataType::from(&caps[1]));
                    IRDataType::Dictionary(key, value)
                } else {
                    IRDataType::Unresolved(String::from(other))
                }
            },
        }
    }


    /// Check to see if this data type is completely resolved or not.
    pub fn is_resolved(&self) -> bool {
        match self {
            IRDataType::Unresolved(_) => false,
            &IRDataType::Unknown => false,
            IRDataType::Int => true,
            IRDataType::Float => true,
            IRDataType::String => true,
            IRDataType::Char => true,
            IRDataType::Bool => true,
            IRDataType::Error => true,
            IRDataType::Null => true,
            IRDataType::List(e) => e.is_resolved(),
            IRDataType::Array(e, _) => e.is_resolved(),
            IRDataType::Option(e) => e.is_resolved(),
            IRDataType::Result(e) => e.is_resolved(),
            IRDataType::Tuple(v) => v.iter().all(|e| e.is_resolved()),
            IRDataType::Dictionary(k, v) => k.is_resolved() && v.is_resolved(),
            IRDataType::Struct(_, f) => f.iter().all(|(_, e)| e.is_resolved()),
        }
    }
}
