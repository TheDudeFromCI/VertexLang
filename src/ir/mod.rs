//! The immediate representation runtime container for Vertex programs.
//!
//! The nodes in this module are designed to reflect a more well-defined
//! code representation, with more appropriate type-validation, identifier
//! references, and where similar compilation errors can be properly reviewed.
//!
//! In addition, this module aims to be ready for direct interpretation into a
//! virtual machine for rapid execution without additional compiling or bytecode
//! optimizations, if desired.


pub mod compiler;
mod errors;
pub mod ir_nodes;
