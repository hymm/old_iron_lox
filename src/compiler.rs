use crate::scanner::{scan, TokenType};

pub fn compile(source: &str) {
    let mut line = 0;
    for token in std::iter::from_coroutine(scan(source)) {
        if token.line != line {
            print!("{:04} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        match token.typee {
            TokenType::Error => println!("error!!!"),
            _ => println!("{:?} {:.*}", token.typee, token.length, token.start()),
        }
    }
}
