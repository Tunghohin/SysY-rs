use std::{env, fs};

mod rust_mir_gen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let src = fs::read_to_string(&args[1]).expect("Failed to read file");
    let syntax = syn::parse_file(&src).expect("Failed to parse file");
    rust_mir_gen::gen_mir(&syntax);
}
