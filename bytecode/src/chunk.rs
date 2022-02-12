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
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant => write!(f, "OP_CONSTANT"),
            Self::Return => write!(f, "OP_RETURN"),
        }
    }
}

impl OpCode {
    fn disassemble(&self, chunk: &Chunk, offset: usize) -> usize {
        print!("{offset:04} ");

        if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            let line = chunk.lines[offset];
            print!("{line:4} ");
        }

        match self {
            Self::Constant => {
                let constant = chunk.code[offset + 1];
                print!("{self:-16} {constant:4} ");
                let value = chunk.constants[constant as usize];
                println!("{value}");

                offset + 2
            }
            Self::Return => {
                println!("{self}");

                offset + 1
            }
        }
    }
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<f64>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, constant: f64) -> u8 {
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
