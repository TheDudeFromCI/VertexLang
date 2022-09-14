use crate::bytecode::error::InvalidBytecodeError;

pub fn read_u32(bytes: &[u8], index: usize) -> Result<u32, InvalidBytecodeError> {
	if index + 3 >= bytes.len() {
		return Err(InvalidBytecodeError(String::from("Unexpected EOF while parsing u32")));
	}
	
	match bytes[index..index+4].try_into() {
		Ok(b) => Ok(u32::from_be_bytes(b)),
		Err(_) => Err(InvalidBytecodeError(String::from("Cannot parse u32 bytes"))),
	}
}

pub fn read_i64(bytes: &[u8], index: usize) -> Result<i64, InvalidBytecodeError> {
	if index + 7 >= bytes.len() {
		return Err(InvalidBytecodeError(String::from("Unexpected EOF while parsing i64")));
	}
	
	match bytes[index..index+8].try_into() {
		Ok(b) => Ok(i64::from_be_bytes(b)),
		Err(_) => Err(InvalidBytecodeError(String::from("Cannot parse i64 bytes"))),
	}
}

pub fn read_f64(bytes: &[u8], index: usize) -> Result<f64, InvalidBytecodeError> {
	if index + 7 >= bytes.len() {
		return Err(InvalidBytecodeError(String::from("Unexpected EOF while parsing f64")));
	}
	
	match bytes[index..index+8].try_into() {
		Ok(b) => Ok(f64::from_be_bytes(b)),
		Err(_) => Err(InvalidBytecodeError(String::from("Cannot parse f64 bytes"))),
	}
}

pub fn read_str(bytes: &[u8], index: usize) -> Result<String, InvalidBytecodeError> {
	let length = read_u32(bytes, index)? as usize;
	if index + 3 + length >= bytes.len() {
		return Err(InvalidBytecodeError(String::from("Unexpected EOF while parsing str")));
	}
	
	let s = String::from_utf8(bytes[index+4..index+4+length].to_vec());
	match s {
		Ok(v) => Ok(v),
		Err(_) => Err(InvalidBytecodeError("Cannot parse string literal".to_string())),
	}
}

pub fn read_bool(bytes: &[u8], index: usize) -> Result<bool, InvalidBytecodeError> {
	if index >= bytes.len() {
		return Err(InvalidBytecodeError(String::from("Unexpected EOF while parsing bool")));
	}
	
	match bytes[index] {
		0x00 => Ok(false),
		_ => Ok(true),
	}
}

pub fn write_u32(n: u32) -> Vec<u8> {
	u32::to_be_bytes(n).to_vec()
}

pub fn write_i64(n: i64) -> Vec<u8> {
	i64::to_be_bytes(n).to_vec()
}

pub fn write_f64(n: f64) -> Vec<u8> {
	f64::to_be_bytes(n).to_vec()
}

pub fn write_str(s: &str) -> Vec<u8> {
	let mut v = s.as_bytes().to_vec();
	v.splice(0..0, write_u32(s.len() as u32));
	v
}

pub fn write_bool(b: bool) -> Vec<u8> {
	if b {
		vec![0x01]
	} else {
		vec![0x00]
	}
}
