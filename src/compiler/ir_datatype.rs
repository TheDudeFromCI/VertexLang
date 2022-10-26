//! Intermediate Representation level program nodes.
//!
//! These nodes can be compiled from an abstract syntax tree in order to get a
//! lower-level code view of the program bytecode that will be generated. Nodes
//! within this module define the strict data structures that are defined for
//! compilation type-checking and optimization.


use lazy_static::lazy_static;
use regex::Regex;


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
    pub fn from(name: &str) -> Self {
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
