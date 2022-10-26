use super::errors::{CompilerError, IRError};
use super::{IRContext, IRDataType, IRFuncCall, IRFunction, IRNode, IRNodeInput, IRStruct};
use crate::compiler::ast::*;
use crate::runtime::registry::FunctionRegistry;
use std::cmp::{min, Ordering};


/// Compiles an AST ContextNode into an intermediate representation.
pub fn compile_context(
    context: ContextNode, function_registry: &FunctionRegistry,
) -> Result<IRContext, CompilerError> {
    let mut ir_context = IRContext::new();
    let path = vec![];

    // Load all structs and function headers, first
    for module in context.modules {
        load_module(&mut ir_context, &path, &module, 0, 0, function_registry)?;
    }

    resolve_function_calls(&mut ir_context);

    Ok(ir_context)
}


fn load_module(
    context: &mut IRContext, path: &[String], module: &ModuleNode, mut depth: u32,
    mut accessability: u32, function_registry: &FunctionRegistry,
) -> Result<(), CompilerError> {
    let mut path = path.to_owned();
    path.push(module.name.clone());

    depth += 1;
    if !module.export {
        accessability = depth;
    }

    for nested_module in &module.modules {
        load_module(
            context,
            &path,
            nested_module,
            depth,
            accessability,
            function_registry,
        )?;
    }

    for nested_function in &module.functions {
        load_function(
            context,
            &path,
            nested_function,
            depth,
            accessability,
            function_registry,
        )?;
    }

    for nested_struct in &module.structs {
        load_struct(context, &path, nested_struct, accessability)?;
    }

    Ok(())
}


fn load_function(
    context: &mut IRContext, path: &[String], function: &FunctionNode, mut depth: u32,
    mut accessability: u32, function_registry: &FunctionRegistry,
) -> Result<(), CompilerError> {
    let mut path = path.to_owned();
    path.push(function.name.clone());

    depth += 1;
    if !function.export {
        accessability = depth;
    }

    for nested_function in &function.functions {
        load_function(
            context,
            &path,
            nested_function,
            depth,
            accessability,
            function_registry,
        )?;
    }

    for nested_struct in &function.structs {
        load_struct(context, &path, nested_struct, accessability)?;
    }

    let mut inputs = vec![];
    for param in &function.params.arguments {
        inputs.push(IRDataType::from(&param.dtype));
    }

    let mut outputs = vec![];
    for returned in &function.returns.arguments {
        outputs.push(IRDataType::from(&returned.dtype));
    }

    let output;
    if outputs.is_empty() {
        output = IRDataType::Null;
    } else if outputs.len() == 1 {
        output = outputs.pop().unwrap();
    } else {
        output = IRDataType::Tuple(outputs);
    }

    let statements = parse_function_statements(function, function_registry)?;
    let ir_function = IRFunction::new(path, accessability, inputs, output, statements);
    context.add_function(ir_function);

    Ok(())
}


fn load_struct(
    context: &mut IRContext, path: &[String], structure: &StructNode, accessability: u32,
) -> Result<(), CompilerError> {
    let mut path = path.to_owned();
    path.push(structure.name.clone());

    let mut ir_struct = IRStruct::new(path, accessability);
    for field in &structure.fields.arguments {
        let res = ir_struct.add_field(field.name.clone(), IRDataType::from(field.dtype.as_str()));
        if let Err(e) = res {
            return Err(CompilerError::new(field.position.clone(), e));
        }
    }

    context.add_struct(ir_struct);
    Ok(())
}


fn parse_function_statements(
    function: &FunctionNode, function_registry: &FunctionRegistry,
) -> Result<Vec<IRNode>, CompilerError> {
    let mut nodes = vec![];
    verify_no_circular_deps(&function.assignments)?;

    let mut assignments = function.assignments.clone();
    assignments.sort_by(|a, b| {
        if a.variable.is_none() || b.variable.is_none() {
            Ordering::Equal
        } else if expression_contains_variable(&b.expression, a.variable.as_ref().unwrap()) {
            Ordering::Less
        } else if expression_contains_variable(&a.expression, b.variable.as_ref().unwrap()) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    let params = &function.params.arguments;

    for assignment in &function.assignments {
        parse_expression_into_nodes(
            &assignment.expression,
            &mut nodes,
            &assignments,
            params,
            function_registry,
        )?;
    }

    Ok(nodes)
}


fn parse_expression_into_nodes(
    expr: &ExpressionNode, nodes: &mut Vec<IRNode>, assignments: &[AssignmentNode],
    params: &[ArgumentNode], function_registry: &FunctionRegistry,
) -> Result<IRNodeInput, CompilerError> {
    let node = match expr {
        ExpressionNode::IntLiteral(v) => {
            IRNode::new(IRFuncCall::IntConstant(v.value), vec![], IRDataType::Int)
        },
        ExpressionNode::FloatLiteral(v) => {
            IRNode::new(
                IRFuncCall::FloatConstant(v.value),
                vec![],
                IRDataType::Float,
            )
        },
        ExpressionNode::StringLiteral(v) => {
            IRNode::new(
                IRFuncCall::StringConstant(v.value.clone()),
                vec![],
                IRDataType::String,
            )
        },
        ExpressionNode::BoolLiteral(v) => {
            IRNode::new(IRFuncCall::BoolConstant(v.value), vec![], IRDataType::Bool)
        },
        ExpressionNode::Variable(v) => {
            let param_pos = params.iter().position(|a| a.name.eq(&v.name));
            let hidden_pos = assignments
                .iter()
                .filter_map(|a| a.variable.as_ref())
                .position(|a| a.name.eq(&v.name));

            if let Some(p) = param_pos {
                return Ok(IRNodeInput::FunctionParam(p as u32));
            } else if let Some(p) = hidden_pos {
                return Ok(IRNodeInput::HiddenNode(p as u32));
            } else {
                return Err(CompilerError::new(
                    v.position.clone(),
                    IRError::UnknownIdentifier(v.name.clone()),
                ));
            }
        },
        ExpressionNode::InnerVariable(_) => todo!(),
        ExpressionNode::FunctionCall(f) => {
            let mut inputs = vec![];
            for arg_expr in &f.arguments.expressions {
                inputs.push(parse_expression_into_nodes(
                    arg_expr,
                    nodes,
                    assignments,
                    params,
                    function_registry,
                )?);
            }

            if f.external {
                if let Some(ext_func) = function_registry.get_function(&f.function_name) {
                    IRNode::new(
                        IRFuncCall::External(f.function_name.clone()),
                        inputs,
                        ext_func.get_output().clone(),
                    )
                } else {
                    return Err(CompilerError::new(
                        f.position.clone(),
                        IRError::UnknownIdentifier(f.function_name.clone()),
                    ));
                }
            } else {
                IRNode::new(
                    IRFuncCall::Unresolved(f.function_name.clone()),
                    inputs,
                    IRDataType::Unknown,
                )
            }
        },
    };

    let len = nodes.len();
    nodes.push(node);
    Ok(IRNodeInput::HiddenNode(len as u32))
}


fn expression_contains_variable(expr: &ExpressionNode, var: &VariableNode) -> bool {
    match expr {
        ExpressionNode::IntLiteral(_) => false,
        ExpressionNode::FloatLiteral(_) => false,
        ExpressionNode::StringLiteral(_) => false,
        ExpressionNode::BoolLiteral(_) => false,
        ExpressionNode::Variable(v) => v.name.eq(&var.name),
        ExpressionNode::InnerVariable(v) => v.path[0].eq(&var.name),
        ExpressionNode::FunctionCall(f) => {
            f.arguments.expressions.iter().any(|a| expression_contains_variable(a, var))
        },
    }
}


fn verify_no_circular_deps(_assignments: &[AssignmentNode]) -> Result<(), CompilerError> {
    // TODO
    Ok(())
}


fn resolve_function_calls(ir_context: &mut IRContext) {
    for function in ir_context.get_functions() {
        for node in function.get_statements() {
            if let IRFuncCall::Unresolved(func) = node.get_function() {
                let mut path = function.path().to_owned();
                path.push(func.clone());

                let called = ir_context
                    .get_functions()
                    .iter()
                    .map(|e| (e, match_path_accessibility(&path, e.path())))
                    .next();
            }
        }
    }
}


fn match_path_accessibility(path1: &[String], path2: &[String]) -> u32 {
    let mut sim = 0;

    let count = min(path1.len(), path2.len());
    for i in 0..count {
        if path1[i].eq(&path2[i]) {
            sim += 1;
        } else {
            break;
        }
    }

    sim
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::grammar::parse;
    use crate::runtime::registry::FuncMeta;
    use crate::runtime::Data;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;


    #[test]
    fn load_functions_and_structs() {
        fn external_function(_: &[Arc<Data>]) -> Data {
            // Implementation is not important.
            unimplemented!();
        }

        let mut function_registry = FunctionRegistry::new();
        function_registry
            .register(
                FuncMeta::new(
                    String::from("Add"),
                    external_function,
                    vec![IRDataType::Int, IRDataType::Int],
                    IRDataType::Int,
                )
                .unwrap(),
            )
            .unwrap();
        function_registry
            .register(
                FuncMeta::new(
                    String::from("Mul"),
                    external_function,
                    vec![IRDataType::Int, IRDataType::Int],
                    IRDataType::Int,
                )
                .unwrap(),
            )
            .unwrap();

        let source = compile_context(
            parse(indoc! {r#"
                Math = export mod {
                    Vector = export mod {
                        Point = export struct {
                            x: Float
                            y: Float
                        }
                    }

                    Add = export function {
                        params = (a: Int, b: Int)
                        return = (value: Int)

                        value = extern Add(a, b)
                    }

                    Multiply = export function {
                        params = (a: Int, b: Int)
                        return = (value: Int)

                        value = extern Mul(a, b)
                    }
                }
            "#})
            .unwrap(),
            &function_registry,
        )
        .unwrap();

        let add_func = IRFunction::new(
            vec![String::from("Math"), String::from("Add")],
            0,
            vec![IRDataType::Int, IRDataType::Int],
            IRDataType::Int,
            vec![IRNode::new(
                IRFuncCall::External(String::from("Add")),
                vec![IRNodeInput::FunctionParam(0), IRNodeInput::FunctionParam(1)],
                IRDataType::Int,
            )],
        );

        let mul_func = IRFunction::new(
            vec![String::from("Math"), String::from("Multiply")],
            0,
            vec![IRDataType::Int, IRDataType::Int],
            IRDataType::Int,
            vec![IRNode::new(
                IRFuncCall::External(String::from("Mul")),
                vec![IRNodeInput::FunctionParam(0), IRNodeInput::FunctionParam(1)],
                IRDataType::Int,
            )],
        );

        let mut point_struct = IRStruct::new(
            vec![String::from("Math"), String::from("Vector"), String::from("Point")],
            0,
        );
        point_struct.add_field(String::from("x"), IRDataType::Float).unwrap();
        point_struct.add_field(String::from("y"), IRDataType::Float).unwrap();

        let mut target = IRContext::new();
        target.add_function(add_func);
        target.add_function(mul_func);
        target.add_struct(point_struct);

        assert_eq!(source, target);
    }
}
