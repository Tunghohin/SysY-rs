mod formatter;
mod lexer;
mod parser;
mod semantic;

mod codegen;

use crate::codegen::asm::RV32IASMGenerator;
use crate::codegen::ir::LLVMIRGenerator;
use crate::codegen::regs::NoneRegisterAllocator;
use inkwell::context::Context;

use crate::parser::parse;
use parser::BuildConfig;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).expect("Failed to read file");
    let ast = parse(&input, BuildConfig::default())
        .map_err(|e| println!("{}", e))
        .unwrap_or_else(|_| panic!("Failed to parse source code"));

    let context = Context::create();
    let mut llvm_ir_gen = LLVMIRGenerator::new("module", &context);
    llvm_ir_gen
        .gen_ir(&ast)
        .unwrap_or_else(|e| panic!("Failed to generate LLVM IR: {}", e));
    llvm_ir_gen.optimize();

    let mut asm_gen: RV32IASMGenerator<NoneRegisterAllocator> =
        RV32IASMGenerator::new(llvm_ir_gen, "main");
    asm_gen
        .gen_asm()
        .unwrap_or_else(|e| panic!("Failed to generate assembly: {}", e));
    println!("{}", asm_gen.emit());
}
