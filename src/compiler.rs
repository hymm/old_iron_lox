use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{fmt::Debug, ptr::null_mut};

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
    let token_iter = std::iter::from_coroutine(scan(source)).peekable();
    let mut parser = Parser {
        current: Token::error("uninitialized", 0),
        previous: Token::error("uninitialized", 0),
        had_error: false,
        panic_mode: false,
        token_iter: Box::new(token_iter),
        source,
    };
    parser.advance();
    parser.expression();
    parser.consume(TokenType::Eof, "Expect end of expression");
    parser.end_compiler();
    !parser.had_error
}

struct Parser<'iter> {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    token_iter: Box<dyn Iterator<Item = Token> + 'iter>,
    source: &'iter str,
}

impl<'iter> Debug for Parser<'iter> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("current", &self.current)
            .field("previous", &self.previous)
            .field("had_error", &self.had_error)
            .field("panic_mode", &self.panic_mode)
            .field("source", &self.source)
            .finish()
    }
}

impl<'iter> Parser<'iter> {
    fn advance(&mut self) {
        std::mem::swap(&mut self.previous, &mut self.current);
        loop {
            let Some(token) = self.token_iter.next() else {
                return;
            };

            self.current = token;
            if !matches!(self.current.typee, TokenType::Error) {
                break;
            }

            self.error_at_current(self.current.message());
        }
    }

    fn consume(&mut self, token: TokenType, message: &'static str) {
        if self.current.typee == token {
            self.token_iter.next();
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

        #[cfg(feature = "debug_print_code")]
        if !self.had_error {
            let chunk = unsafe { &mut *VM.chunk };
            chunk.disassemble_chunk(self.source);
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.typee;
        let ParseRule((_, _, precedence)) = operator_type.rule();
        self.parse_precedence(precedence.next());

        match operator_type {
            Plus => self.emit_byte(OpCode::Add as u8),
            Minus => self.emit_byte(OpCode::Subtract as u8),
            Star => self.emit_byte(OpCode::Multiply as u8),
            Slash => self.emit_byte(OpCode::Divide as u8),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(RightParen, "Expect ')' after expression.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let start = self.previous.start();
        let str_value = self
            .source
            .get(start..(start + self.previous.length))
            .unwrap();
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

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let ParseRule((Some(prefix_rule), _, _)) = TokenType::rule(self.previous.typee) {
            prefix_rule(self);
        } else {
            self.error("expect expression");
        }

        while let ParseRule((_, _, current_precendence)) = TokenType::rule(self.current.typee)
            && precedence <= current_precendence
        {
            self.advance();
            let ParseRule((_, Some(infix_rule), _)) = TokenType::rule(self.previous.typee) else {
                return;
            };
            infix_rule(self);
        }
    }

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

#[derive(Clone, Copy, PartialEq, PartialOrd, FromPrimitive)]
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

impl Precedence {
    fn next(&self) -> Precedence {
        FromPrimitive::from_u8(*self as u8 + 1).unwrap()
    }
}

struct ParseRule<'iter>((Option<ParseFn<'iter>>, Option<ParseFn<'iter>>, Precedence));

type ParseFn<'iter> = fn(&mut Parser<'iter>);

impl TokenType {
    fn rule<'a>(self) -> ParseRule<'a> {
        match self {
            LeftParen => ParseRule((Some(Parser::grouping), None, Precedence::None)),
            Minus => ParseRule((Some(Parser::unary), Some(Parser::binary), Precedence::Term)),
            Plus => ParseRule((None, Some(Parser::binary), Precedence::Term)),
            Slash => ParseRule((None, Some(Parser::binary), Precedence::Factor)),
            Star => ParseRule((None, Some(Parser::binary), Precedence::Factor)),
            Number => ParseRule((Some(Parser::number), None, Precedence::None)),
            RightParen | LeftBrace | RightBrace | Comma | Dot | Semicolon | Bang | BangEqual
            | Equal | EqualEqual | Greater | GreaterEqual | Less | LessEqual | Identifier
            | String | And | Class | Else | False | For | Fun | If | Nil | Or | Print | Return
            | Super | This | True | Var | While | Error | Eof => {
                ParseRule((None, None, Precedence::None))
            }
        }
    }
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
