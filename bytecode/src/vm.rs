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

    fn reset_stack(&mut self) {
        self.stack = vec![];
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

    fn peek(&self, distance: usize) -> Option<&Value> {
        self.stack.get(self.stack.len() - 1 - distance)
    }

    fn runtime_error(&mut self, message: &str, chunk: &Chunk) {
        let line = chunk.lines()[self.ip - 1];
        eprintln!("{message}\n[line {line}] in script");
        self.reset_stack();
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
                    if let (Some(Value::Number(_)), Some(Value::Number(_))) = (self.peek(0), self.peek(1)) {
                        let b = self.stack.pop().expect("stack mut have values");
                        let a = self.stack.pop().expect("stack mut have values");
                        self.stack.push(a $op b);
                    } else {
                        self.runtime_error("Operands must be numbers.", &chunk);
                        return Err(Error::Runtime);
                    }
                }
            }

            macro_rules! cmp_op {
                ($op:tt) => {
                    if let (Some(Value::Number(_)), Some(Value::Number(_))) = (self.peek(0), self.peek(1)) {
                        let b = self.stack.pop().expect("stack mut have values");
                        let a = self.stack.pop().expect("stack mut have values");
                        self.stack.push(Value::Boolean(a $op b));
                    } else {
                        self.runtime_error("Operands must be numbers.", &chunk);
                        return Err(Error::Runtime);
                    }
                }
            }

            match op {
                OpCode::Constant => {
                    let constant = self.read_constant(&chunk);
                    self.stack.push(constant.clone());
                }
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::True => {
                    self.stack.push(Value::Boolean(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Boolean(false));
                }
                OpCode::Equal => {
                    let b = self.stack.pop().expect("stack mut have values");
                    let a = self.stack.pop().expect("stack mut have values");
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Greater => {
                    cmp_op!(>);
                }
                OpCode::Less => {
                    cmp_op!(<);
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
                OpCode::Not => {
                    let value = self.stack.pop().expect("stack must have values");
                    self.stack.push(Value::Boolean(value.is_falsey()));
                }
                OpCode::Negate => {
                    if let Some(Value::Number(_)) = self.peek(0) {
                        let value = self.stack.pop().expect("stack must have values");
                        self.stack.push(-value);
                    } else {
                        self.runtime_error("Operand must be a number.", &chunk);
                        return Err(Error::Runtime);
                    }
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

    pub fn interpret(&mut self, source: &str) -> Result<()> {
        let mut chunk = Chunk::new();

        if !compile(source, &mut chunk) {
            return Err(Error::Compile);
        }

        self.ip = 0;

        self.run(chunk)
    }
}
