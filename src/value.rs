use std::ptr::null_mut;

use crate::memory::{free_array, grow_array, grow_capacity};

pub type Value = f64;

// This would make more sense as new typed Vec<Value>, but for learning purposes we're going to play
// with allocation
pub struct ValueArray {
    pub capacity: usize,
    pub count: usize,
    pub values: *mut Value,
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            capacity: 0,
            count: 0,
            values: null_mut(),
        }
    }

    pub fn write_value_array(&mut self, value: Value) {
        if self.capacity < self.count + 1 {
            let old_capacity = self.capacity;
            self.capacity = grow_capacity(old_capacity);
            self.values = unsafe { grow_array::<Value>(self.values, old_capacity, self.capacity) };
        }

        // Safety:
        // - We checked that we have enough allocation above.
        // - grow_array allocates properly aligned data for Value
        unsafe { *self.values.add(self.count) = value };
        self.count += 1;
    }

    pub fn free_value_array(&mut self) {
        // Safety:
        // - always allocated from calls to grow_array
        unsafe { free_array(self.values, self.capacity) };
        *self = Self::new();
    }

    pub fn print_value(&self, index: u8) {
        print!("{}", unsafe { *self.values.add(index as usize) })
    }
}
