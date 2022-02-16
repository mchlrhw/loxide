use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Compile error.")]
    Compile,
    #[error("Runtime error.")]
    Runtime,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Vm {
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    fn incr_ip(&mut self) -> usize {
        let before = self.ip;
        self.ip += 1;

        before
    }

    fn read_byte(&mut self, chunk: &Chunk) -> u8 {
        let ip = self.incr_ip();

        chunk.code()[ip]
    }

    fn read_constant<'c>(&mut self, chunk: &'c Chunk) -> &'c Value {
        let idx = self.read_byte(chunk) as usize;

        &chunk.constants()[idx]
    }

    fn run(&mut self, chunk: Chunk) -> Result<()> {
        loop {
            #[cfg(feature = "trace_execution")]
            let offset = self.ip;

            let instruction = self.read_byte(&chunk);
            let op = OpCode::try_from(instruction).map_err(|_| Error::Runtime)?;

            #[cfg(feature = "trace_execution")]
            op.disassemble(&chunk, offset);

            match op {
                OpCode::Return => return Ok(()),
                OpCode::Constant => {
                    let constant = self.read_constant(&chunk);
                    println!("{constant}");
                }
            }
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<()> {
        self.ip = 0;

        self.run(chunk)
    }
}
