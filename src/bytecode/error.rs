use std::fmt;

pub trait VtRuntimeError: core::fmt::Debug {
	fn get_message(&self) -> String;
}

impl fmt::Display for dyn VtRuntimeError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Vertex runtime error!\n{}", self.get_message())
	}
}

#[derive(Debug, Clone)]
pub struct UnknownOpError(pub u8);

impl VtRuntimeError for UnknownOpError {
	fn get_message(&self) -> String {
		format!("Unknown bytecode operation: 0x{:02x}", self.0)
	}
}

#[derive(Debug, Clone)]
pub struct UnknownConstError(pub u8);

impl VtRuntimeError for UnknownConstError {
	fn get_message(&self) -> String {
		format!("Unknown bytecode const type: 0x{:02x}", self.0)
	}
}

#[derive(Debug, Clone)]
pub struct InvalidBytecodeError(pub String);

impl VtRuntimeError for InvalidBytecodeError {
	fn get_message(&self) -> String {
		self.0.clone()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bytecode_err() {
		let e: Box::<dyn VtRuntimeError> = Box::new(UnknownOpError(0x07));

		let mes: String = format!("{}", e);
		assert_eq!(mes, "Vertex runtime error!\nUnknown bytecode operation: 0x07");
	}
}
