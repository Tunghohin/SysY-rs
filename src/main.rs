use std::{env, fs};

mod formatter;
mod lexer;
mod parser;
mod semantic;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).expect("Failed to read file");
    let _ = parser::parse(&input, parser::BuildConfig::default()).map_or_else(
        |e| eprintln!("{}", e),
        |_| {
            eprintln!("No semantic errors in the program!");
        },
    );
}
