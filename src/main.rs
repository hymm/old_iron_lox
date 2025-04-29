#![feature(coroutines, coroutine_trait, iter_from_coroutine)]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

use std::{env, fs, io::{stdin, Write}, process::exit};

use vm::{free_vm, init_vm, interpret, InterpretError};

mod chunk;
mod compiler;
mod debug;
mod memory;
mod scanner;
mod value;
mod vm;

fn main() {
    init_vm();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: clox [path]");
            exit(64);
        }
    }

    free_vm();
    exit(0);
}

fn repl() {
    let mut line = String::new();
    loop {
        println!("Starting repl...");
        print!("> ");

        std::io::stdout().flush().unwrap();
        stdin().read_line(&mut line).expect("Did not get line");

        let _ = interpret(&line);
    }
}

fn run_file(path: &str) {
    let Ok(source) = fs::read_to_string(path) else {
        println!("Could not open file {path}");
        exit(74);
    };
    println!("running {path}");
    let result = interpret(&source);
    match result {
        Err(InterpretError::CompileError) => todo!(),
        Err(InterpretError::RuntimeError) => todo!(),
        Ok(_) => {}
    }
}
