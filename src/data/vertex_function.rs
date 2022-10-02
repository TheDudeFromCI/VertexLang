use crate::data::Data;
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
pub type VertexFunction = fn(inputs: Vec<Arc<Data>>) -> Data;


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
