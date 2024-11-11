use chunk::{Chunk, OpCode};

mod chunk;
mod debug;
mod memory;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant as u8, 123);
    chunk.write_chunk(constant, 123);

    chunk.write_chunk(OpCode::OpReturn as u8, 123);

    chunk.disassemble_chunk("test chunk");
    chunk.free_chunk();
}
