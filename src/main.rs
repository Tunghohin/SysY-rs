use std::{env, fs};

mod formatter;
mod lexer;
mod parser;
mod semantic;

mod codegen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).expect("Failed to read file");
    let ast = parser::parse(&input, parser::BuildConfig::default())
        .map_err(|e| println!("{}", e))
        .unwrap_or_else(|_| panic!("Failed to parse source code"));
    let context = inkwell::context::Context::create();
    let mut codegen = codegen::Codegen::new("module", &context);
    codegen
        .gen_ir(&ast)
        .unwrap_or_else(|e| panic!("Failed to generate LLVM IR: {}", e));

    let output = args.get(2).cloned().unwrap_or("output.ll".to_string());
    fs::write(&output, codegen.print_to_string())
        .unwrap_or_else(|e| panic!("Failed to write LLVM IR to file: {}", e));
}
