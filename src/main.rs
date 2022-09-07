mod compiler;
use clap::{Parser, Subcommand};
use compiler::interpreter::Interpreter;
use compiler::Compile;
use std::fs;

#[macro_use]
extern crate pest_derive;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// Executes a .vtx file.
    Run {
        /// The .vtx file to execute
        #[clap(value_parser)]
        file: String,
    },

    /// Compiles a .vt file into a .vtx file
    Compile {
        /// The .vt file to compile
        #[clap(value_parser)]
        file: String,
    },
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Subcommands::Run { file } => {
            println!("Running file: {:?}", file);
        }

        Subcommands::Compile { file } => {
            println!("Compiling file: {:?}", file);
            let code = fs::read_to_string(file).unwrap();
            let val = Interpreter::compile_from_source(&code);

            match val {
                Ok(v) => println!("{}", v),
                Err(e) => println!("{}", e),
            }
        }
    }
}
