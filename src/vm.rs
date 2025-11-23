use std::{fmt::Display, ptr::null_mut};

use crate::{
    chunk::{Chunk, OpCode},
    compiler::compile,
    value::{Value, is_bool, is_nil, values_equal},
};

const STACK_MAX: usize = 256;

pub struct Vm {
    pub(crate) chunk: *mut Chunk,
    /// pointer to current instruction
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
pub static mut VM: Vm = Vm {
    chunk: null_mut(),
    instruction_pointer: null_mut(),
    stack: [Value::Double(0.0); STACK_MAX],
    stack_top: null_mut(),
};

fn reset_stack() {
    unsafe {
        VM.stack_top = &mut VM.stack[0] as *mut Value;
    }
}

fn runtime_error(err: impl Display) {
    // get the line number
    let instruction_index = unsafe {
        VM.instruction_pointer
            .byte_sub((*VM.chunk).code as usize)
            // we want the previous instruction, since the pointer was already advanced
            .byte_sub(1)
    };
    let line = unsafe { (*VM.chunk).lines.add(*instruction_index as usize) };
    print!("{err} {} in script\n", unsafe { *line });
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

fn peek(distance: isize) -> Value {
    unsafe { *VM.stack_top.offset(-1 - distance) }
}

fn is_falsey(value: Value) -> bool {
    is_nil(value) || TryFrom::try_from(value).unwrap_or(false)
}

// These methods might be a little too "C" and should be converted to a more rust styld.
pub fn init_vm() {
    reset_stack();
}

pub fn free_vm() {}

// should consider making this lifetimed
pub fn interpret(source: &str) -> Result<(), InterpretError> {
    let mut chunk = Chunk::new();

    if !compile(source, &mut chunk) {
        chunk.free_chunk();
    }

    unsafe {
        VM.chunk = &mut chunk as *mut Chunk;
        VM.instruction_pointer = (*VM.chunk).code;
    }

    let result = run();

    chunk.free_chunk();
    result
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
        ($variant:expr, $op:tt) => {
            {
                let (Ok(a), Ok(b)) = (TryInto::<f64>::try_into(peek(0)), TryInto::<f64>::try_into(peek(1))) else {
                    runtime_error("Operands must be a numbers.");
                    return Err(InterpretError::RuntimeError);
                };
                pop();
                pop();
                push($variant(a $op b));
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
            OpCode::Nil => push(Value::Nil),
            OpCode::True => push(Value::Bool(true)),
            OpCode::False => push(Value::Bool(false)),
            OpCode::Equal => {
                let b = pop();
                let a = pop();
                push(Value::Bool(values_equal(a, b)));
            }
            OpCode::Negate => {
                let Ok(value) = TryInto::<f64>::try_into(peek(0)) else {
                    runtime_error("Operand must be a number.");
                    return Err(InterpretError::RuntimeError);
                };
                pop();
                push(Value::Double(-value));
            }
            OpCode::Greater => binary_op!(Value::Bool, >),
            OpCode::Less => binary_op!(Value::Bool, <),
            OpCode::Add => binary_op!(Value::Double, +),
            OpCode::Subtract => binary_op!(Value::Double, -),
            OpCode::Multiply => binary_op!(Value::Double, *),
            OpCode::Divide => binary_op!(Value::Double, /),
            OpCode::Not => push(Value::Bool(is_falsey(pop()))),
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
