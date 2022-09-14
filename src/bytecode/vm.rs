use crate::bytecode::error::{InvalidBytecodeError, VtRuntimeError};
use crate::bytecode::ops::{Constant, Op};
use crate::bytecode::Bytecode;
use ordered_float::OrderedFloat;

#[derive(Debug)]
pub struct VM<'a> {
    bytecode: &'a Bytecode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Bool(bool),
    FuncPtr(usize),
}

impl<'a> VM<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        VM { bytecode }
    }

    pub fn exec(&self, pos: usize) -> Result<Node, Box<dyn VtRuntimeError>> {
        let mut stack: Vec<Node> = Vec::with_capacity(512);
        let mut ip = pos;

        loop {
            if ip >= self.bytecode.instructions.len() {
                return Err(Box::new(InvalidBytecodeError(
                    "EOF reached before function end".to_string(),
                )));
            }

            let mut ip_moved = false;

            let r: Result<(), Box<dyn VtRuntimeError>> = match self.bytecode.instructions[ip] {
                Op::NoOp => Ok(()),

                Op::Constant(index) => {
                    let con = match &self.bytecode.constants[index] {
                        Constant::Int(v) => Node::Int(*v),
                        Constant::Float(v) => Node::Float(*v),
                        Constant::String(v) => Node::String(v.to_string()),
                        Constant::Bool(v) => Node::Bool(*v),
                    };

                    stack.push(con);
                    Ok(())
                }

                Op::Return => match stack.pop() {
                    Some(return_value) => {
                        loop {
                            match stack.last() {
                                Some(Node::FuncPtr(ptr)) => {
                                    ip = *ptr;
                                    ip_moved = true;
                                    stack.pop();
                                    stack.push(return_value);
                                    break;
                                }
                                Some(_) => {
                                    stack.pop();
                                }
                                None => {
                                    return Ok(return_value);
                                }
                            }
                        }

                        Ok(())
                    }
                    None => Err(Box::new(InvalidBytecodeError(format!(
                        "Stack underflow at instruction {}",
                        ip
                    )))),
                },

                Op::IntAdd => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs + rhs));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntAdd at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::IntSub => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs - rhs));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntSub at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::IntMul => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs * rhs));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntMul at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::IntDiv => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs / rhs));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntDiv at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::IntMod => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs % rhs));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntMod at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::IntPow => {
                    let a = pop(&mut stack, ip)?;
                    let b = pop(&mut stack, ip)?;
                    match (a, b) {
                        (Node::Int(lhs), Node::Int(rhs)) => {
                            stack.push(Node::Int(lhs.pow(rhs as u32)));
                            Ok(())
                        }
                        pair => Err(Box::new(InvalidBytecodeError(format!(
                            "Unexpected data type {:?} for Op::IntPow at instruction {}",
                            pair, ip
                        )))),
                    }
                }

                Op::Jump(pos) => {
                    if pos >= self.bytecode.instructions.len() {
                        Err(Box::new(InvalidBytecodeError(format!(
                        	"Tried to jump to instruction {} out of max range: {}, at instruction {}",
                        	pos, self.bytecode.instructions.len(), ip
                        ))))
                    } else {
                        stack.push(Node::FuncPtr(ip as usize + 1));
                        ip = pos;
                        ip_moved = true;

                        Ok(())
                    }
                }

                Op::Copy(offset) => {
                    let copy_index: i64 = (stack.len() as i64 - 1) - (offset as i64);
                    if copy_index < 0 {
                        Err(Box::new(InvalidBytecodeError(format!(
                        	"Tried to copy value from outside of stack. Stack size: {}, copy index: {}, at instruction {}",
                        	stack.len(), copy_index, ip
                        ))))
                    } else {
                        stack.push(stack[copy_index as usize].clone());
                        Ok(())
                    }
                }
            };

            match r {
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            if !ip_moved {
                ip += 1;
            }
        }
    }
}

fn pop(stack: &mut Vec<Node>, ip: usize) -> Result<Node, Box<dyn VtRuntimeError>> {
    stack.pop().ok_or_else(|| {
        let err = InvalidBytecodeError(format!("Stack underflow at instruction {}", ip));
        let b: Box<dyn VtRuntimeError> = Box::new(err);
        b
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_simple_add() {
        let bytes = vec![
            0x56, 0x25, 0x14, 0xAF, // Magic number
            0x00, 0x00, 0x00, 0x02, // Const Count
            0x00, 0x00, 0x00, 0x04, // Op Count
            // Constants
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x17, // Int(23)
            0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xED, // Int(-19)
            // Ops
            0x01, 0x00, 0x00, 0x00, 0x00, // Load const 0
            0x01, 0x00, 0x00, 0x00, 0x01, // Load const 1
            0x03, // Add
            0x02, // Return
        ];

        let bytecode = Bytecode::from_bytes(&bytes).unwrap();
        let vm = VM::new(&bytecode);
        let answer = vm.exec(0).unwrap();

        assert_eq!(answer, Node::Int(4));
    }

    #[test]
    fn func_call() {
        let bytes = vec![
            0x56, 0x25, 0x14, 0xAF, // Magic number
            0x00, 0x00, 0x00, 0x03, // Const Count
            0x00, 0x00, 0x00, 0x0E, // Op Count
            // Constants
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Int(1)
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // Int(2)
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, // Int(3)
            // Ops
            // Add Function
            0x0A, 0x00, 0x00, 0x00, 0x02, // Copy int from 2 units back as arg
            0x0A, 0x00, 0x00, 0x00, 0x02, // Copy int from 2 units back as arg
            0x03, // Add
            0x02, // Return
            // Mul Function
            0x0A, 0x00, 0x00, 0x00, 0x02, // Copy int from 2 units back as arg
            0x0A, 0x00, 0x00, 0x00, 0x02, // Copy int from 2 units back as arg
            0x05, // Mul
            0x02, // Return
            // Main Function
            0x01, 0x00, 0x00, 0x00, 0x00, // Load const 0
            0x01, 0x00, 0x00, 0x00, 0x01, // Load const 1
            0x09, 0x00, 0x00, 0x00, 0x04, // Jump to instruction 4 (Mul Func)
            0x01, 0x00, 0x00, 0x00, 0x02, // Load const 2
            0x09, 0x00, 0x00, 0x00, 0x00, // Jump to instruction 0 (Add Func)
            0x02, // Return
        ];

        let bytecode = Bytecode::from_bytes(&bytes).unwrap();
        let vm = VM::new(&bytecode);
        let answer = vm.exec(8).unwrap();

        assert_eq!(answer, Node::Int(5));
    }
}
