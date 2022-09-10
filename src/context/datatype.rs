use std::fmt;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int,
    Float,
    String,
    Bool,
    Unknown,
    Struct {
        name: String,
        fields: Vec<(String, Box<DataType>)>,
    },
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            DataType::Int => write!(f, "Int"),
            DataType::Float => write!(f, "Float"),
            DataType::String => write!(f, "String"),
            DataType::Bool => write!(f, "Bool"),
            DataType::Unknown => write!(f, "?"),
            DataType::Struct { name, fields } => {
                write!(f, "{} {{\n{}}}", name, format_fields(fields))
            }
        }
    }
}

fn format_fields(fields: &Vec<(String, Box<DataType>)>) -> String {
    let mut s = String::new();

    for (name, dtype) in fields {
        writeln!(s, "  {}: {}", name, dtype).unwrap();
    }

    s
}
