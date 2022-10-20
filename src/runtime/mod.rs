//! Contains the runtime resources for the Vertex programming language.


mod bytecode;
mod data;
pub mod multithreading;
pub mod registry;
mod virtual_machine;

pub use bytecode::*;
pub use data::*;
pub use virtual_machine::*;
