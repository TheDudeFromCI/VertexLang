use super::peg::*;
use super::{IRFunction, IRPathElement, IRStruct};
use anyhow::{Context, Result};
use pest::iterators::Pair;
use pest::Parser;


/// Represents an intermediate-level Vertex representation of a program context.
#[derive(Debug, Clone, PartialEq)]
pub struct IRContext {
    structs:   Vec<IRStruct>,
    functions: Vec<IRFunction>,
}

impl IRContext {
    /// Creates a new intermediate representation of a Vertex program context.
    pub fn new() -> Self {
        Self {
            structs:   vec![],
            functions: vec![],
        }
    }


    /// Creates a new IR Context object out of the given Vertex source code.
    ///
    /// The source code is parsed via the Pest library using the Vertex grammar.
    /// The generated abstract syntax tree is then converted over to this
    /// intermediate representation. No optimizations are performed on the
    /// parsed data structures, only validation.
    pub fn from_source(source: &str) -> Result<Self> {
        let mut pairs = VertexLangParser::parse(Rule::Program, source)
            .context("Failed to generate IRContext instance")?;

        let mut context = Self::new();
        let path: [IRPathElement; 0] = [];

        for pair in pairs.next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::Module => {
                    context
                        .load_module(pair, &path)
                        .context("Failed to parse module for IRContext")?
                },
                Rule::EOI => {},

                // Should be unreachable unless grammar is incorrect.
                _ => panic!("Unexpected token: {}", pair),
            };
        }

        Ok(context)
    }


    /// Loads the given pair as a module, loading child modules, functions, and
    /// structs as needed.
    ///
    /// If the pair cannot be loaded for any reason, and error is returned.
    fn load_module(&mut self, pair: Pair<Rule>, path: &[IRPathElement]) -> Result<()> {
        let mut pairs = pair.into_inner();
        let mut path = path.to_owned();

        let name = get_rule(&mut pairs, Rule::Identifier).unwrap();
        let name = name.as_str().to_string();

        let export = get_rule(&mut pairs, Rule::ExportKeyword).is_some();

        path.push(IRPathElement {
            identifier: name,
            exported:   export,
        });

        let module_body = get_rule(&mut pairs, Rule::ModuleBody).unwrap();
        for pair in module_body.into_inner() {
            match pair.as_rule() {
                Rule::Module => self.load_module(pair, &path)?,
                Rule::Function => self.load_function(pair, &path)?,
                Rule::Struct => self.load_struct(pair, &path)?,
                _ => panic!("Unexpected token: {}", pair),
            };
        }

        Ok(())
    }


    /// Loads the given pair as a function, loading child functions and structs
    /// as needed.
    ///
    /// If the pair cannot be loaded for any reason, and error is returned.
    fn load_function(&mut self, pair: Pair<Rule>, path: &[IRPathElement]) -> Result<()> {
        let (line, col) = pair.as_span().start_pos().line_col();
        let mut pairs = pair.into_inner();

        let name = get_rule(&mut pairs, Rule::Identifier).unwrap();
        let name = name.as_str().to_string();

        let export = get_rule(&mut pairs, Rule::ExportKeyword).is_some();
        let serial = get_rule(&mut pairs, Rule::SerialKeyword).is_some();

        let params = get_rule(&mut pairs, Rule::Params).unwrap();
        let params = params.into_inner().next().unwrap();
        let params = parse_arg_list(params);

        let returns = get_rule(&mut pairs, Rule::Return).unwrap();
        let returns = returns.into_inner().next().unwrap();
        let returns = parse_arg_list(returns);

        let mut functions = vec![];
        let mut structs = vec![];
        let mut assignments = vec![];

        let function_body = get_rule(&mut pairs, Rule::FunctionBody).unwrap();
        for pair in function_body.into_inner() {
            match pair.as_rule() {
                Rule::Function => self.load_function(pair, &path)?,
                Rule::Struct => self.load_struct(pair, &path)?,
                Rule::Assignment => assignments.push(parase_assignment(pair)),
                _ => panic!("Unexpected token: {}", pair),
            }
        }

        Ok(())
    }


    /// Loads the given pair as a struct.
    ///
    /// If the pair cannot be loaded for any reason, and error is returned.
    fn load_struct(&mut self, pair: Pair<Rule>, path: &[IRPathElement]) -> Result<()> {
        todo!();
    }


    /// Adds the given function to this context.
    ///
    /// If there is already another element with the same path, an error is
    /// returned.
    fn add_function(&mut self, function: IRFunction) -> Result<()> {
        // TODO: Check if identifier already exists.

        self.functions.push(function);
        Ok(())
    }


    /// Adds the given struct to this context.
    ///
    /// If there is already another element with the same path, an error is
    /// returned.
    fn add_struct(&mut self, structure: IRStruct) -> Result<()> {
        // TODO: Check if identifier already exits

        self.structs.push(structure);
        Ok(())
    }


    /// Gets a list of all functions within this context.
    pub fn get_functions(&self) -> &Vec<IRFunction> {
        &self.functions
    }
}

impl Default for IRContext {
    fn default() -> Self {
        Self::new()
    }
}
