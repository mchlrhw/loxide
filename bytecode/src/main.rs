use lox_bytecode::{
    chunk::{Chunk, OpCode},
    value::Value,
    vm::Vm,
};

fn main() -> anyhow::Result<()> {
    let mut vm = Vm::new();

    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(Value::Number(1.2));
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);

    constant = chunk.add_constant(Value::Number(3.4));
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write(OpCode::Add, 123);

    constant = chunk.add_constant(Value::Number(5.6));
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write(OpCode::Divide, 123);
    chunk.write(OpCode::Negate, 123);

    chunk.write(OpCode::Return, 123);

    vm.interpret(chunk)?;

    Ok(())
}
