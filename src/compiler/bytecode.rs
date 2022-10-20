use super::ir::{IRContext, IRFuncCall, IRNodeInput};
use crate::runtime::registry::FunctionRegistry;
use crate::runtime::{
    Data, ExternalFunction, FunctionCall, InternalFunction, Operation, OperationInput, VertexBytecode
};


/// Creates a new VertexBytecode instance based on the given IRContext.
///
/// This method will panic if the intermediate representation is not properly
/// loaded or generated.
pub fn bytecode_from_ir(context: IRContext, registry: &FunctionRegistry) -> VertexBytecode {
    let mut bytecode = VertexBytecode::new();

    for function in context.get_functions() {
        let mut int_func = InternalFunction::new();
        for statement in function.get_statements() {
            let func = match statement.get_function() {
                IRFuncCall::External(f) => add_ext_func(&mut bytecode, f, registry),
                IRFuncCall::Internal(f) => FunctionCall::Internal(*f),
                IRFuncCall::IntConstant(v) => add_const(&mut bytecode, Data::Int(*v)),
                IRFuncCall::FloatConstant(v) => add_const(&mut bytecode, Data::Float(*v)),
                IRFuncCall::StringConstant(v) => add_const(&mut bytecode, Data::String(v.clone())),
                IRFuncCall::CharConstant(v) => add_const(&mut bytecode, Data::Char(*v)),
                IRFuncCall::BoolConstant(v) => add_const(&mut bytecode, Data::Bool(*v)),
                IRFuncCall::Unresolved(_) => {
                    panic!("Cannot load bytecode from unresolved functions!")
                },
            };

            let mut inputs = vec![];
            for input in statement.get_inputs() {
                let input = match input {
                    IRNodeInput::FunctionParam(i) => OperationInput::Param(*i as usize),
                    IRNodeInput::HiddenNode(i) => OperationInput::Hidden(*i as usize),
                };
                inputs.push(input);
            }

            let operation = Operation::new(func, inputs);
            int_func.add_operation(operation);
        }

        bytecode.add_internal_function(int_func);
    }

    bytecode
}


fn add_const(bytecode: &mut VertexBytecode, constant: Data) -> FunctionCall {
    if let Some(index) = bytecode.get_constants().iter().position(|c| **c == constant) {
        FunctionCall::Constant(index)
    } else {
        bytecode.add_constant(constant);
        FunctionCall::Constant(bytecode.get_constants().len() - 1)
    }
}


fn add_ext_func(
    bytecode: &mut VertexBytecode, function: &str, registry: &FunctionRegistry,
) -> FunctionCall {
    if let Some(index) = bytecode
        .get_external_functions()
        .iter()
        .position(|f| f.get_function_name().eq(function))
    {
        FunctionCall::External(index)
    } else if let Some(func_meta) = registry.get_function(function) {
        let func = func_meta.get_func();
        bytecode.add_external_function(ExternalFunction::new(function.to_owned(), func));
        FunctionCall::External(bytecode.get_external_functions().len() - 1)
    } else {
        panic!("Unknown function: {}", function);
    }
}
