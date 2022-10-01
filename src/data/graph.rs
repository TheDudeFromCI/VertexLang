//! A node-graph data structure implementation.
//!
//! The purpose of this module is solely to store a node-graph data structure
//! and does not make any attempt to execute or process the nodes within the
//! graph. This module is intended to serve as an in-between state for
//! processing a graph.


use crate::data::DataType;
use std::rc::Rc;


/// An input or output argument for a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    name:  String,
    dtype: DataType,
}


impl Argument {
    /// Creates a new argument instance from the given variable name and data
    /// type.
    pub fn new(name: String, dtype: DataType) -> Self {
        Argument {
            name,
            dtype,
        }
    }


    /// Gets the name of this argument.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }


    /// Gets the data type of this argument.
    pub fn get_type(&self) -> &DataType {
        &self.dtype
    }
}


/// A single function call within a graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    name:    String,
    node_id: usize,
    inputs:  Vec<Argument>,
    outputs: Vec<Argument>,
}


impl Function {
    /// Creates a new function instance from the given function name, node id,
    /// input arguments, and output arguments.
    pub fn new(
        name: String, node_id: usize, inputs: Vec<Argument>, outputs: Vec<Argument>,
    ) -> Self {
        Function {
            name,
            node_id,
            inputs,
            outputs,
        }
    }


    /// Gets the name of this function.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }


    /// Gets the unique node id for this function instance.
    pub fn get_node_id(&self) -> usize {
        self.node_id
    }


    /// Gets the input arguments for this function.
    pub fn get_inputs(&self) -> &Vec<Argument> {
        &self.inputs
    }


    /// Gets the output arguments for this function.
    pub fn get_outputs(&self) -> &Vec<Argument> {
        &self.outputs
    }
}


/// A connection between the output arguments of one function to the input
/// arguments of another function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    start_fn:  Rc<Function>,
    start_arg: usize,
    end_fn:    Rc<Function>,
    end_arg:   usize,
}


impl Connection {
    /// Creates a new connection instance between the given output argument
    /// index of the start function and the input argument index of the end
    /// function.
    pub fn new(
        start_fn: Rc<Function>, start_arg: usize, end_fn: Rc<Function>, end_arg: usize,
    ) -> Self {
        Connection {
            start_fn,
            start_arg,
            end_fn,
            end_arg,
        }
    }


    /// Gets the starting function and the corresponding output argument index
    /// of this connection.
    pub fn get_start(&self) -> (Rc<Function>, usize) {
        (self.start_fn.clone(), self.start_arg)
    }


    /// Gets the ending function and the corresponding input argument index of
    /// this connection.
    pub fn get_end(&self) -> (Rc<Function>, usize) {
        (self.end_fn.clone(), self.end_arg)
    }
}


/// A feed-forward network of function nodes, connecting the outputs of one
/// function node to the inputs of another function node in order to process
/// data in a lambda-style way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    params:      Rc<Function>,
    returns:     Rc<Function>,
    nodes:       Vec<Rc<Function>>,
    connections: Vec<Rc<Connection>>,
}


impl Graph {
    /// Creates a new graph instance with the given input and output function
    /// definitions.
    pub fn new(params: Function, returns: Function) -> Self {
        Graph {
            params:      Rc::new(params),
            returns:     Rc::new(returns),
            nodes:       vec![],
            connections: vec![],
        }
    }


    /// Gets the input arguments function for this graph.
    pub fn get_params(&self) -> Rc<Function> {
        self.params.clone()
    }


    /// Gets the output arguments function for this graph.
    pub fn get_returns(&self) -> Rc<Function> {
        self.returns.clone()
    }


    /// Adds a new function to this graph and returns an Rc reference to it.
    pub fn add_function(&mut self, func: Function) -> Rc<Function> {
        let func = Rc::new(func);
        self.nodes.push(func.clone());
        func
    }


    /// Adds a new connection to this graph and returns an Rc reference to it.
    pub fn add_connection(&mut self, connection: Connection) -> Rc<Connection> {
        let connection = Rc::new(connection);
        self.connections.push(connection.clone());
        connection
    }


    /// Gets the direct parent function nodes for the given function.
    pub fn get_parent_functions(&self, func: Rc<Function>) -> Vec<Rc<Function>> {
        let mut parents = vec![];

        for conn in &self.connections {
            let (end_fn, _) = conn.get_end();
            if end_fn == func {
                let (start_fn, _) = conn.get_start();
                parents.push(start_fn);
            }
        }

        parents
    }


    /// Gets the direct child function nodes for the given function.
    pub fn get_child_functions(&self, func: Rc<Function>) -> Vec<Rc<Function>> {
        let mut children = vec![];

        for conn in &self.connections {
            let (start_fn, _) = conn.get_start();
            if start_fn == func {
                let (end_fn, _) = conn.get_end();
                children.push(end_fn);
            }
        }

        children
    }


    /// Gets a list of the functions in this graph.
    ///
    /// This list *does not* include the params and returns meta functions.
    pub fn get_functions(&self) -> &Vec<Rc<Function>> {
        &self.nodes
    }


    /// Gets a list of all connections in this graph.
    pub fn get_connections(&self) -> &Vec<Rc<Connection>> {
        &self.connections
    }
}
