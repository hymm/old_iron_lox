use std::{ptr::null_mut, sync::LazyLock};

use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

struct Vm {
    chunk: *mut Chunk,
    instruction_pointer: *mut u8,
}

// TODO: not really send and sync, but we do this to make it a global static.
// if we actually access this from multiple threads it will be UB.
unsafe impl Sync for Vm {}
unsafe impl Send for Vm {}

// TODO: this is very unsafe and probably will have UB. But we do it to mirror the book and will refactor
// later
static mut VM: Vm = Vm {
    chunk: null_mut(),
    instruction_pointer: null_mut(),
};

// These methods might be a little too "C" and should be converted to a more rust styld.
pub fn init_vm() {}

pub fn free_vm() {}

// should consider making this lifetimed
pub fn interpret(chunk: *mut Chunk) -> Result<(), InterpretError> {
    unsafe { VM.chunk = chunk };
    unsafe { VM.instruction_pointer = (*chunk).code };

    run()
}

fn run() -> Result<(), InterpretError> {
    // #define READ_BTYE() (*vm.instruction_pointer++);
    fn read_byte() -> u8 {
        let byte = unsafe { *VM.instruction_pointer };
        unsafe {
            VM.instruction_pointer = VM.instruction_pointer.add(1);
        }
        byte
    }

    fn read_constant() -> Value {
        let chunk = unsafe { &mut *VM.chunk };
        unsafe { *chunk.constants.values.add(read_byte() as usize) }
    }

    loop {
        #[cfg(feature = "debug_trace_execution")]
        {
            let chunk = unsafe { &mut *VM.chunk };
            let diff = unsafe { VM.instruction_pointer.offset_from(chunk.code) };
            unsafe {
                chunk.disassemble_instruction(diff);
            }
        }
        let instruction: OpCode = read_byte().into();
        match instruction {
            OpCode::OpConstant => {
                let constant = read_constant();
                println!("{constant}");
            }
            OpCode::OpReturn => return Ok(()),
        }
    }
}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}
