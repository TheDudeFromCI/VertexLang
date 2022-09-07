extern crate pest;
use super::ast::*;
use super::CompilerError;
use ordered_float::OrderedFloat;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "compiler/grammar.pest"]
pub struct VertexLangParser;

pub fn parse(source: &str) -> Result<Node, CompilerError> {
    let pairs = VertexLangParser::parse(Rule::Program, source);
    let mut pairs = match pairs {
        Ok(p) => p,
        Err(e) => {
            return Err(CompilerError {
                message: e.to_string(),
            });
        }
    };

    let program = pairs.next().unwrap();
    let root = build_ast_from_expr(program);
    Ok(root)
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
        Rule::Float => Node::Float(OrderedFloat::from(pair.as_str().parse::<f64>().unwrap())),
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
    fn basic_math_order_of_operations() {
        assert_eq!(
            parse("1 + ~2.0 * 3").unwrap(),
            Node::BinaryExpr {
                op: Operator::Plus,
                lhs: Box::new(Node::Int(1)),
                rhs: Box::new(Node::BinaryExpr {
                    op: Operator::Multiply,
                    lhs: Box::new(Node::UnaryExpr {
                        op: Operator::BitwiseNegate,
                        child: Box::new(Node::Float(OrderedFloat(2.0)))
                    }),
                    rhs: Box::new(Node::Int(3))
                })
            }
        );
    }
}
