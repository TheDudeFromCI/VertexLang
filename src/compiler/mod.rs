//! Handles converting Vertex source code into an executable form.


mod ast;
mod ast_to_ir;
mod bytecode;
mod errors;
mod grammar;
mod ir_context;
mod ir_datatype;
mod ir_function;
mod ir_node;
mod ir_path;
mod ir_struct;
mod peg;

pub use ast::*;
pub use ast_to_ir::*;
pub use bytecode::*;
pub use grammar::*;
pub use ir_context::*;
pub use ir_datatype::*;
pub use ir_function::*;
pub use ir_node::*;
pub use ir_path::*;
pub use ir_struct::*;
