use std::{env, fs};

mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).expect("Failed to read file");

    lexer::tokenize(&input)
        .unwrap_or_else(|errs| {
            errs.iter().for_each(|err| {
                eprintln!("{}", err);
            });
            vec![]
        })
        .iter()
        .for_each(|token| match token.kind {
            lexer::token::TokenKind::Eof => {}
            _ => {
                eprintln!("{}", token);
            }
        })
}