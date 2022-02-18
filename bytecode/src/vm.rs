use crate::{
    chunk::{Chunk, OpCode},
    compiler::compile,
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
    stack: Vec<Value>,
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
            {
                print!("          ");
                for value in &self.stack {
                    print!("[{value}]");
                }
                println!();
                op.disassemble(&chunk, offset);
            }

            macro_rules! binary_op {
                ($op:tt) => {
                    let b = self.stack.pop().expect("stack mut have values");
                    let a = self.stack.pop().expect("stack mut have values");
                    self.stack.push(a $op b);
                }
            }

            match op {
                OpCode::Constant => {
                    let constant = self.read_constant(&chunk);
                    self.stack.push(constant.clone());
                }
                OpCode::Add => {
                    binary_op!(+);
                }
                OpCode::Subtract => {
                    binary_op!(-);
                }
                OpCode::Multiply => {
                    binary_op!(*);
                }
                OpCode::Divide => {
                    binary_op!(/);
                }
                OpCode::Negate => {
                    let value = self.stack.pop().expect("stack must have values");
                    self.stack.push(-value);
                }
                OpCode::Return => {
                    if let Some(value) = self.stack.pop() {
                        println!("{value}");
                    }

                    return Ok(());
                }
            }
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<()> {
        self.ip = 0;

        self.run(chunk)
    }
}

pub fn interpret(source: &str) -> Result<()> {
    compile(source);

    Ok(())
}
