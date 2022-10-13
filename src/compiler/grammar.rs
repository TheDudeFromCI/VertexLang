// Because Pest generates impl for VertexLangParser that doesn't have docs.
#[allow(missing_docs)]
mod peg {

    #[derive(Parser)]
    #[grammar = "grammars/vertex.pest"]
    pub struct VertexLangParser;
}

use super::nodes::*;
use peg::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::error::Error;


type Result<T> = std::result::Result<T, Box<dyn Error>>;


/// Parse the given source code into an abstract syntax tree of grammar nodes.
pub fn parse(source: &str) -> Result<ContextNode> {
    let mut pairs = VertexLangParser::parse(Rule::Program, source)?;
    let context = parse_context(pairs.next().unwrap());
    Ok(context)
}


fn parse_context(pair: Pair<Rule>) -> ContextNode {
    let mut modules = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Module => modules.push(parse_module(pair)),
            Rule::EOI => {},
            _ => panic!("Unexpected token: {}", pair),
        };
    }

    ContextNode {
        modules,
    }
}


fn parse_module(pair: Pair<Rule>) -> ModuleNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let export = is_rule_consume(&mut pairs, Rule::ExportKeyword);

    let mut modules = vec![];
    let mut functions = vec![];
    let mut structs = vec![];

    let module_body = get_rule_consume(&mut pairs, Rule::ModuleBody).unwrap();
    for pair in module_body.into_inner() {
        match pair.as_rule() {
            Rule::Module => modules.push(parse_module(pair)),
            Rule::Function => functions.push(parse_function(pair)),
            Rule::Struct => structs.push(parse_struct(pair)),
            _ => panic!("Unexpected token: {}", pair),
        };
    }

    ModuleNode {
        position: NodePosition {
            line,
            col,
        },
        name,
        export,
        modules,
        functions,
        structs,
    }
}


fn parse_function(pair: Pair<Rule>) -> FunctionNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let export = is_rule_consume(&mut pairs, Rule::ExportKeyword);
    let serial = is_rule_consume(&mut pairs, Rule::SerialKeyword);

    let params = get_rule_consume(&mut pairs, Rule::Params).unwrap();
    let params = params.into_inner().next().unwrap();
    let params = parse_arg_list(params);

    let returns = get_rule_consume(&mut pairs, Rule::Return).unwrap();
    let returns = returns.into_inner().next().unwrap();
    let returns = parse_arg_list(returns);

    let mut functions = vec![];
    let mut structs = vec![];
    let mut assignments = vec![];

    let function_body = get_rule_consume(&mut pairs, Rule::FunctionBody).unwrap();
    for pair in function_body.into_inner() {
        match pair.as_rule() {
            Rule::Function => functions.push(parse_function(pair)),
            Rule::Struct => structs.push(parse_struct(pair)),
            Rule::Assignment => assignments.push(parse_assignment(pair)),
            _ => panic!("Unexpected token: {}", pair),
        }
    }

    FunctionNode {
        position: NodePosition {
            line,
            col,
        },
        name,
        export,
        serial,
        params,
        returns,
        functions,
        structs,
        assignments,
    }
}


fn parse_struct(pair: Pair<Rule>) -> StructNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let export = is_rule_consume(&mut pairs, Rule::ExportKeyword);

    let fields = get_rule_consume(&mut pairs, Rule::StructBody).unwrap();
    let fields = parse_arg_list(fields);

    StructNode {
        position: NodePosition {
            line,
            col,
        },
        name,
        export,
        fields,
    }
}


fn parse_arg_list(pair: Pair<Rule>) -> ArgumentListNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut arguments = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Arg => arguments.push(parse_argument(pair)),
            _ => panic!("Unexpected token: {}", pair),
        };
    }

    ArgumentListNode {
        position: NodePosition {
            line,
            col,
        },
        arguments,
    }
}


fn parse_argument(pair: Pair<Rule>) -> ArgumentNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let dtype = get_rule_consume(&mut pairs, Rule::DataType).unwrap();
    let dtype = dtype.as_str().to_string();

    ArgumentNode {
        position: NodePosition {
            line,
            col,
        },
        name,
        dtype,
    }
}


fn parse_assignment(pair: Pair<Rule>) -> AssignmentNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let variable;
    if let Some(variable_pair) = get_rule_consume(&mut pairs, Rule::Identifier) {
        variable = Some(parse_variable(variable_pair));
    } else {
        variable = None;
    }

    let expression = get_rule_consume(&mut pairs, Rule::Expr).unwrap();
    let expression = parse_expression(expression);

    AssignmentNode {
        position: NodePosition {
            line,
            col,
        },
        variable,
        expression,
    }
}


fn parse_function_call(pair: Pair<Rule>) -> FunctionCallNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();

    let serial = is_rule_consume(&mut pairs, Rule::SerialKeyword);
    let external = is_rule_consume(&mut pairs, Rule::ExternKeyword);

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let expr_list = get_rule_consume(&mut pairs, Rule::ExprList).unwrap();
    let expr_list = parse_expression_list(expr_list);

    FunctionCallNode {
        position: NodePosition {
            line,
            col,
        },
        function_name: name,
        serial,
        external,
        arguments: expr_list,
    }
}


fn parse_expression_list(pair: Pair<Rule>) -> ExpressionListNode {
    let (line, col) = pair.as_span().start_pos().line_col();

    let mut expressions = vec![];
    for pair in pair.into_inner() {
        expressions.push(parse_expression(pair));
    }

    ExpressionListNode {
        position: NodePosition {
            line,
            col,
        },
        expressions,
    }
}


fn parse_expression(pair: Pair<Rule>) -> ExpressionNode {
    match pair.as_rule() {
        Rule::Expr => parse_expression(pair.into_inner().next().unwrap()),
        Rule::Int => ExpressionNode::IntLiteral(parse_integer(pair)),
        Rule::Float | Rule::ENotation => ExpressionNode::FloatLiteral(parse_float(pair)),
        Rule::String => ExpressionNode::StringLiteral(parse_string(pair)),
        Rule::Bool => ExpressionNode::BoolLiteral(parse_bool(pair)),
        Rule::FuncCall => ExpressionNode::FunctionCall(parse_function_call(pair)),
        Rule::Identifier => ExpressionNode::Variable(parse_variable(pair)),
        Rule::InnerVar => ExpressionNode::InnerVariable(parse_inner_variable(pair)),
        _ => panic!("Unexpected token: {}", pair),
    }
}


fn parse_integer(pair: Pair<Rule>) -> IntLiteralNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let value = pair.as_str().parse::<i64>().unwrap();

    IntLiteralNode {
        position: NodePosition {
            line,
            col,
        },
        value,
    }
}


fn parse_float(pair: Pair<Rule>) -> FloatLiteralNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let value = pair.as_str().parse::<f64>().unwrap();

    FloatLiteralNode {
        position: NodePosition {
            line,
            col,
        },
        value,
    }
}


fn parse_string(pair: Pair<Rule>) -> StringLiteralNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut pairs = pair.into_inner();
    let inner = get_rule_consume(&mut pairs, Rule::StringInterior).unwrap();

    StringLiteralNode {
        position: NodePosition {
            line,
            col,
        },
        value:    inner.as_str().to_string(),
    }
}


fn parse_bool(pair: Pair<Rule>) -> BoolLiteralNode {
    let (line, col) = pair.as_span().start_pos().line_col();
    let value = pair.as_str().parse::<bool>().unwrap();

    BoolLiteralNode {
        position: NodePosition {
            line,
            col,
        },
        value,
    }
}


fn parse_variable(pair: Pair<Rule>) -> VariableNode {
    let (line, col) = pair.as_span().start_pos().line_col();

    VariableNode {
        position: NodePosition {
            line,
            col,
        },
        name:     pair.as_str().to_string(),
    }
}


fn parse_inner_variable(pair: Pair<Rule>) -> InnerVariableNode {
    let (line, col) = pair.as_span().start_pos().line_col();

    let mut path = vec![];

    for pair in pair.into_inner() {
        if pair.as_rule() == Rule::Identifier {
            path.push(pair.as_str().to_string());
        } else {
            panic!("Unexpected token: {}", pair);
        }
    }

    InnerVariableNode {
        position: NodePosition {
            line,
            col,
        },
        path,
    }
}


/// Checks if the next element within the pairs iterator is of the given rule
/// type. If it is, then this function will automatically consume that rule.
fn is_rule_consume(pair: &mut Pairs<Rule>, rule: Rule) -> bool {
    match pair.peek() {
        Some(p) => {
            if p.as_rule() == rule {
                pair.next(); // Skip since we just checked it.
                true
            } else {
                false
            }
        },
        None => false,
    }
}


/// Checks if the next element within the pairs iterator is of the given rule
/// type. If it is, then that rule is returned. Otherwise, this function returns
/// None.
fn get_rule_consume<'a>(pair: &'a mut Pairs<Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    match pair.peek() {
        Some(p) => {
            if p.as_rule() == rule {
                pair.next(); // Skip since we just checked it.
                Some(p)
            } else {
                None
            }
        },
        None => None,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;


    #[test]
    fn hello_world() {
        let ast = parse(indoc! {r#"
            HelloWorld = mod {
                Main = export serial function {
                    params = ()
                    return = ()

                    serial Println("Hello, world!")
                }
            }
        "#});

        if let Err(e) = ast {
            println!("{}", e);
            panic!();
        }

        assert_eq!(ast.unwrap(), ContextNode {
            modules: vec![ModuleNode {
                position:  NodePosition {
                    line: 1,
                    col:  1,
                },
                name:      String::from("HelloWorld"),
                export:    false,
                modules:   vec![],
                functions: vec![FunctionNode {
                    position:    NodePosition {
                        line: 2,
                        col:  5,
                    },
                    name:        String::from("Main"),
                    export:      true,
                    serial:      true,
                    params:      ArgumentListNode {
                        position:  NodePosition {
                            line: 3,
                            col:  19,
                        },
                        arguments: vec![],
                    },
                    returns:     ArgumentListNode {
                        position:  NodePosition {
                            line: 4,
                            col:  19,
                        },
                        arguments: vec![],
                    },
                    functions:   vec![],
                    structs:     vec![],
                    assignments: vec![AssignmentNode {
                        position:   NodePosition {
                            line: 6,
                            col:  9,
                        },
                        variable:   None,
                        expression: ExpressionNode::FunctionCall(FunctionCallNode {
                            position:      NodePosition {
                                line: 6,
                                col:  9,
                            },
                            function_name: String::from("Println"),
                            serial:        true,
                            external:      false,
                            arguments:     ExpressionListNode {
                                position:    NodePosition {
                                    line: 6,
                                    col:  24,
                                },
                                expressions: vec![ExpressionNode::StringLiteral(
                                    StringLiteralNode {
                                        position: NodePosition {
                                            line: 6,
                                            col:  24,
                                        },
                                        value:    String::from("Hello, world!"),
                                    }
                                )],
                            },
                        }),
                    }],
                }],
                structs:   vec![],
            }],
        });
    }


    #[test]
    fn external_function() {
        let ast = parse(indoc! {r#"
            Module = mod {
                Main = function {
                    params = ()
                    return = ()

                    extern Println("Apple")
                }
            }
        "#});

        if let Err(e) = ast {
            println!("{}", e);
            panic!();
        }

        assert_eq!(ast.unwrap(), ContextNode {
            modules: vec![ModuleNode {
                position:  NodePosition {
                    line: 1,
                    col:  1,
                },
                name:      String::from("Module"),
                export:    false,
                modules:   vec![],
                functions: vec![FunctionNode {
                    position:    NodePosition {
                        line: 2,
                        col:  5,
                    },
                    name:        String::from("Main"),
                    export:      false,
                    serial:      false,
                    params:      ArgumentListNode {
                        position:  NodePosition {
                            line: 3,
                            col:  19,
                        },
                        arguments: vec![],
                    },
                    returns:     ArgumentListNode {
                        position:  NodePosition {
                            line: 4,
                            col:  19,
                        },
                        arguments: vec![],
                    },
                    functions:   vec![],
                    structs:     vec![],
                    assignments: vec![AssignmentNode {
                        position:   NodePosition {
                            line: 6,
                            col:  9,
                        },
                        variable:   None,
                        expression: ExpressionNode::FunctionCall(FunctionCallNode {
                            position:      NodePosition {
                                line: 6,
                                col:  9,
                            },
                            function_name: String::from("Println"),
                            serial:        false,
                            external:      true,
                            arguments:     ExpressionListNode {
                                position:    NodePosition {
                                    line: 6,
                                    col:  24,
                                },
                                expressions: vec![ExpressionNode::StringLiteral(
                                    StringLiteralNode {
                                        position: NodePosition {
                                            line: 6,
                                            col:  24,
                                        },
                                        value:    String::from("Apple"),
                                    }
                                )],
                            },
                        }),
                    }],
                }],
                structs:   vec![],
            }],
        })
    }
}
