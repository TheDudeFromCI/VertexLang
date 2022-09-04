mod compiler;
use clap::{Parser, Subcommand};
use std::fs;

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
      let code = fs::read_to_string(file).unwrap();
      let tokens = compiler::token_reader::read_all_tokens(&code);
      println!("Tokens: {:?}", tokens);
    }
  }
}
