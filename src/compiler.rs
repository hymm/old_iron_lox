use std::{iter::Peekable, ops::Coroutine};

use crate::{
    chunk::Chunk,
    scanner::{scan, Token, TokenType},
};

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    // for token in std::iter::from_coroutine(scan(source)) {

    let token_iter = std::iter::from_coroutine(scan(source)).peekable();
    let mut parser = Parser {
        current: Token::error("uninitialized", 0),
        previous: Token::error("uninitialized", 0),
        had_error: false,
        panic_mode: false,
    };
    parser.advance(token_iter);
    expression();
    parser.consume(token_iter, TokenType::Eof, "Expect end of expression");

    !parser.had_error
}

struct Parser {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn advance(&mut self, mut token_iter: impl Iterator<Item = Token>) {
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
        mut token_iter: impl Iterator<Item = Token>,
        token: TokenType,
        message: &'static str,
    ) {
        if self.current.typee == token {
            token_iter.next();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &'static str) {
        error_at(
            &self.current,
            message,
            &mut self.had_error,
            &mut self.panic_mode,
        );
    }

    fn error(&mut self) {
        error_at(
            &self.previous,
            self.previous.message(),
            &mut self.had_error,
            &mut self.panic_mode,
        );
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
