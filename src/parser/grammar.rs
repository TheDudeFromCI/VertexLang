//! Loads the VertexLang grammar through Pest.

// Because Pest generates impl for VertexLangParser that doesn't have docs.
#![allow(missing_docs)]

use super::GrammarNode;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::error::Error;
use thiserror::Error as ThisError;


#[derive(Parser)]
#[grammar = "grammars/vertex.pest"]
struct VertexLangParser;


type Result<T> = std::result::Result<T, Box<dyn Error>>;


pub fn parse(source: &str) -> Result<GrammarNode> {
    let mut pairs = VertexLangParser::parse(Rule::Program, source)?;
    let context = parse_context(pairs.next().unwrap())?;
    Ok(context)
}


fn parse_context(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut modules = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Module => modules.push(parse_module(pair)?),
            Rule::EOI => {},
            _ => return unexpected_token(&pair),
        };
    }

    Ok(GrammarNode::Context {
        modules,
    })
}


fn parse_module(pair: Pair<Rule>) -> Result<GrammarNode> {
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
            Rule::Module => modules.push(parse_module(pair)?),
            Rule::Function => functions.push(parse_function(pair)?),
            Rule::Struct => structs.push(parse_struct(pair)?),
            _ => return unexpected_token(&pair),
        };
    }

    Ok(GrammarNode::Module {
        name,
        export,
        modules,
        functions,
        structs,
    })
}


fn parse_function(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let export = is_rule_consume(&mut pairs, Rule::ExportKeyword);
    let serial = is_rule_consume(&mut pairs, Rule::SerialKeyword);

    let params = get_rule_consume(&mut pairs, Rule::Params).unwrap();
    let params = params.into_inner().next().unwrap();
    let params = Box::new(parse_arg_list(params)?);

    let returns = get_rule_consume(&mut pairs, Rule::Return).unwrap();
    let returns = returns.into_inner().next().unwrap();
    let returns = Box::new(parse_arg_list(returns)?);

    let mut functions = vec![];
    let mut structs = vec![];
    let mut statements = vec![];

    let function_body = get_rule_consume(&mut pairs, Rule::FunctionBody).unwrap();
    for pair in function_body.into_inner() {
        match pair.as_rule() {
            Rule::Function => functions.push(parse_struct(pair)?),
            Rule::Struct => structs.push(parse_struct(pair)?),
            Rule::Assignment => statements.push(parse_assignment(pair)?),
            Rule::FuncCall => statements.push(parse_function_call(pair)?),
            _ => return unexpected_token(&pair),
        }
    }

    Ok(GrammarNode::Function {
        name,
        export,
        serial,
        params,
        returns,
        functions,
        structs,
        statements,
    })
}


fn parse_struct(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let export = is_rule_consume(&mut pairs, Rule::ExportKeyword);

    let fields = get_rule_consume(&mut pairs, Rule::StructBody).unwrap();
    let fields = Box::new(parse_arg_list(fields)?);

    Ok(GrammarNode::Struct {
        name,
        export,
        fields,
    })
}


fn parse_arg_list(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut arguments = vec![];

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Arg => arguments.push(parse_argument(pair)?),
            _ => return unexpected_token(&pair),
        };
    }

    Ok(GrammarNode::ArgumentList {
        arguments,
    })
}


fn parse_argument(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let dtype = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let dtype = dtype.as_str().to_string();

    Ok(GrammarNode::Argument {
        name,
        dtype,
    })
}


fn parse_assignment(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let variable = GrammarNode::Variable {
        name: name.as_str().to_string(),
    };

    let expr = get_rule_consume(&mut pairs, Rule::Expr).unwrap();
    let expr = parse_expression(expr)?;

    Ok(GrammarNode::Assignment {
        variable:   Box::new(variable),
        expression: Box::new(expr),
    })
}


fn parse_function_call(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let serial = is_rule_consume(&mut pairs, Rule::SerialKeyword);

    let name = get_rule_consume(&mut pairs, Rule::Identifier).unwrap();
    let name = name.as_str().to_string();

    let expr_list = get_rule_consume(&mut pairs, Rule::ExprList).unwrap();
    let expr_list = parse_expression_list(expr_list)?;

    Ok(GrammarNode::FunctionCall {
        function_name: name,
        serial,
        arguments: Box::new(expr_list),
    })
}


fn parse_expression_list(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut expressions = vec![];
    for pair in pair.into_inner() {
        expressions.push(parse_expression(pair)?);
    }

    Ok(GrammarNode::ExpressionList {
        expressions,
    })
}


fn parse_expression(pair: Pair<Rule>) -> Result<GrammarNode> {
    match pair.as_rule() {
        Rule::Expr => parse_expression(pair.into_inner().next().unwrap()),
        Rule::Int => parse_integer(pair),
        Rule::Float | Rule::ENotation => parse_float(pair),
        Rule::String => parse_string(pair),
        Rule::Bool => parse_bool(pair),
        Rule::FuncCall => parse_function_call(pair),
        Rule::Identifier => parse_variable(pair),
        Rule::InnerVar => parse_inner_variable(pair),
        _ => unexpected_token(&pair),
    }
}


fn parse_integer(pair: Pair<Rule>) -> Result<GrammarNode> {
    let value = pair.as_str().parse::<i64>().unwrap();
    Ok(GrammarNode::IntLiteral {
        value,
    })
}


fn parse_float(pair: Pair<Rule>) -> Result<GrammarNode> {
    let value = pair.as_str().parse::<f64>().unwrap();
    Ok(GrammarNode::FloatLiteral {
        value,
    })
}


fn parse_string(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut pairs = pair.into_inner();

    let inner = get_rule_consume(&mut pairs, Rule::StringInterior).unwrap();
    Ok(GrammarNode::StringLiteral {
        value: inner.as_str().to_string(),
    })
}


fn parse_bool(pair: Pair<Rule>) -> Result<GrammarNode> {
    let value = pair.as_str().parse::<bool>().unwrap();
    Ok(GrammarNode::BoolLiteral {
        value,
    })
}


fn parse_variable(pair: Pair<Rule>) -> Result<GrammarNode> {
    Ok(GrammarNode::Variable {
        name: pair.as_str().to_string(),
    })
}


fn parse_inner_variable(pair: Pair<Rule>) -> Result<GrammarNode> {
    let mut path = vec![];

    for pair in pair.into_inner() {
        if pair.as_rule() == Rule::Identifier {
            path.push(pair.as_str().to_string());
        } else {
            return unexpected_token(&pair);
        }
    }

    Ok(GrammarNode::InnerVariable {
        path,
    })
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


#[derive(ThisError, Debug)]
pub enum ParsingError {
    #[error("Unexpected {0} token at {1}:{2}.")]
    UnexpectedToken(String, usize, usize),
}


fn unexpected_token<T>(pair: &Pair<Rule>) -> std::result::Result<T, Box<dyn Error>> {
    let token = format!("{:?}", pair.as_rule());
    let (line, col) = pair.as_span().start_pos().line_col();
    Err(Box::new(ParsingError::UnexpectedToken(token, line, col)))
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

        assert_eq!(ast.unwrap(), GrammarNode::Context {
            modules: vec![GrammarNode::Module {
                name:      String::from("HelloWorld"),
                export:    false,
                modules:   vec![],
                functions: vec![GrammarNode::Function {
                    name:       String::from("Main"),
                    export:     true,
                    serial:     true,
                    params:     Box::new(GrammarNode::ArgumentList {
                        arguments: vec![],
                    }),
                    returns:    Box::new(GrammarNode::ArgumentList {
                        arguments: vec![],
                    }),
                    functions:  vec![],
                    structs:    vec![],
                    statements: vec![GrammarNode::FunctionCall {
                        function_name: String::from("Println"),
                        serial:        true,
                        arguments:     Box::new(GrammarNode::ExpressionList {
                            expressions: vec![GrammarNode::StringLiteral {
                                value: String::from("Hello, world!"),
                            }],
                        }),
                    }],
                }],
                structs:   vec![],
            }],
        });
    }
}
