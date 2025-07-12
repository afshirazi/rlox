use std::{
    env, fs,
    io::{self},
};

use parser::Parser;
use scanner::Scanner;

mod environment;
mod expr;
mod parser;
mod scanner;
mod stmt;
mod tokens;

struct Lox {
    has_error: bool,
}

impl Lox {
    fn init(args: Vec<String>) {
        let mut lox = Lox { has_error: false };

        match args.len() {
            1 => lox.run_prompt(),
            2 => lox.run_file(&args[1]),
            _ => eprintln!("Usage: rlox <file>"),
        };
    }

    fn run_file(&mut self, file_name: &str) {
        let file = fs::read_to_string(file_name).unwrap();
        self.run(&file);
    }

    fn run_prompt(&mut self) {
        let mut input = String::new();
        loop {
            if self.has_error {
                self.has_error = false;
            }

            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim_end() == "exit" {
                return;
            }

            self.run(&input);
        }
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source.to_owned(), self);
        scanner.scan_tokens();
        let tokens = scanner.tokens(); // TODO: move scanner and parser out of run to maintain Environment state in REPL
        let mut parser = Parser::new(tokens, self);

        parser
            .parse()
            .into_iter()
            .map(|stmt| stmt.interpret_stmt())
            .fold(Ok(()), |acc, el| match (acc, el) {
                (Err(s), Err(discard)) => {
                    eprintln!("{discard}");
                    Err(s)
                }
                (Ok(_), Err(disc)) => {
                    eprintln!("{disc}");
                    Err("Something went wrong, check error messages")
                }
                (Err(s), Ok(_)) => Err(s),
                (Ok(_), Ok(_)) => Ok(()),
            })
            .err()
            .map(|err| eprintln!("{err}"));
    }

    fn report(&mut self, line: u32, loc_in_line: u32, chars_in_line: &str, message: &str) {
        //TODO: make better
        eprintln!("[Line {line}:{loc_in_line}] Error at {chars_in_line}: {message}.");
        self.has_error = true;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    Lox::init(args);
}
