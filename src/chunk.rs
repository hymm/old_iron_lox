use std::ptr::null_mut;

use crate::{
    memory::{free_array, grow_array, grow_capacity},
    value::{Value, ValueArray},
};

#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            // TODO: this is getting a bit ugly. Should I look for a library for this?
            0 => OpCode::Constant,
            1 => OpCode::Add,
            2 => OpCode::Subtract,
            3 => OpCode::Multiply,
            4 => OpCode::Divide,
            5 => OpCode::Negate,
            6 => OpCode::Return,
            _ => panic!("unexpected value {value} for OpCode"),
        }
    }
}

// it'd probably be better to use a `Vec` or `Bytes`, but we use some unsafe here
// for learning purposes.
pub struct Chunk {
    pub(crate) count: usize,
    pub(crate) capacity: usize,
    pub(crate) code: *mut u8,
    pub(crate) lines: *mut usize,
    pub(crate) constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            count: 0,
            capacity: 0,
            code: null_mut(),
            lines: null_mut(),
            constants: ValueArray::new(),
        }
    }

    pub fn write_chunk(&mut self, byte: u8, line: usize) {
        if self.capacity < self.count + 1 {
            let old_capacity = self.capacity;
            self.capacity = grow_capacity(old_capacity);
            // Safety:
            // - always allocated from calls to grow_array
            // - layout is always u8
            self.code = unsafe { grow_array::<u8>(self.code, old_capacity, self.capacity) };
            self.lines = unsafe { grow_array::<usize>(self.lines, old_capacity, self.capacity) }
        }

        // Safety:
        // - We checked that we have enough allocation above.
        // - u8 is always aligned
        unsafe { *self.code.add(self.count) = byte };
        unsafe { *self.lines.add(self.count) = line };
        self.count += 1;
    }

    pub fn free_chunk(&mut self) {
        // Safety:
        // - always allocated from calls to grow_array
        // - layout is always u8
        unsafe { free_array::<u8>(self.code, self.capacity) };
        unsafe { free_array::<usize>(self.lines, self.capacity) };
        self.constants.free_value_array();
        *self = Self::new();
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write_value_array(value);
        (self.constants.count - 1) as u8
    }
}
