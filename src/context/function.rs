use super::datatype::DataType;
use std::any::Any;
use std::rc::Rc;

pub struct Variable {
    dtype: DataType,
    index: usize,
}

pub type VariableData = Option<Rc<dyn Any>>;

pub struct Procedure {
    func_index: usize,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

pub struct Function {
    variables: Vec<Variable>,
    input_vars: Vec<usize>,
    output_vars: Vec<usize>,
    procedures: Vec<Procedure>,
}

pub trait Callable {
    fn exec(&self, context: &Context, inputs: Vec<VariableData>) -> Vec<VariableData>;
}

pub struct Context {
    functions: Vec<Box<dyn Callable>>,
}

impl Function {
    pub fn new() -> Self {
        Function {
            variables: vec![],
            input_vars: vec![],
            output_vars: vec![],
            procedures: vec![],
        }
    }

    pub fn add_variable(&mut self, dtype: DataType) -> usize {
        let index = self.variables.len();
        let variable = Variable { dtype, index };
        self.variables.push(variable);
        index
    }

    pub fn add_input(&mut self, dtype: DataType) -> usize {
        let index = self.add_variable(dtype);
        self.input_vars.push(index);
        index
    }

    pub fn add_output(&mut self, dtype: DataType) -> usize {
        let index = self.add_variable(dtype);
        self.output_vars.push(index);
        index
    }

    pub fn add_procedure(&mut self, func_index: usize, inputs: Vec<usize>, outputs: Vec<usize>) {
        let proc = Procedure {
            func_index,
            inputs,
            outputs,
        };
        self.procedures.push(proc);
    }
}

impl Context {
    pub fn new() -> Self {
        Context { functions: vec![] }
    }

    pub fn add_function(&mut self, callable: Box<dyn Callable>) -> usize {
        let index = self.functions.len();
        self.functions.push(callable);
        index
    }

    pub fn exec(&self, func_index: usize, inputs: Vec<VariableData>) -> Vec<VariableData> {
        (*self.functions[func_index]).exec(self, inputs)
    }
}

impl Callable for Function { 
    fn exec(&self, context: &Context, inputs: Vec<VariableData>) -> Vec<VariableData> {
        let mut vars: Vec<VariableData> = vec![None; self.variables.len()];
        let mut outputs: Vec<VariableData> = vec![None; self.output_vars.len()];

        for (index, var) in inputs.iter().enumerate() {
            match &var {
                Some(v) => vars[self.input_vars[index]] = Some(Rc::clone(v)),
                None => vars[self.input_vars[index]] = None,
            };
        }

        for proc in &self.procedures {
            let callable = &context.functions[proc.func_index];

            let mut proc_in: Vec<VariableData> = vec![None; proc.inputs.len()];
            for (index, var) in proc.inputs.iter().enumerate() {
                match &vars[*var] {
                    Some(v) => proc_in[index] = Some(Rc::clone(v)),
                    None => proc_in[index] = None,
                }
            }

            let proc_out = callable.exec(context, proc_in);
            for (index, var) in proc_out.iter().enumerate() {
                match var {
                    Some(v) => vars[proc.outputs[index]] = Some(Rc::clone(v)),
                    None => vars[proc.outputs[index]] = None,
                }
            }
        }

        for (index, var_index) in self.output_vars.iter().enumerate() {
            match &vars[*var_index] {
                Some(v) => outputs[index] = Some(Rc::clone(v)),
                None => outputs[index] = None,
            }
        }

        outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
	use crate::runtime::math::{AddOperation, MulOperation};

    #[test]
    fn simple_madd_func() {
        let mut context = Context::new();

        let add_func_index = context.add_function(Box::new(AddOperation {}));
        let mul_func_index = context.add_function(Box::new(MulOperation {}));

        let mut madd_function = Function::new();
        let a_input = madd_function.add_input(DataType::Int);
        let b_input = madd_function.add_input(DataType::Int);
        let c_input = madd_function.add_input(DataType::Int);
        let d_var = madd_function.add_variable(DataType::Int);
        let out_var = madd_function.add_output(DataType::Int);
        madd_function.add_procedure(mul_func_index, vec![a_input, b_input], vec![d_var]);
        madd_function.add_procedure(add_func_index, vec![c_input, d_var], vec![out_var]);
        let madd_func_index = context.add_function(Box::new(madd_function));

        let madd_a: VariableData = Some(Rc::new(2_i64));
        let madd_b: VariableData = Some(Rc::new(3_i64));
        let madd_c: VariableData = Some(Rc::new(1_i64));
        let mut madd_output = context.exec(madd_func_index, vec![madd_a, madd_b, madd_c]);
        let madd_output = madd_output.remove(0).unwrap();
        let madd_output = *(*madd_output).downcast_ref::<i64>().unwrap();
        assert_eq!(madd_output, 7)
    }
}
