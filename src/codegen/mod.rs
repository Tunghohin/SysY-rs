#![allow(unused)]

pub mod asm;
pub mod ir;
pub mod regs;
pub mod scope;
pub mod symbol_table;

use crate::parser;
use crate::parser::BuildConfig;
use crate::parser::PrimaryExpInner;
use crate::parser::ast::{
    AddOpInner, AstNode, AstNodeInner, ConstInitValInner, InitValInner, MulOpInner, RelOpInner,
    StmtInner, UnaryExpInner, UnaryOpInner,
};
use crate::parser::parse;

use crate::codegen::asm::RV32IASMGenerator;
use crate::codegen::ir::LLVMIRGenerator;
use crate::codegen::regs::NoneRegisterAllocator;
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicValue, BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue,
};

#[test]
fn example() {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let function = module.add_function("add", fn_type, None);

    let entry_block = context.append_basic_block(function, "entry");

    builder.position_at_end(entry_block);
    let a = function.get_first_param().unwrap().into_int_value();
    let b = function.get_last_param().unwrap().into_int_value();

    a.set_name("a");
    b.set_name("b");

    let sum = builder.build_int_add(a, b, "sum").unwrap();
    let _ = builder.build_return(Some(&sum));

    module.print_to_file("output.ll").unwrap();
}

#[test]
fn codegen_dummy() {
    let context = Context::create();
    let mut codegen = LLVMIRGenerator::new("test", &context);
    codegen.gen_dummy();
    codegen.execute();
    codegen.print_to_stderr();
}

#[test]
fn codegen() {
    let src = std::fs::read_to_string("tests/codegen/asm3.in").unwrap_or_default();
    let ast = parse(&src, BuildConfig::default())
        .map_err(|e| println!("{}", e))
        .unwrap_or_else(|_| panic!("Failed to parse source code"));

    // parser::display_ast(&src);

    let context = Context::create();
    let mut llvm_ir_gen = LLVMIRGenerator::new("module", &context);
    llvm_ir_gen
        .gen_ir(&ast)
        .unwrap_or_else(|e| panic!("Failed to generate LLVM IR: {}", e));
    llvm_ir_gen.optimize();

    llvm_ir_gen.print_to_stderr();

    let ret = llvm_ir_gen
        .execute()
        .unwrap_or_else(|e| panic!("Failed to execute: {}", e));
    println!("Program return: {}\n\n\n", ret);

    let mut asm_gen: RV32IASMGenerator<NoneRegisterAllocator> =
        RV32IASMGenerator::new(llvm_ir_gen, "main");
    asm_gen
        .gen_asm()
        .unwrap_or_else(|e| panic!("Failed to generate assembly: {}", e));
    std::fs::write("./solution.txt", asm_gen.emit()).expect("Unable to write file");

    let output = std::process::Command::new("java")
        .args([
            "-jar",
            "/home/sgimage/Labs/rars.jar",
            "/home/sgimage/Labs/compiler/solution.txt",
            "ic",
            "a0",
        ])
        .output()
        .expect("failed to run RARS");

    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    // run rars
}
