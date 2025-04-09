use std::{
    env, fs,
    io::{self},
};
mod scanner;
mod tokens;

struct Lox {
    hasError: bool,
}

impl Lox {
    fn init(args: Vec<String>) -> () {
        let mut lox = Lox { hasError: false };

        match args.len() {
            1 => lox.run_prompt(),
            2 => lox.run_file(&args[1]),
            _ => eprintln!("Usage: rlox <file>"),
        };
    }

    fn run_file(&self, file_name: &str) -> () {
        let file = fs::read_to_string(file_name).unwrap();
        Self::run(&file);
    }

    fn run_prompt(&mut self) {
        let mut input = String::new();
        loop {
            if self.hasError {
                self.hasError = false;
            }

            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim_end() == "exit" {
                return;
            }

            Self::run(&input);
        }
    }

    fn run(source: &str) {
        // do something
    }

    fn report(&mut self, line: u32, loc_in_line: u32, chars_in_line: &str, message: &str) {
        //TODO: make better
        eprintln!("[Line {line}:{loc_in_line}] Error at {chars_in_line}: {message}.");
        self.hasError = true;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    Lox::init(args);
}
