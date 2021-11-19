use std::{
    env, fs,
    io::{self, Write},
    process,
};

use interpreter::Interpreter;
use log::error;

mod environment;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;

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
    run(&mut interpreter, &contents);
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
        run(&mut interpreter, buf.trim());
    }
}

fn run(interpreter: &mut Interpreter, source: &str) {
    let tokens = scanner::scan_tokens(source);
    match tokens {
        Ok(tokens) => {
            let parser = parser::Parser::new(tokens);
            let statements = parser.parse();
            match statements {
                Ok(statements) => {
                    for statement in &statements {
                        let val = interpreter.execute(statement);
                        match val {
                            Ok(_) => {}
                            Err(e) => error!("{}", e),
                        }
                    }
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}
