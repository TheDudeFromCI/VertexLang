use crate::bytecode::error::*;
use crate::bytecode::util::*;
use crate::bytecode::ops::*;

pub const MAGIC_NUMBER: u32 = 0x562514AF_u32;

#[derive(Debug)]
pub struct Bytecode {
	instructions: Vec<Op>,
	constants: Vec<Constant>,
}

impl PartialEq for Bytecode {
	fn eq(&self, other: &Self) -> bool {
		if self.instructions.len() != other.instructions.len()
			|| self.constants.len() != other.constants.len() {
				return false;
			}
	
		for (index, val) in self.instructions.iter().enumerate() {
			if *val != other.instructions[index] {
				return false;
			}
		}

		for (index, val) in self.constants.iter().enumerate() {
			if *val != other.constants[index] {
				return false;
			}
		}

		true
	}	
}

#[derive(Debug)]
struct BytecodeHeader {
	const_count: usize,
	op_count: usize,
}

impl Bytecode {
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn VtRuntimeError>> {
		let header = match Self::read_header(bytes) {
			Ok(v) => v,
			Err(e) => return Err(Box::new(e)),			
		};

		let mut cons: Vec<Constant> = Vec::with_capacity(header.const_count);
		let mut ops: Vec<Op> = Vec::with_capacity(header.op_count);
		let mut index: usize = 12;

		// Load constants
		for _ in 0..header.const_count {
			let (constant, byte_count) = match Constant::from_bytes(bytes, index) {
				Ok(v) => v,
				Err(e) => return Err(e),
			};
			cons.push(constant);
			index += byte_count;
		}

		// Load ops
		for _ in 0..header.op_count {
			let (op, len) = match Op::from_bytes(bytes, index) {
				Ok(v) => v,
				Err(e) => return Err(e),
			};
			ops.push(op);
			index += len;
		}

		Ok(Bytecode { instructions: ops, constants: cons })
	}

	fn read_header(bytes: &[u8]) -> Result<BytecodeHeader, InvalidBytecodeError> {
		if read_u32(bytes, 0)? != MAGIC_NUMBER {
			return Err(InvalidBytecodeError(String::from("Invalid bytecode header")));
		}

		let const_count = read_u32(bytes, 4)? as usize;
		let op_count = read_u32(bytes, 8)? as usize;
	
		Ok(BytecodeHeader {
			const_count,
			op_count,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_simple_add() {
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
		assert_eq!(bytecode, Bytecode {
			instructions: vec![
				Op::Constant(0),
				Op::Constant(1),
				Op::IntAdd,
				Op::PopOp,
			],
			constants: vec![
				Constant::Int(23),
				Constant::Int(-19),
			],
		});
	}
}
