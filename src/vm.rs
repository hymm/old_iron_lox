use std::ptr::null_mut;

use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

const STACK_MAX: usize = 256;

struct Vm {
    chunk: *mut Chunk,
    instruction_pointer: *mut u8,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
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
    stack: [0.0; STACK_MAX],
    stack_top: null_mut(),
};

fn reset_stack() {
    unsafe {
        VM.stack_top = &mut VM.stack[0] as *mut Value;
    }
}

fn push(value: Value) {
    unsafe {
        *VM.stack_top = value;
    }
    unsafe {
        VM.stack_top = VM.stack_top.offset(1);
    }
}

fn pop() -> Value {
    unsafe {
        VM.stack_top = VM.stack_top.offset(-1);
    }
    unsafe { *VM.stack_top }
}

// These methods might be a little too "C" and should be converted to a more rust styld.
pub fn init_vm() {
    reset_stack();
}

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

    macro_rules! binary_op {
        ($op:tt) => {
            {
                let b = pop();
                let a = pop();
                push(a $op b);
            }
        };
    }

    loop {
        #[cfg(feature = "debug_trace_execution")]
        {
            print!("          ");
            let mut slot = unsafe { &mut VM.stack[0] as *mut Value };
            while slot != unsafe { VM.stack_top } {
                print!("[ {} ]", unsafe { *slot });
                slot = unsafe { slot.add(1) };
            }
            println!();

            let chunk = unsafe { &mut *VM.chunk };
            let diff = unsafe { VM.instruction_pointer.offset_from(chunk.code) };
            unsafe {
                chunk.disassemble_instruction(diff);
            }
        }
        let instruction: OpCode = read_byte().into();
        match instruction {
            OpCode::Constant => {
                let constant = read_constant();
                push(constant);
            }
            OpCode::Negate => {
                push(-pop());
            }
            OpCode::Add => binary_op!(+),
            OpCode::Subtract => binary_op!(-),
            OpCode::Multiply => binary_op!(*),
            OpCode::Divide => binary_op!(/),
            OpCode::Return => {
                println!("{}", pop());
                return Ok(());
            }
        }
    }
}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}
