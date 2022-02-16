use lox_bytecode::{
    chunk::{Chunk, OpCode},
    value::Value,
    vm::Vm,
};

fn main() -> anyhow::Result<()> {
    let mut vm = Vm::new();

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Number(1.2));

    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);
    chunk.write(OpCode::Negate, 123);
    chunk.write(OpCode::Return, 123);

    vm.interpret(chunk)?;

    Ok(())
}
