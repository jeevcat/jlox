use std::{
    env, fs,
    io::{self, Write},
    process,
};

mod scanner;

fn main() {
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
    dbg!(path);
    let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
    run(&contents);
}

fn run_prompt() {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().expect("flush failed!");
        let mut buf = String::new();
        stdin
            .read_line(&mut buf)
            .expect("Something went wrong reading from stdin");
        run(buf.trim());
    }
}

fn run(source: &str) {
    let tokens = scanner::scan_tokens(source).unwrap();
    for token in tokens.iter() {
        dbg!(token);
    }
}
