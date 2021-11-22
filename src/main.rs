use std::{
    env, fs,
    io::{self, Write},
    process,
};

use anyhow::Result;
use log::error;
use runtime::interpreter::Interpreter;

use crate::resolver::Resolver;

mod ast;
mod error;
mod parser;
mod resolver;
mod runtime;
mod scanner;

fn main() {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: jlox [script]");
        process::exit(64);
    } else if let Some(arg) = args.get(1) {
        run_file(arg);
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) {
    let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
    let mut interpreter = Interpreter::new();
    run_errored(&mut interpreter, &contents);
}

fn run_prompt() {
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("flush failed!");
        let mut buf = String::new();
        stdin
            .read_line(&mut buf)
            .expect("Something went wrong reading from stdin");
        run_errored(&mut interpreter, buf.trim());
    }
}

fn run_errored(interpreter: &mut Interpreter, source: &str) {
    match run(interpreter, source) {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);
        }
    }
}

fn run(interpreter: &mut Interpreter, source: &str) -> Result<()> {
    let tokens = scanner::scan_tokens(source)?;
    let parser = parser::Parser::new(tokens);
    let statements = parser.parse()?;
    let mut resolver = Resolver::new(interpreter);
    resolver.resolve_statements(&statements);
    interpreter.interpret(&statements)?;
    Ok(())
}
