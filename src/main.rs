use chunk::{Chunk, OpCode};
use vm::{free_vm, init_vm, interpret};

mod chunk;
mod debug;
mod memory;
mod value;
mod vm;

fn main() {
    init_vm();

    let mut chunk = Chunk::new();
    let chunk = &mut chunk;

    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant as u8, 123);
    chunk.write_chunk(constant, 123);

    chunk.write_chunk(OpCode::OpReturn as u8, 123);

    interpret(chunk as *mut Chunk).ok();

    free_vm();
    chunk.free_chunk();
}
