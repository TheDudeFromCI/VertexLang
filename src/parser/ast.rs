/// A single node element within an abstract syntax tree instance.
#[derive(Debug, Clone, PartialEq)]
pub enum GrammarNode {
    /// A root-level context node, containing all root-level modules within a
    /// program execution instance.
    Context {
        /// A list of modules that are contained within this context node.
        modules: Vec<GrammarNode>,
    },

    /// A module acts like a namespace within Vertex, containing functions,
    /// structs, and other modules.
    Module {
        /// The name of this module.
        name: String,

        /// Whether or not this module is exported.
        export: bool,

        /// A list of nested modules that are contained within this module node.
        modules: Vec<GrammarNode>,

        /// A list of functions within this module.
        functions: Vec<GrammarNode>,

        /// A list of structs within this module.
        structs: Vec<GrammarNode>,
    },

    /// A function is a graph of executable actions or nodes that can be used to
    /// process data inputs into a new data output.
    Function {
        /// The name of this function.
        name: String,

        /// Whether or not this function is available to other modules.
        export: bool,

        /// Whether or not this function contains serial actions.
        serial: bool,

        /// The input arguments, or parameters, of this function.
        params: Box<GrammarNode>,

        /// The output arguments, or returns, of this function.
        returns: Box<GrammarNode>,

        /// A list of functions that are nested within this function.
        functions: Vec<GrammarNode>,

        /// A list of structs that are nested within this function.
        structs: Vec<GrammarNode>,

        /// An ordered list of variable assignments or serial function calls
        /// within this function.
        statements: Vec<GrammarNode>,
    },

    /// A specialized data type that organizes multiple other data primitives or
    /// structs into a single, more complex object.
    Struct {
        /// The name of this struct.
        name: String,

        /// Whether or not this struct is available to other modules.
        export: bool,

        /// The list of fields defined within this struct, stored as an argument
        /// list.
        fields: Box<GrammarNode>,
    },

    /// A named argument value, containing a variable name and data type.
    Argument {
        /// The name of this argument.
        name: String,

        /// The data type of this argument.
        dtype: String,
    },

    /// A list of named arguments definitions.
    ArgumentList {
        /// The list of arguments for the function, in order.
        arguments: Vec<GrammarNode>,
    },

    /// A variable result from an expression evaluation. Used to store data to
    /// pass into other expressions.
    Variable {
        /// The name of this variable.
        name: String,
    },

    /// An expression that reads a nested variable from within another value.
    /// Variables may be nested to any depth, with at least 1 nested access.
    InnerVariable {
        /// The nested variable path, where `path[0]` is the root-level
        /// variable, `path[1]` is the name of the field within that
        /// variable to access, `path[2]` is the name of the field within
        /// `path[1]`, and so on.
        path: Vec<String>,
    },

    /// A function call expression. This is an expression that takes in a set of
    /// expression arguments and sends them to another function to be processed.
    FunctionCall {
        /// The name of the function being called.
        function_name: String,

        /// Whether or not this function call is executed in serial.
        serial: bool,

        /// The list of expressions being passed into this function as
        /// arguments.
        arguments: Box<GrammarNode>,
    },

    /// A list of expressions, usually separated by commas.
    ExpressionList {
        /// The list of expressions within this node.
        expressions: Vec<GrammarNode>,
    },

    /// Represents a statement within a function that evaluates an expression
    /// and stores the result of that expression into a variable.
    Assignment {
        /// The variable being written to.
        variable: Box<GrammarNode>,

        /// The expression being evaluated.
        expression: Box<GrammarNode>,
    },

    /// An integer literal value expression.
    IntLiteral {
        /// The value of this literal.
        value: i64,
    },

    /// A floating point literal value expression.
    FloatLiteral {
        /// The value of this literal.
        value: f64,
    },

    /// A UTF-8 string literal value expression.
    StringLiteral {
        /// The value of this literal.
        value: String,
    },

    /// A boolean literal value expression.
    BoolLiteral {
        /// The value of this literal.
        value: bool,
    },
}
