use std::ptr::null_mut;

use crate::{
    chunk::{Chunk, OpCode},
    scanner::{
        scan, Token,
        TokenType::{self, *},
    },
    value::Value,
};

static mut COMPILING_CHUNK: *mut Chunk = null_mut();

// NOTE: lifetime here is incorrect
fn current_chunk() -> &'static mut Chunk {
    unsafe { &mut *COMPILING_CHUNK }
}

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    // for token in std::iter::from_coroutine(scan(source)) {
    unsafe {
        COMPILING_CHUNK = chunk as *mut Chunk;
    }

    // Note: having trouble naming this type, so not able to store it in Parser, so
    // just explicitly passing it to methods instead
    let mut token_iter = std::iter::from_coroutine(scan(source)).peekable();
    let mut parser = Parser {
        current: Token::error("uninitialized", 0),
        previous: Token::error("uninitialized", 0),
        had_error: false,
        panic_mode: false,
    };
    parser.advance(&mut token_iter);
    parser.expression();
    parser.consume(&mut token_iter, TokenType::Eof, "Expect end of expression");
    parser.end_compiler();
    !parser.had_error
}

struct Parser {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn advance(&mut self, token_iter: &mut impl Iterator<Item = Token>) {
        std::mem::swap(&mut self.previous, &mut self.current);
        loop {
            let Some(token) = token_iter.next() else {
                return;
            };

            self.current = token;
            if !matches!(self.current.typee, TokenType::Error) {
                break;
            }

            self.error_at_current(self.current.message());
        }
    }

    fn consume(
        &mut self,
        token_iter: &mut impl Iterator<Item = Token>,
        token: TokenType,
        message: &'static str,
    ) {
        if self.current.typee == token {
            token_iter.next();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&self, byte: u8) {
        current_chunk().write_chunk(byte, self.previous.line);
    }

    fn emit_bytes(&self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end_compiler(&self) {
        self.emit_return();
    }

    fn grouping(&mut self, token_iter: &mut impl Iterator<Item = Token>) {
        self.expression();
        self.consume(token_iter, RightParen, "Expect ')' after expression.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self, source: &str) {
        let start = self.previous.start();
        let str_value = source.get(start..start + self.previous.length).unwrap();
        let value: f64 = str_value.parse::<f64>().unwrap();
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let operator_type = self.previous.typee;
        self.parse_precedence(Precedence::Unary);

        if matches!(operator_type, Minus) {
            self.emit_byte(OpCode::Negate as u8);
        } else {
            unreachable!();
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {}

    fn emit_return(&self) {
        self.emit_byte(OpCode::Return as u8);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = current_chunk().add_constant(value);
        // TODO: this doesn't look quite right
        if constant > u8::MAX.into() {
            self.error("Too many constants in one chunk.");
            0
        } else {
            constant as u8
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant);
    }

    fn error_at_current(&mut self, message: &'static str) {
        error_at(
            &self.current,
            message,
            &mut self.had_error,
            &mut self.panic_mode,
        );
    }

    fn error(&mut self, message: &'static str) {
        error_at(
            &self.previous,
            message,
            &mut self.had_error,
            &mut self.panic_mode,
        );
    }
}

enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

fn error_at(token: &Token, message: &'static str, had_error: &mut bool, panic_mode: &mut bool) {
    if *panic_mode {
        return;
    }
    *panic_mode = true;
    eprint!("[line {}] Error", token.line);

    match token.typee {
        TokenType::Eof => eprint!(" at end"),
        TokenType::Error => {}
        _ => eprint!(" at '{:.*}'", token.length, token.start()),
    }

    eprintln!(": {message}");
    *had_error = true;
}
