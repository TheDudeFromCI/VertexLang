use crate::context::{Callable, Context, VariableData};
use std::rc::Rc;

pub struct AddOperation;
impl Callable for AddOperation {
    fn exec(&self, _context: &Context, inputs: Vec<VariableData>) -> Vec<VariableData> {
        let a = *inputs[0].as_ref().unwrap().downcast_ref::<i64>().unwrap();
        let b = *inputs[1].as_ref().unwrap().downcast_ref::<i64>().unwrap();
        vec![Some(Rc::new(a + b))]
    }
}

pub struct MulOperation;
impl Callable for MulOperation {
    fn exec(&self, _context: &Context, inputs: Vec<VariableData>) -> Vec<VariableData> {
        let a = *inputs[0].as_ref().unwrap().downcast_ref::<i64>().unwrap();
        let b = *inputs[1].as_ref().unwrap().downcast_ref::<i64>().unwrap();
        vec![Some(Rc::new(a * b))]
    }
}
