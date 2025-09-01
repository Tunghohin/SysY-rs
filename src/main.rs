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

#[test]
fn test_lexer() {
    let case_dir = std::path::Path::new("./lexer/tests");

    let mut entries: Vec<_> = fs::read_dir(case_dir)
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.extension().map(|e| e == "in").unwrap_or(false))
        .collect();

    entries.sort();

    for ent in entries {
        println!("{}", ent.to_str().unwrap());
    }
}
