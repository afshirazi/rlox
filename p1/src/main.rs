use std::{
    env, fs,
    io::{self, Write},
    process::exit,
};
mod tokens;

use tokens::Token;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Usage: rlox <file>");
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(file_name: &str) -> () {
    let file = fs::read_to_string(file_name).unwrap();
    println!("{file}");
}

fn run_prompt() -> () {
    let mut input = String::new();
    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim_end() == "exit" {
            return;
        }
        println!("Your input was: {input}");
    }
}

fn report(line: u32, where_in_line: &str, message: &str) -> () {
    //TODO: make better
    eprintln!("[Line {line}]: Error at {where_in_line}. {message}.");
}
