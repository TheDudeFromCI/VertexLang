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

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
  Int(i64),
  Float(f64),
  UnaryExpr {
    op: Operator,
    child: Box<Node>,
  },
  BinaryExpr {
    op: Operator,
    lhs: Box<Node>,
    rhs: Box<Node>,
  },
}

impl fmt::Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match &self {
      Node::Int(n) => write!(f, "{}", n),
      Node::Float(n) => write!(f, "{:.4}", n),
      Node::UnaryExpr { op, child } => write!(f, "({}{})", op, child),
      Node::BinaryExpr { op, lhs, rhs } => write!(f, "({} {} {})", lhs, op, rhs),
    }
  }
}
