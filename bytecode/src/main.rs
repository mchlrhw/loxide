use lox_bytecode::chunk::{Chunk, OpCode};

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write(OpCode::Return, 123);

    chunk.disassemble("test chunk")?;

    Ok(())
}
