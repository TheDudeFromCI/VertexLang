extern crate pest;
use super::ast::*;
use super::CompilerError;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "compiler/grammar.pest"]
pub struct CalcParser;

pub fn parse(source: &str) -> Result<Vec<Node>, CompilerError> {
  let mut ast: Vec<Node> = vec![];
  let pairs = CalcParser::parse(Rule::Program, source);

  let pairs = match pairs {
    Ok(p) => p,
    Err(e) => {
      return Err(CompilerError {
        message: e.to_string(),
      });
    }
  };

  for pair in pairs {
    if let Rule::Expr = pair.as_rule() {
      ast.push(build_ast_from_expr(pair));
    }
  }

  return Ok(ast);
}

fn build_ast_from_expr(pair: Pair<Rule>) -> Node {
  match pair.as_rule() {
    Rule::Expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
    Rule::L4 | Rule::L3 | Rule::L2 => {
      let mut pair = pair.into_inner();
      let mut lhs = build_ast_from_term(pair.next().unwrap());

      loop {
        if pair.peek().is_none() {
          return lhs;
        }

        let op = pair.next().unwrap();
        let rhs = build_ast_from_term(pair.next().unwrap());
        lhs = parse_binary_expr(op, lhs, rhs);
      }
    }
    Rule::L1 => {
      let mut pair = pair.into_inner();
      let op = pair.next().unwrap();
      let term = pair.next();
      if term.is_none() {
        // Op is the term, then.
        return build_ast_from_term(op);
      }

      let term = build_ast_from_term(term.unwrap());
      return parse_unary_expr(op, term);
    }
    Rule::Term => build_ast_from_term(pair.into_inner().next().unwrap()),
    unknown => panic!("Unknown expression: {:?}", unknown),
  }
}

fn build_ast_from_term(pair: Pair<Rule>) -> Node {
  match pair.as_rule() {
    Rule::Int => Node::Int(pair.as_str().parse::<i64>().unwrap()),
    Rule::Float => Node::Float(pair.as_str().parse::<f64>().unwrap()),
    _ => build_ast_from_expr(pair),
  }
}

fn parse_unary_expr(pair: Pair<Rule>, child: Node) -> Node {
  Node::UnaryExpr {
    op: Operator::from_str(pair.as_str()).unwrap(),
    child: Box::new(child),
  }
}

fn parse_binary_expr(pair: pest::iterators::Pair<Rule>, lhs: Node, rhs: Node) -> Node {
  Node::BinaryExpr {
    op: Operator::from_str(pair.as_str()).unwrap(),
    lhs: Box::new(lhs),
    rhs: Box::new(rhs),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn basics() {
    assert!(parse("b").is_err());
  }

  #[test]
  fn unary_expr() {
    let plus_one = parse("+1");
    assert!(plus_one.is_ok());
    assert_eq!(
      plus_one.clone().unwrap(),
      vec![Node::UnaryExpr {
        op: Operator::Plus,
        child: Box::new(Node::Int(1))
      }]
    );
    assert_eq!(format!("{}", plus_one.unwrap()[0]), "+1");

    let neg_two = parse("-2");
    assert!(neg_two.is_ok());
    assert_eq!(
      neg_two.clone().unwrap(),
      vec![Node::UnaryExpr {
        op: Operator::Minus,
        child: Box::new(Node::Int(2))
      }]
    );
    assert_eq!(format!("{}", neg_two.unwrap()[0]), "-2");
  }
  #[test]
  fn binary_expr() {
    let sum = parse("1 + 2");
    assert!(sum.is_ok());
    assert_eq!(
      sum.clone().unwrap(),
      vec![Node::BinaryExpr {
        op: Operator::Plus,
        lhs: Box::new(Node::Int(1)),
        rhs: Box::new(Node::Int(2))
      }]
    );
    assert_eq!(format!("{}", sum.unwrap()[0]), "1 + 2");
    let minus = parse("1   -  \t  2");
    assert!(minus.is_ok());
    assert_eq!(
      minus.clone().unwrap(),
      vec![Node::BinaryExpr {
        op: Operator::Minus,
        lhs: Box::new(Node::Int(1)),
        rhs: Box::new(Node::Int(2))
      }]
    );
    assert_eq!(format!("{}", minus.unwrap()[0]), "1 - 2");
    // fails as there's no rhs:
    // let paran_sum = parse("(1 + 2)");
    // assert!(paran_sum.is_ok());
  }

  #[test]
  fn nested_expr() {
    fn test_expr(expected: &str, src: &str) {
      assert_eq!(
        expected,
        parse(src)
          .unwrap()
          .iter()
          .fold(String::new(), |acc, arg| acc + &format!("{}", &arg))
      );
    }

    test_expr("1 + 2 + 3", "(1 + 2) + 3");
    test_expr("1 + 2 + 3", "1 + (2 + 3)");
    test_expr("1 + 2 + 3 + 4", "1 + (2 + (3 + 4))");
    test_expr("1 + 2 + 3 - 4", "(1 + 2) + (3 - 4)");
  }

  #[test]
  fn multiple_operators() {
    assert_eq!(
      parse("1+2+3").unwrap(),
      vec![Node::BinaryExpr {
        op: Operator::Plus,
        lhs: Box::new(Node::BinaryExpr {
          op: Operator::Plus,
          lhs: Box::new(Node::Int(1)),
          rhs: Box::new(Node::Int(2)),
        }),
        rhs: Box::new(Node::Int(3)),
      }]
    )
  }
}
