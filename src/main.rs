use std::{env, fs};

mod formatter;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).expect("Failed to read file");

    println!(
        "{}",
        formatter::format_ast(
            &crate::parser::parse(&input, crate::parser::BuildConfig::default()).unwrap_or_else(
                |e| {
                    println!("{}", e);
                    panic!()
                }
            )
        )
    );
}
