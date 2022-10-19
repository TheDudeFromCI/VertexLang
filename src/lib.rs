//! A handler crate for compiling and interpreting VertexLang code.
//!
//! This crate is primarily intended to be used as a standalone executable,
//! however it does expose several of it's inner module components for the
//! purpose of interfacing with the VertexLang interpreter. This is useful for
//! embedding the VertexLang toolkit, adding new built-in functions, or adding
//! a more stream-lined compilation pipeline for your own applications.

#![warn(missing_docs)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]


mod bytecode;
mod compiler;
mod data;
pub mod multithreading;
pub mod registry;
mod virtual_machine;

pub use bytecode::*;
pub use data::*;
pub use virtual_machine::*;


extern crate derivative;

#[macro_use]
extern crate pest_derive;
