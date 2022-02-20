use crate::value::Value;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    InvalidOpCode(#[from] num_enum::TryFromPrimitiveError<OpCode>),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant => write!(f, "OP_CONSTANT"),
            Self::Nil => write!(f, "OP_NIL"),
            Self::True => write!(f, "OP_TRUE"),
            Self::False => write!(f, "OP_FALSE"),
            Self::Equal => write!(f, "OP_EQUAL"),
            Self::Greater => write!(f, "OP_GREATER"),
            Self::Less => write!(f, "OP_LESS"),
            Self::Add => write!(f, "OP_ADD"),
            Self::Subtract => write!(f, "OP_SUBTRACT"),
            Self::Multiply => write!(f, "OP_MULTIPLY"),
            Self::Divide => write!(f, "OP_DIVIDE"),
            Self::Not => write!(f, "OP_NOT"),
            Self::Negate => write!(f, "OP_NEGATE"),
            Self::Return => write!(f, "OP_RETURN"),
        }
    }
}

impl OpCode {
    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> usize {
        print!("{offset:04} ");

        if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            let line = chunk.lines[offset];
            print!("{line:4} ");
        }

        fn simple_intruction(op: &OpCode, offset: usize) -> usize {
            println!("{op}");

            offset + 1
        }

        match self {
            Self::Constant => {
                let constant = chunk.code[offset + 1];
                print!("{self:-16} {constant:4} ");
                let value = &chunk.constants[constant as usize];
                println!("{value}");

                offset + 2
            }
            Self::Nil => simple_intruction(self, offset),
            Self::True => simple_intruction(self, offset),
            Self::False => simple_intruction(self, offset),
            Self::Equal => simple_intruction(self, offset),
            Self::Greater => simple_intruction(self, offset),
            Self::Less => simple_intruction(self, offset),
            Self::Add => simple_intruction(self, offset),
            Self::Subtract => simple_intruction(self, offset),
            Self::Multiply => simple_intruction(self, offset),
            Self::Divide => simple_intruction(self, offset),
            Self::Not => simple_intruction(self, offset),
            Self::Negate => simple_intruction(self, offset),
            Self::Return => simple_intruction(self, offset),
        }
    }
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    pub fn lines(&self) -> &[usize] {
        &self.lines
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);

        (self.constants.len() - 1) as u8
    }

    pub fn disassemble(&self, name: &str) -> Result<()> {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            let instruction = self.code[offset];
            let op = OpCode::try_from(instruction)?;
            offset = op.disassemble(self, offset);
        }

        Ok(())
    }
}
