use crate::context::DataType;
use ordered_float::OrderedFloat;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    IntDivide,
    Modulus,
    Power,
    BooleanNot,
    BitwiseNegate,
}

impl Operator {
    pub fn from_str(op: &str) -> Option<Operator> {
        match op {
            "+" => Some(Operator::Plus),
            "-" => Some(Operator::Minus),
            "*" => Some(Operator::Multiply),
            "/" => Some(Operator::Divide),
            "//" => Some(Operator::IntDivide),
            "%" => Some(Operator::Modulus),
            "**" => Some(Operator::Power),
            "!" => Some(Operator::BooleanNot),
            "~" => Some(Operator::BitwiseNegate),
            _ => None,
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::IntDivide => write!(f, "//"),
            Operator::Modulus => write!(f, "%"),
            Operator::Power => write!(f, "**"),
            Operator::BooleanNot => write!(f, "!"),
            Operator::BitwiseNegate => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Bool(bool),
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
        rtype: DataType,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
        rtype: DataType,
    },
    ExprList {
        exprs: Vec<Node>,
    },
    Function {
        name: String,
        params: Box<Node>,
        rtype: DataType,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Node::Int(n) => write!(f, "{}", n),
            Node::Float(n) => write!(f, "{:.4}", n),
            Node::String(s) => write!(f, "\"{}\"", s),
            Node::Bool(b) => write!(f, "{}", b),
            Node::UnaryExpr {
                op,
                child,
                rtype: _,
            } => write!(f, "({}{})", op, child),
            Node::BinaryExpr {
                op,
                lhs,
                rhs,
                rtype: _,
            } => write!(f, "({} {} {})", lhs, op, rhs),
            Node::ExprList { exprs } => write!(f, "{}", format_params(exprs)),
            Node::Function {
                name,
                params,
                rtype: _,
            } => write!(f, "{}({})", name, params),
        }
    }
}

fn format_params(params: &Vec<Node>) -> String {
    let mut s = String::new();

    for param in params {
        if !s.is_empty() {
            s.push_str(", ");
        }
        s.push_str(param.to_string().as_str())
    }

    s
}
