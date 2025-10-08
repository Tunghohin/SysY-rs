pub mod scope;
pub mod symbol_table;

use crate::parser::ast::{AstNode, AstNodeInner};

use self::scope::ScopeStack;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::{IntPredicate, OptimizationLevel};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Codegen<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    scope: ScopeStack<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(module_name: &str, ctx: &'ctx Context) -> Self {
        let module = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        Self {
            ctx,
            module,
            builder,
            scope: ScopeStack::default(),
        }
    }

    pub fn gen_llvm_ir(&mut self, ast: &AstNode) -> Result<(), String> {
        if !matches!(ast.as_inner(), AstNodeInner::CompUnit(_)) {
            return Err("Expected CompUnit as root node".to_string());
        }
        Ok(())
    }

    pub fn execute(&self) {
        let engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        unsafe {
            let main = engine
                .get_function::<unsafe extern "C" fn() -> i32>("main")
                .unwrap();
            println!("Program returned: {}", main.call());
        }
    }

    fn gen_comp_unit(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::CompUnit(children) = node.as_inner() else {
            return Err("Invalid CompUnit node".to_string());
        };
        for child in children {
            match child.as_inner() {
                AstNodeInner::Decl(_) => self.gen_decl(child)?,
                AstNodeInner::FuncDef { .. } => self.gen_func_def(child)?,
                _ => return Err("Unexpected node in CompUnit".to_string()),
            }
        }
        Ok(())
    }

    fn gen_decl(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::Decl(child) = node.as_inner() else {
            return Err("Invalid Decl node".to_string());
        };
        match child.as_inner() {
            AstNodeInner::ConstDecl { .. } => self.gen_const_decl(child),
            AstNodeInner::VarDecl { .. } => self.gen_var_decl(child),
            _ => Err("Unexpected node in Decl".to_string()),
        }
    }

    fn gen_const_decl(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::ConstDecl { btype, const_defs } = node.as_inner() else {
            return Err("Invalid ConstDecl node".to_string());
        };
        for const_def in const_defs {
            self.gen_const_def(const_def, btype)?;
        }

        Ok(())
    }

    fn gen_const_def(&mut self, node: &AstNode, btype: &AstNode) -> Result<(), String> {
        let AstNodeInner::ConstDef {
            ident,
            dimensions,
            init_val,
        } = node.as_inner()
        else {
            return Err("Invalid ConstDef node".to_string());
        };

        Ok(())
    }

    fn gen_var_decl(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::VarDecl { btype, var_defs } = node.as_inner() else {
            return Err("Invalid VarDecl node".to_string());
        };

        Ok(())
    }

    fn gen_var_def(&mut self, node: &AstNode, btype: &AstNode) -> Result<(), String> {
        let AstNodeInner::VarDef {
            ident,
            dimensions,
            init_val,
        } = node.as_inner()
        else {
            return Err("Invalid VarDef node".to_string());
        };

        Ok(())
    }

    fn gen_func_def(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::FuncDef {
            func_type,
            ident,
            params,
            body,
        } = node.as_inner()
        else {
            return Err("Invalid FuncDef node".to_string());
        };

        Ok(())
    }

    fn gen_block(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::Block(stmts) = node.as_inner() else {
            return Err("Invalid Block node".to_string());
        };
        Ok(())
    }

    fn gen_stmt(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::Stmt(stmt_inner) = node.as_inner() else {
            return Err("Invalid Stmt node".to_string());
        };
        match stmt_inner {
            _ => Err("Unimplemented statement type".to_string()),
        }
    }

    fn gen_exp(&mut self, node: &AstNode) -> Result<BasicValueEnum<'ctx>, String> {
        match node.as_inner() {
            AstNodeInner::Exp(child)
            | AstNodeInner::Cond(child)
            | AstNodeInner::ConstExp(child) => self.gen_exp(child),
            AstNodeInner::Number(_) => self.gen_number(node),
            _ => Err("Unimplemented expression type".to_string()),
        }
    }

    fn gen_lval(&mut self, node: &AstNode) -> Result<BasicValueEnum<'ctx>, String> {
        let AstNodeInner::LVal { ident, dimensions } = node.as_inner() else {
            return Err("Invalid LVal node".to_string());
        };
        unimplemented!()
    }

    fn gen_number(&mut self, node: &AstNode) -> Result<BasicValueEnum<'ctx>, String> {
        let AstNodeInner::Number(value) = node.as_inner() else {
            return Err("Invalid Number node".to_string());
        };
        let literal: i32 = value.clone().try_into()?;
        Ok(self.ctx.i32_type().const_int(literal as u64, false).into())
    }
}

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
