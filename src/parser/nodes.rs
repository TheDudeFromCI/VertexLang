//! A collection of node definitions that make up the abstract syntax tree for
//! Vertex.


/// Contains the line number and column number of a node within the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePosition {
    /// The line number this node is defined on.
    pub line: usize,

    /// The column number this node is defined on.
    pub col: usize,
}


/// A root-level context node, containing all root-level modules within a
/// program execution instance.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextNode {
    /// A list of modules that are contained within this context node.
    pub modules: Vec<ModuleNode>,
}


/// A module acts like a namespace within Vertex, containing functions,
/// structs, and other modules.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of this module.
    pub name: String,

    /// Whether or not this module is exported.
    pub export: bool,

    /// A list of nested modules that are contained within this module node.
    pub modules: Vec<ModuleNode>,

    /// A list of functions within this module.
    pub functions: Vec<FunctionNode>,

    /// A list of structs within this module.
    pub structs: Vec<StructNode>,
}


/// A function is a graph of executable actions or nodes that can be used to
/// process data inputs into a new data output.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of this function.
    pub name: String,

    /// Whether or not this function is available to other modules.
    pub export: bool,

    /// Whether or not this function contains serial actions.
    pub serial: bool,

    /// The input arguments, or parameters, of this function.
    pub params: ArgumentListNode,

    /// The output arguments, or returns, of this function.
    pub returns: ArgumentListNode,

    /// A list of functions that are nested within this function.
    pub functions: Vec<FunctionNode>,

    /// A list of structs that are nested within this function.
    pub structs: Vec<StructNode>,

    /// A list of variable assignment operations within this function.
    pub assignments: Vec<AssignmentNode>,
}


/// A specialized data type that organizes multiple other data primitives or
/// structs into a single, more complex object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of this struct.
    pub name: String,

    /// Whether or not this struct is available to other modules.
    pub export: bool,

    /// The list of fields defined within this struct, stored as an argument
    /// list.
    pub fields: ArgumentListNode,
}


/// A named argument value, containing a variable name and data type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of this argument.
    pub name: String,

    /// The data type of this argument.
    pub dtype: String,
}


/// A list of named arguments definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentListNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The list of arguments for the function, in order.
    pub arguments: Vec<ArgumentNode>,
}


/// A variable result from an expression evaluation. Used to store data to
/// pass into other expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of this variable.
    pub name: String,
}


/// An expression that reads a nested variable from within another value.
/// Variables may be nested to any depth, with at least 1 nested access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerVariableNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The nested variable path, where `path[0]` is the root-level
    /// variable, `path[1]` is the name of the field within that
    /// variable to access, `path[2]` is the name of the field within
    /// `path[1]`, and so on.
    pub path: Vec<String>,
}


/// A function call expression. This is an expression that takes in a set of
/// expression arguments and sends them to another function to be processed.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The name of the function being called.
    pub function_name: String,

    /// Whether or not this function call is executed in serial.
    pub serial: bool,

    /// Whether or not this function is an internal function or an external
    /// function.
    pub external: bool,

    /// The list of expressions being passed into this function as
    /// arguments.
    pub arguments: ExpressionListNode,
}


/// A list of expressions, usually separated by commas.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionListNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The list of expressions within this node.
    pub expressions: Vec<ExpressionNode>,
}


/// Represents a statement within a function that evaluates an expression
/// and stores the result of that expression into a variable.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The variable being written to.
    pub variable: Option<VariableNode>,

    /// The expression being evaluated.
    pub expression: ExpressionNode,
}


/// An integer literal value expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntLiteralNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The value of this literal.
    pub value: i64,
}


/// A floating point literal value expression.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteralNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The value of this literal.
    pub value: f64,
}


/// A UTF-8 string literal value expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteralNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The value of this literal.
    pub value: String,
}


/// A boolean literal value expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolLiteralNode {
    /// The position of this node within the source code.
    pub position: NodePosition,

    /// The value of this literal.
    pub value: bool,
}


/// Contains an expression that can be resolved.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    /// An integer literal value expression.
    IntLiteral(IntLiteralNode),

    /// A float literal value expression.
    FloatLiteral(FloatLiteralNode),

    /// A string literal value expression.
    StringLiteral(StringLiteralNode),

    /// A boolean literal value expression.
    BoolLiteral(BoolLiteralNode),

    /// A function call expression.
    FunctionCall(FunctionCallNode),

    /// A variable value expression.
    Variable(VariableNode),

    /// A nested variable value expression.
    InnerVariable(InnerVariableNode),
}
