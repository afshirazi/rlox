use std::{env, fs, io, process::exit};

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
    io::stdin().read_line(&mut input).unwrap();
    println!("Your input was: {input}");
}
