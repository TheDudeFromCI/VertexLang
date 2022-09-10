extern crate pest;
use super::ast::*;
use super::CompilerError;
use crate::context::DataType;
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
        Rule::ExprList => {
            let mut exprs = vec![];
            for pair in pair.into_inner() {
                exprs.push(build_ast_from_expr(pair));
            }
            Node::ExprList { exprs }
        }
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
            parse_unary_expr(op, term)
        }
        Rule::Term => build_ast_from_term(pair.into_inner().next().unwrap()),
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn build_ast_from_term(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Int => Node::Int(pair.as_str().parse::<i64>().unwrap()),
        Rule::Float | Rule::ENotation => {
            Node::Float(OrderedFloat::from(pair.as_str().parse::<f64>().unwrap()))
        }
        Rule::String => Node::String(pair.into_inner().as_str().to_owned()),
        Rule::Boolean => Node::Bool(pair.as_str().parse::<bool>().unwrap()),
        Rule::Function => parse_function_expr(pair),
        _ => build_ast_from_expr(pair),
    }
}

fn parse_function_expr(pair: Pair<Rule>) -> Node {
    let mut pair = pair.into_inner();
    let name = pair.next().unwrap().as_str().to_owned();

    let expr_list = pair.next();
    let expr_list_node = match expr_list {
        Some(e) => build_ast_from_expr(e),
        None => Node::ExprList { exprs: vec![] },
    };

    Node::Function {
        name,
        params: Box::new(expr_list_node),
        rtype: DataType::Unknown,
    }
}

fn parse_unary_expr(pair: Pair<Rule>, child: Node) -> Node {
    Node::UnaryExpr {
        op: Operator::from_str(pair.as_str()).unwrap(),
        child: Box::new(child),
        rtype: DataType::Unknown,
    }
}

fn parse_binary_expr(pair: pest::iterators::Pair<Rule>, lhs: Node, rhs: Node) -> Node {
    Node::BinaryExpr {
        op: Operator::from_str(pair.as_str()).unwrap(),
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        rtype: DataType::Unknown,
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
                        child: Box::new(Node::Float(OrderedFloat(2.0))),
                        rtype: DataType::Unknown,
                    }),
                    rhs: Box::new(Node::Int(3)),
                    rtype: DataType::Unknown,
                }),
                rtype: DataType::Unknown,
            }
        );
    }

    #[test]
    fn parse_function_expr() {
        assert_eq!(
            parse("pi() * max(2 + 3, 2.0)").unwrap(),
            Node::BinaryExpr {
                op: Operator::Multiply,
                lhs: Box::new(Node::Function {
                    name: String::from("pi"),
                    params: Box::new(Node::ExprList { exprs: vec![] }),
                    rtype: DataType::Unknown,
                }),
                rhs: Box::new(Node::Function {
                    name: String::from("max"),
                    params: Box::new(Node::ExprList {
                        exprs: vec![
                            Node::BinaryExpr {
                                op: Operator::Plus,
                                lhs: Box::new(Node::Int(2)),
                                rhs: Box::new(Node::Int(3)),
                                rtype: DataType::Unknown,
                            },
                            Node::Float(OrderedFloat(2.0))
                        ]
                    }),
                    rtype: DataType::Unknown,
                }),
                rtype: DataType::Unknown,
            }
        );
    }

    #[test]
    fn parse_constants() {
        assert_eq!(
            parse("' hi there '").unwrap(),
            Node::String(String::from(" hi there "))
        );
        assert_eq!(parse("true").unwrap(), Node::Bool(true));
        assert_eq!(parse("13").unwrap(), Node::Int(13));
        assert_eq!(parse("`pink`").unwrap(), Node::String(String::from("pink")));
        assert_eq!(parse("\"red\"").unwrap(), Node::String(String::from("red")));
        assert_eq!(parse("0.2").unwrap(), Node::Float(OrderedFloat(0.2)));
        assert_eq!(parse("1.").unwrap(), Node::Float(OrderedFloat(1.0)));
        assert_eq!(parse(".64").unwrap(), Node::Float(OrderedFloat(0.64)));
        assert_eq!(parse("2.5e2").unwrap(), Node::Float(OrderedFloat(250.0)));
        assert_eq!(parse("1E-2").unwrap(), Node::Float(OrderedFloat(0.01)));
        assert_eq!(parse("'\n'").unwrap(), Node::String(String::from("\n")));
        assert_eq!(
            parse("'\\u1230'").unwrap(),
            Node::String(String::from("\\u1230"))
        );
        parse("'\\q'").unwrap_err();
    }
}
