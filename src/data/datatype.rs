/// A named field within a struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    /// The name of the field.
    pub name: String,

    /// The data type of the field.
    pub dtype: DataType,
}


/// A data type primitive within Vertex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    /// An 8-byte signed integer value.
    Int,

    /// An 8-byte signed floating point value.
    Float,

    /// A managed, UTF-8 string value.
    String,

    /// A boolean value.
    Bool,

    /// An error type that can be returned from a function that failed.
    Error,

    /// An list containing a variable number of a specific data type.
    List(Box<DataType>),

    /// A return result that may be an error or a normal data type.
    Result(Box<DataType>),

    /// A return result that may be null or a normal data type.
    Option(Box<DataType>),

    /// A custom data type, built from a set of name data types acting as
    /// fields.
    ///
    /// Structs can contain any other data type as a field, including other
    /// structs.
    Struct {
        /// The name of the struct type.
        type_name: String,

        /// The fields within the struct.
        fields: Vec<StructField>,
    },
}
