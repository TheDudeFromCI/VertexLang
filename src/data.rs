//! Core data container elements within Vertex.


use std::fmt::{self, Write};
use std::sync::Arc;


/// An external Rust function that can be executed from with Vertex.
///
/// This function takes a set of ordered input arguments based off the metadata
/// specified when the function is registered. This function, after finishing
/// it's computation, returns the generated data. For serial functions that have
/// no return type, this returned data may simply be Null.
///
/// When implementing this function type, it might be useful to use the
/// `unwrap_data!()` macro.
pub type VertexFunction = fn(inputs: &[Arc<Data>]) -> Data;


/// A macro to quickly unwrap data into a specific type. This is used in
/// situations where you can be reasonably sure that the data is of a certain
/// type and need to unwrap it to retrieve the contents inside.
///
/// If the type cannot be unwrapped into the specified data type, then this
/// macro will panic.
///
/// # Example
/// ```
/// use vertex_lang::data::Data;
/// use vertex_lang::unwrap_data;
///
/// fn add(inputs: &[Data], outputs: &mut [Data]) {
///     let a = unwrap_data!(&inputs[0], Int);
///     let b = unwrap_data!(&inputs[1], Int);
///     outputs[0] = Data::Int(a + b);
/// }
///
///
/// let inputs = vec![Data::Int(1), Data::Int(2)];
/// let mut outputs = vec![Data::Null]; // All unassigned data defaults to null
///
/// add(&inputs, &mut outputs);
/// assert_eq!(outputs[0], Data::Int(3));
/// ```
#[macro_export]
macro_rules! unwrap_data {
    ( $input:expr, $dtype:ident ) => {{
        match &*$input {
            Data::$dtype(val) => val,
            inp => panic!("Unexpected data type: {}", inp),
        }
    }};
}


/// Contains the struct type and contents of a managed struct within the Vertex
/// runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct StructData {
    /// The name of the struct type being used.
    pub struct_type: String,

    /// The values of the fields within this struct, where each item in the
    /// list corresponds to it's field index within the struct definition.
    pub fields: Vec<Data>,
}


/// A managed data instance used by Vertex and stored on the heap. Data values
/// are immutable and statically typed.
#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    /// A data value that has not yet been assigned.
    Null,

    /// An 8-byte signed integer value.
    Int(i64),

    /// An 8-byte signed floating point value.
    Float(f64),

    /// A managed, UTF-8 string value.
    String(String),

    /// A single UTF-8 character value.
    Char(char),

    /// A boolean value.
    Bool(bool),

    /// A constructed data value built from a specific struct definition. See
    /// [`crate::data::StructData`] for more information.
    Struct(StructData),

    /// A list of data values, where all data values within the list are of the
    /// same type.
    List(Vec<Data>),

    /// A fixed size array of data values.
    Array(Vec<Data>),

    /// An error that was thrown while executing a function. Contains an error
    /// message.
    Error(String),

    /// A wrapper for data that may or may not be null.
    Option(Box<Data>),

    /// A wrapper for data that may be an error or not.
    Result(Box<Data>),

    /// A fixed array of data types, where each element represents a different
    /// property within the tuple. This is effectively an anonymous struct
    /// value.
    Tuple(Vec<Data>),

    /// A dictionary of key-value pairs of data.
    Dictionary(Vec<Data>, Vec<Data>),
}


impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Null => write!(f, "Null"),
            Data::Int(val) => write!(f, "{}", val),
            Data::Float(val) => write!(f, "{:.3}", val),
            Data::String(val) => write!(f, "{}", val),
            Data::Char(val) => write!(f, "{}", val),
            Data::Bool(val) => write!(f, "{}", val),

            Data::Error(val) => write!(f, "Error: \"{}\"", val),
            Data::Option(val) => write!(f, "Option({}) ", val),
            Data::Result(val) => write!(f, "Result({})", val),

            Data::Struct(val) => {
                write!(f, "{} {{ {} }}", val.struct_type, {
                    let mut field_list = String::new();
                    for field in &val.fields {
                        if !field_list.is_empty() {
                            field_list.push_str(", ");
                        }
                        field_list.write_fmt(format_args!("{field}"))?;
                    }
                    field_list
                })
            },

            Data::List(val) | Data::Array(val) | Data::Tuple(val) => {
                for (index, value) in val.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                Ok(())
            },

            Data::Dictionary(keys, values) => {
                write!(f, "{{")?;
                for i in 0..keys.len() {
                    write!(f, "{}: {}", keys[i], values[i])?;
                }
                write!(f, "}}")?;

                Ok(())
            },
        }
    }
}
