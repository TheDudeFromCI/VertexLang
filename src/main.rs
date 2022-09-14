mod compiler;
mod context;
mod runtime;
mod bytecode;
use clap::{Parser, Subcommand};
use std::fs;

#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate cascade;

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
            let bytes = fs::read(file).unwrap();
            let code = bytecode::Bytecode::from_bytes(&bytes).unwrap();
            let vm = bytecode::VM::new(&code);
            vm.exec(0);
        }

        Subcommands::Compile { file } => {
            println!("Compiling file: {:?}", file);
            let code = fs::read_to_string(file).unwrap();
            let val = compiler::compile(&code);

            match val {
                Ok(_) => println!("Compiled."),
                Err(e) => println!("{}", e),
            }
        }
    }
}
