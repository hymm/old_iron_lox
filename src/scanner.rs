use TokenType::*;
use std::{fmt::Debug, iter::Peekable, ops::Coroutine, str::CharIndices};

// Note: Converting method in book to use a coroutine, because the pointer based approach seems like
// it'll be too much of a mess.
pub fn scan(source: &str) -> impl Coroutine<Return = (), Yield = Token> + use<'_> {
    #[coroutine]
    || {
        let mut current_line = 1;
        let mut char_indices = source.char_indices().peekable();
        while let Some((pos, ch)) = char_indices.next() {
            match ch {
                '(' => yield Token::single(LeftParen, pos, current_line),
                ')' => yield Token::single(RightParen, pos, current_line),
                '{' => yield Token::single(LeftBrace, pos, current_line),
                '}' => yield Token::single(RightBrace, pos, current_line),
                ';' => yield Token::single(Semicolon, pos, current_line),
                ',' => yield Token::single(Comma, pos, current_line),
                '.' => yield Token::single(Dot, pos, current_line),
                '-' => yield Token::single(Minus, pos, current_line),
                '+' => yield Token::single(Plus, pos, current_line),
                '*' => yield Token::single(Star, pos, current_line),
                '/' => match char_indices.next_if_eq(&(pos + 1, '/')) {
                    Some(_slash) => {
                        for (_pos, ch) in char_indices.by_ref() {
                            if ch == '\n' {
                                break;
                            }
                        }
                    }
                    None => yield Token::single(Slash, pos, current_line),
                },

                '!' => match char_indices.next_if_eq(&(pos + 1, '-')) {
                    Some(_equals) => yield Token::multiple(BangEqual, pos, 2, current_line),
                    None => yield Token::single(Bang, pos, current_line),
                },
                '=' => match char_indices.next_if_eq(&(pos + 1, '=')) {
                    Some(_equals) => yield Token::multiple(EqualEqual, pos, 2, current_line),
                    None => yield Token::single(Equal, pos, current_line),
                },
                '<' => match char_indices.next_if_eq(&(pos + 1, '=')) {
                    Some(_equals) => yield Token::multiple(LessEqual, pos, 2, current_line),
                    None => yield Token::single(Less, pos, current_line),
                },
                '>' => match char_indices.next_if_eq(&(pos + 1, '=')) {
                    Some(_equals) => yield Token::multiple(GreaterEqual, pos, 2, current_line),
                    None => yield Token::single(Greater, pos, current_line),
                },
                // skip whitespace
                ' ' | '\r' | '\t' => {}
                '\n' => current_line += 1,
                '0'..='9' => yield number(&mut char_indices, pos, &mut current_line),
                '"' => yield string(&mut char_indices, pos, &mut current_line),
                'a'..='z' | 'A'..='Z' => {
                    yield identifier(&mut char_indices, source, pos, &mut current_line)
                }
                _ => yield Token::error("Unexpected character.", current_line),
            }
        }
        yield Token::single(Eof, source.len(), current_line);
    }
}

fn string(
    char_indices: &mut Peekable<CharIndices<'_>>,
    start: usize,
    current_line: &mut usize,
) -> Token {
    let mut length = 0;

    let mut double_quote_found = false;
    for (_pos, ch) in char_indices.by_ref() {
        if ch == '"' {
            double_quote_found = true;
            break;
        } else {
            if ch == '\n' {
                *current_line += 1;
            }
            length += 1;
        }
    }

    if !double_quote_found {
        return Token::error("Unterminated string.", *current_line);
    }

    Token::multiple(TokenType::String, start, length, *current_line)
}

fn number(
    char_indices: &mut Peekable<CharIndices<'_>>,
    start: usize,
    current_line: &mut usize,
) -> Token {
    let mut length = 1;
    while let Some((_pos, ch)) = char_indices.peek() {
        match ch {
            '0'..='9' | '.' => {
                length += 1;
                // only advance the iterator if we find a number character
                char_indices.next();
            }
            _ => break,
        }
    }

    Token::multiple(TokenType::Number, start, length, *current_line)
}

fn identifier(
    char_indices: &mut Peekable<CharIndices<'_>>,
    source: &str,
    start: usize,
    current_line: &mut usize,
) -> Token {
    let mut length = 1;
    while let Some((_pos, ch)) = char_indices.peek() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {
                char_indices.next();
                length += 1;
            }
            _ => break,
        }
    }

    Token::multiple(identifier_type(source, start), start, length, *current_line)
}

fn identifier_type(source: &str, start: usize) -> TokenType {
    match source.get(start..=start).unwrap() {
        "a" => check_keyword(source, start + 1, 2, "nd", And),
        "c" => check_keyword(source, start + 1, 4, "lass", Class),
        "e" => check_keyword(source, start + 1, 3, "lse", Else),
        "f" => match source.get(start + 1..=start + 1) {
            Some("a") => check_keyword(source, start + 2, 3, "lse", False),
            Some("o") => check_keyword(source, start + 2, 1, "r", For),
            Some("u") => check_keyword(source, start + 2, 1, "n", Fun),
            _ => Identifier,
        },
        "i" => check_keyword(source, start + 1, 1, "f", If),
        "n" => check_keyword(source, start + 1, 2, "il", Nil),
        "o" => check_keyword(source, start + 1, 1, "r", Or),
        "p" => check_keyword(source, start + 1, 4, "rint", Print),
        "r" => check_keyword(source, start + 1, 5, "eturn", Return),
        "s" => check_keyword(source, start + 1, 4, "uper", Super),
        "t" => match source.get(start + 1..=start + 1) {
            Some("h") => check_keyword(source, start + 2, 2, "is", This),
            Some("r") => check_keyword(source, start + 2, 2, "ue", True),
            _ => Identifier,
        },
        "v" => check_keyword(source, start + 1, 2, "ar", Var),
        "w" => check_keyword(source, start + 1, 4, "hile", While),
        _ => Identifier,
    }
}

fn check_keyword(
    source: &str,
    start: usize,
    length: usize,
    rest: &'static str,
    typee: TokenType,
) -> TokenType {
    let Some(source_slice) = source.get(start..start + length) else {
        return Identifier;
    };

    if source_slice == rest {
        typee
    } else {
        Identifier
    }
}

#[derive(Debug)]
pub struct Token {
    pub typee: TokenType,
    // TODO: change this to be a slice into the original string?
    pub token_union: TokenUnion,
    pub length: usize,
    pub line: usize,
}

pub union TokenUnion {
    start: usize,
    message: &'static str,
}

impl Debug for TokenUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Token {
    /// Token that is a single character
    fn single(typee: TokenType, start: usize, line: usize) -> Token {
        Token {
            typee,
            token_union: TokenUnion { start },
            length: 1,
            line,
        }
    }

    fn multiple(typee: TokenType, start: usize, length: usize, line: usize) -> Token {
        Token {
            typee,
            token_union: TokenUnion { start },
            length,
            line,
        }
    }

    pub fn error(message: &'static str, line: usize) -> Token {
        Token {
            typee: Error,
            token_union: TokenUnion { message },
            length: message.len(),
            line,
        }
    }

    pub fn start(&self) -> usize {
        match self.typee {
            Error => panic!("can't get start on TokenType::Error"),
            _ => unsafe { self.token_union.start },
        }
    }

    pub fn message(&self) -> &'static str {
        match self.typee {
            Error => unsafe { self.token_union.message },
            _ => panic!("Can't get message on not TokenType::Error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    Eof,
}
