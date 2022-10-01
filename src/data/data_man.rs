//! Core data container elements within Vertex.


use std::fmt::{self, Write};


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

    /// A boolean value.
    Bool(bool),

    /// A constructed data value built from a specific struct definition. See
    /// [`crate::data::DataType::Struct`] for more information.
    Struct(StructData),

    /// A list of data values, where all data values within the list are of the
    /// same type.
    List(Vec<Data>),

    /// An error that was thrown while executing a function. Contains an error
    /// message.
    Error(String),

    /// A wrapper for data that may or may not be null.
    Option(Box<Data>),

    /// A wrapper for data that may be an error or not.
    Result(Box<Data>),
}


impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Null => write!(f, "Null"),
            Data::Int(val) => write!(f, "{}", val),
            Data::Float(val) => write!(f, "{:.3}", val),
            Data::String(val) => write!(f, "{}", val),
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

            Data::List(val) => {
                for (index, value) in val.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                Ok(())
            },
        }
    }
}
