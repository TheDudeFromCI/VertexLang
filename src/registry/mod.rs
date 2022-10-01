//! A container module for hosting registries for elements that are plugged into
//! the Vertex runtime without directly part of the language itself. This
//! includes elements from the standard library as well.


mod error;
mod function;

pub use function::*;
