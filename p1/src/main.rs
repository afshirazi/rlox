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

struct LoxError {
    line: u32,
    loc_in_line: u32,
    chars_in_line: String,
    message: String,
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
        let mut parser = Parser::new();
        self.run(&file, &mut parser);
    }

    fn run_prompt(&mut self) {
        let mut input = String::new();
        let mut parser = Parser::new();
        loop {
            if self.has_error {
                self.has_error = false;
            }

            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim_end() == "exit" {
                return;
            }

            self.run(&input, &mut parser);
        }
    }

    fn run(&mut self, source: &str, parser: &mut Parser) {
        let mut scanner = Scanner::new(source.to_owned(), self);
        scanner.scan_tokens();
        let tokens = scanner.tokens();
        parser.reset_tokens(tokens);

        parser
            .parse()
            .into_iter()
            .filter_map(|stmt| match stmt {
                Ok(s) => Some(s.interpret_stmt()),
                Err(err) => {
                    self.report(err.line, err.loc_in_line, &err.chars_in_line, &err.message);
                    None
                }
            })
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

impl LoxError {
    fn new(line: u32, loc_in_line: u32, chars_in_line: String, message: String) -> Self {
        Self {
            line,
            loc_in_line,
            chars_in_line,
            message,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    Lox::init(args);
}
