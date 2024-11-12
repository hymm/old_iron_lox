use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disassemble_chunk(&self, name: &'static str) {
        println!("== {} ==", name);
        let mut offset: isize = 0;
        while offset < self.count as isize {
            // SAFETY: while loop is limited by self.count.
            offset = unsafe { self.disassemble_instruction(offset) };
        }
    }

    /// SAFETY:
    /// - offset cannot go outside of the allocation of chuck
    pub unsafe fn disassemble_instruction(&self, offset: isize) -> isize {
        print!("{:04} ", offset);
        if offset > 0 && unsafe { *self.lines.offset(offset) == *self.lines.offset(offset - 1) } {
            print!("   | ");
        } else {
            print!("{:04} ", unsafe { *self.lines.offset(offset) });
        }

        // SAFETY: is ensured by caller
        let instruction: OpCode = unsafe { *self.code.byte_offset(offset) }.into();
        match instruction {
            OpCode::OpReturn => simple_instruction("OpReturn", offset),
            OpCode::OpConstant => self.constant_instruction("OpConstant", offset),
        }
    }

    fn constant_instruction(&self, name: &'static str, offset: isize) -> isize {
        let constant = unsafe { *self.code.offset(offset + 1) };
        print!("{name:<16} {offset:04} ");
        self.constants.print_value(constant);
        println!();
        offset + 2
    }
}

fn simple_instruction(name: &'static str, offset: isize) -> isize {
    println!("{}", name);
    offset + 1
}
