use crate::bytecode::error::*;
use crate::bytecode::util::*;
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    NoOp,
    Constant(usize),
    Return,
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntMod,
    IntPow,
    Jump(usize),
    Copy(u32),
}

impl Op {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Op::NoOp => vec![0x00],
            Op::Constant(b) => cascade! { write_u32(*b as u32);..insert(0, 0x01); },
            Op::Return => vec![0x02],
            Op::IntAdd => vec![0x03],
            Op::IntSub => vec![0x04],
            Op::IntMul => vec![0x05],
            Op::IntDiv => vec![0x06],
            Op::IntMod => vec![0x07],
            Op::IntPow => vec![0x08],
            Op::Jump(b) => cascade! { write_u32(*b as u32);..insert(0, 0x09); },
            Op::Copy(b) => cascade! { write_u32(*b);..insert(0, 0x0A); },
        }
    }

    pub fn from_bytes(bytes: &[u8], index: usize) -> Result<(Op, usize), Box<dyn VtRuntimeError>> {
        match &bytes[index] {
            0x00 => Ok((Op::NoOp, 1)),
            0x01 => Ok((
                Op::Constant(match read_u32(bytes, index + 1) {
                    Ok(n) => n as usize,
                    Err(e) => return Err(Box::new(e)),
                }),
                5,
            )),
            0x02 => Ok((Op::Return, 1)),

            0x03 => Ok((Op::IntAdd, 1)),
            0x04 => Ok((Op::IntSub, 1)),
            0x05 => Ok((Op::IntMul, 1)),
            0x06 => Ok((Op::IntDiv, 1)),
            0x07 => Ok((Op::IntMod, 1)),
            0x08 => Ok((Op::IntPow, 1)),

            0x09 => Ok((
                Op::Jump(match read_u32(bytes, index + 1) {
                    Ok(n) => n as usize,
                    Err(e) => return Err(Box::new(e)),
                }),
                5,
            )),
            0x0A => Ok((
                Op::Copy(match read_u32(bytes, index + 1) {
                    Ok(n) => n,
                    Err(e) => return Err(Box::new(e)),
                }),
                5,
            )),

            b => Err(Box::new(UnknownOpError(*b))),
        }
    }
}

impl Constant {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Constant::Int(v) => cascade! { write_i64(*v);..insert(0, 0x01); },
            Constant::Float(v) => cascade! { write_f64(**v);..insert(0, 0x02); },
            Constant::String(v) => cascade! { write_str(v);..insert(0, 0x03); },
            Constant::Bool(v) => cascade! { write_bool(*v);..insert(0, 0x04); },
        }
    }

    pub fn from_bytes(
        bytes: &[u8],
        index: usize,
    ) -> Result<(Constant, usize), Box<dyn VtRuntimeError>> {
        match &bytes[index] {
            // Integer
            0x01 => Ok((
                Constant::Int(match read_i64(bytes, index + 1) {
                    Ok(n) => n,
                    Err(e) => return Err(Box::new(e)),
                }),
                9,
            )),

            // Float
            0x02 => Ok((
                Constant::Float(match read_f64(bytes, index + 1) {
                    Ok(n) => OrderedFloat(n),
                    Err(e) => return Err(Box::new(e)),
                }),
                9,
            )),

            // String
            0x03 => {
                let s = match read_str(bytes, index + 1) {
                    Ok(n) => n,
                    Err(e) => return Err(Box::new(e)),
                };

                let byte_count = s.len() + 5;
                Ok((Constant::String(s), byte_count))
            }

            // Bool
            0x04 => Ok((
                Constant::Bool(match read_bool(bytes, index + 1) {
                    Ok(n) => n,
                    Err(e) => return Err(Box::new(e)),
                }),
                2,
            )),

            // Other
            b => Err(Box::new(UnknownConstError(*b))),
        }
    }
}
