use crate::bytecode::Bytecode;
use crate::bytecode::error::{VtRuntimeError, InvalidBytecodeError};
use crate::bytecode::ops::{Op, Constant};
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
				return Err(Box::new(InvalidBytecodeError("EOF reached before function end".to_string())));
			}

			let r: Result<(), Box<dyn VtRuntimeError>> = match self.bytecode.instructions[ip] {
				Op::NoOp =>  Ok(()),

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

				Op::PopOp => {
					match stack.pop() {
						Some(n) => {
							if stack.is_empty() {
								return Ok(n);
							}
						
							Ok(())
						}
						None => Err(Box::new(InvalidBytecodeError("Stack underflow".to_string())))
					}
				}

				Op::IntAdd => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs + rhs)); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}

				Op::IntSub => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs - rhs)); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}

				Op::IntMul => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs * rhs)); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}

				Op::IntDiv => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs / rhs)); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}

				Op::IntMod => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs % rhs)); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}

				Op::IntPow => {
					let a = pop(&mut stack)?;
					let b = pop(&mut stack)?;
					match (a, b) {
						(Node::Int(lhs), Node::Int(rhs)) => { stack.push(Node::Int(lhs.pow(rhs as u32))); Ok(()) },
						_ => Err(Box::new(InvalidBytecodeError("Unexpected data type".to_string()))),
					}
				}
			};

			match r {
				Ok(_) => (),
				Err(e) => return Err(e)
			};
		
			ip += 1;
		}
	}
}

fn pop(stack: &mut Vec<Node>) -> Result<Node, Box<dyn VtRuntimeError>> {
	stack.pop().ok_or_else(|| {
		let err = InvalidBytecodeError("Stack underflow".to_string());
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
			// Magic number
			0x56, 0x25, 0x14, 0xAF,

			// Const Count
			0x00, 0x00, 0x00, 0x02,

			// Op Count
			0x00, 0x00, 0x00, 0x04,

			// Constants
			0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x17, // Int(23)
			0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xED, // Int(-19)

			// Ops
			0x01, 0x00, 0x00, 0x00, 0x00, // Load const 0
			0x01, 0x00, 0x00, 0x00, 0x01, // Load const 1
			0x03, // Add
			0x02, // Pop
		];

		let bytecode = Bytecode::from_bytes(&bytes).unwrap();
		let vm = VM::new(&bytecode);
		let answer = vm.exec(0).unwrap();
		
		assert_eq!(answer, Node::Int(4));
	}
}
