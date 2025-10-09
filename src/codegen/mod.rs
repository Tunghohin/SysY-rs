#![allow(unused)]

pub mod scope;
pub mod symbol_table;

use crate::parser::BuildConfig;
use crate::parser::PrimaryExpInner;
use crate::parser::ast::{
    AddOpInner, AstNode, AstNodeInner, ConstInitValInner, InitValInner, MulOpInner, RelOpInner,
    StmtInner, UnaryExpInner, UnaryOpInner,
};
use crate::parser::parse;

use self::scope::ScopeStack;
use core::panic;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicValue, BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue,
};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Codegen<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    scope_stk: ScopeStack<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(module_name: &str, ctx: &'ctx Context) -> Self {
        let module = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        Self {
            ctx,
            module,
            builder,
            scope_stk: ScopeStack::default(),
        }
    }

    pub fn gen_llvm_ir(&mut self, ast: &AstNode) -> Result<(), String> {
        if !matches!(ast.as_inner(), AstNodeInner::CompUnit(_)) {
            return Err("Expected CompUnit as root node".to_string());
        }
        Ok(())
    }

    pub fn gen_dummy(&mut self) {
        let i32_type = self.ctx.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);

        let entry_block = self.ctx.append_basic_block(function, "entry");

        self.builder.position_at_end(entry_block);
        let ret_val = i32_type.const_int(42, false);
        let _ = self.builder.build_return(Some(&ret_val));
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

    pub fn print_to_stderr(&self) {
        self.module.print_to_stderr();
    }

    fn into_int_value_helper(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, String> {
        match val {
            BasicValueEnum::IntValue(iv) => Ok(iv),
            BasicValueEnum::PointerValue(pv) => {
                let loaded = self
                    .builder
                    .build_load(pv, "loadtmp")
                    .map_err(|e| e.to_string())?;
                match loaded {
                    BasicValueEnum::IntValue(iv) => Ok(iv),
                    _ => Err("Expected IntValue after loading from PointerValue".to_string()),
                }
            }
            _ => Err("Expected IntValue".to_string()),
        }
    }

    fn gen_comp_unit(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::CompUnit(children) = node.as_inner() else {
            return Err("Invalid CompUnit node".to_string());
        };
        for child in children {
            match child.as_inner() {
                AstNodeInner::Decl(_) => self.gen_decl(child, true)?,
                AstNodeInner::FuncDef { .. } => self.gen_func_def(child)?,
                _ => return Err("Unexpected node in CompUnit".to_string()),
            }
        }
        Ok(())
    }

    fn gen_decl(&mut self, node: &AstNode, is_global: bool) -> Result<(), String> {
        let AstNodeInner::Decl(child) = node.as_inner() else {
            return Err("Invalid Decl node".to_string());
        };
        match child.as_inner() {
            AstNodeInner::ConstDecl { .. } => self.gen_const_decl(child, is_global),
            AstNodeInner::VarDecl { .. } => self.gen_var_decl(child, is_global),
            _ => Err("Unexpected node in Decl".to_string()),
        }
    }

    fn gen_const_decl(&mut self, node: &AstNode, is_global: bool) -> Result<(), String> {
        let AstNodeInner::ConstDecl { btype, const_defs } = node.as_inner() else {
            return Err("Invalid ConstDecl node".to_string());
        };
        for const_def in const_defs {
            self.gen_const_def(const_def, btype, is_global)?;
        }

        Ok(())
    }

    fn gen_const_def(
        &mut self,
        node: &AstNode,
        btype: &AstNode,
        is_global: bool,
    ) -> Result<(), String> {
        let AstNodeInner::ConstDef {
            ident,
            dimensions,
            init_val,
        } = node.as_inner()
        else {
            return Err("Invalid ConstDef node".to_string());
        };

        if !dimensions.is_empty() {
            return Err("Array not supported yet".to_string());
        }

        if !matches!(btype.as_inner(), AstNodeInner::BType(t) if t == "int") {
            return Err("Only int type is supported".to_string());
        }

        let AstNodeInner::ConstInitVal(ConstInitValInner::ConstExp(exp_inner)) =
            init_val.as_inner()
        else {
            return Err("Only ConstExp is supported in ConstInitVal".to_string());
        };

        if !dimensions.is_empty() {
            return Err("Array not supported yet".to_string());
        } else {
            let init_val = self.const_expr_inference(exp_inner)?;
            let ty = self.ctx.i32_type();
            let value = ty.const_int(init_val as u64, false);

            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
                global_val.set_initializer(&value);
                global_val.set_constant(true);
                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        crate::codegen::symbol_table::BasicValueEnumWrapper {
                            inner: global_val.as_basic_value_enum(),
                            constness: true,
                        },
                    )?;
            } else {
                return Err("Local const not supported yet".to_string());
            }
        }

        Ok(())
    }

    fn gen_var_decl(&mut self, node: &AstNode, is_global: bool) -> Result<(), String> {
        let AstNodeInner::VarDecl { btype, var_defs } = node.as_inner() else {
            return Err("Invalid VarDecl node".to_string());
        };
        for var_def in var_defs {
            self.gen_var_def(var_def, btype, is_global)?;
        }

        Ok(())
    }

    fn gen_var_def(
        &mut self,
        node: &AstNode,
        btype: &AstNode,
        is_global: bool,
    ) -> Result<(), String> {
        let AstNodeInner::VarDef {
            ident,
            dimensions,
            init_val,
        } = node.as_inner()
        else {
            return Err("Invalid VarDef node".to_string());
        };

        if !dimensions.is_empty() {
            return Err("Array not supported yet".to_string());
        }
        if !matches!(btype.as_inner(), AstNodeInner::BType(t) if t == "int") {
            return Err("Only int type is supported".to_string());
        }

        let Some(init_val) = init_val else {
            return Err("Uninitialized variable is not supported yet".to_string());
        };
        let AstNodeInner::InitVal(InitValInner::ConstExp(exp_inner)) = init_val.as_inner() else {
            return Err("Only Exp is supported in InitVal".to_string());
        };

        if !dimensions.is_empty() {
            return Err("Array not supported yet".to_string());
        } else {
            let init_val = self.const_expr_inference(exp_inner)?;
            let ty = self.ctx.i32_type();
            let value = ty.const_int(init_val as u64, false);

            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
                global_val.set_initializer(&value);
                global_val.set_constant(false);
                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        crate::codegen::symbol_table::BasicValueEnumWrapper {
                            inner: global_val.as_basic_value_enum(),
                            constness: false,
                        },
                    )?;
            } else {
                return Err("Local const not supported yet".to_string());
            }
        }

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

        let AstNodeInner::FuncType(ret_type_str) = func_type.as_inner() else {
            return Err("Invalid function return type".to_string());
        };

        let ret_type = match ret_type_str.as_str() {
            "int" => Some(self.ctx.i32_type()),
            "void" => None,
            _ => return Err("Unsupported return type".to_string()),
        };

        let param_types = if let Some(params_node) = params {
            if let AstNodeInner::FuncFParams(params_list) = params_node.as_inner() {
                params_list
                    .iter()
                    .map(|_| self.ctx.i32_type().into())
                    .collect::<Vec<BasicMetadataTypeEnum>>()
            } else {
                return Err("Invalid function parameters".to_string());
            }
        } else {
            vec![]
        };

        let fn_type = if let Some(ret_ty) = ret_type {
            self.ctx.i32_type().fn_type(&param_types, false)
        } else {
            self.ctx.void_type().fn_type(&param_types, false)
        };

        let function = self.module.add_function(ident.as_str(), fn_type, None);

        let entry_bb = self.ctx.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        let AstNodeInner::Block(items) = body.as_inner() else {
            return Err("Invalid function body".to_string());
        };
        // for item in items {
        //     match item.as_inner() {
        //         AstNodeInner::Decl(_) => self.gen_decl(item, false)?,
        //         AstNodeInner::Stmt(stmt) => match &**stmt {
        //             StmtInner::Return(Some(exp)) => {
        //                 let ret_val = self.gen_exp(exp)?;
        //                 let ret_int = self.into_int_value_helper(ret_val)?;
        //                 let _ = self.builder.build_return(Some(&ret_int));
        //             }
        //             StmtInner::Return(None) => {
        //                 let _ = self.builder.build_return(None);
        //             }
        //             _ => self.gen_stmt(item)?,
        //         },
        //         _ => return Err("Unexpected node in function body".to_string()),
        //     }
        // }
        let _ = self.builder.build_return(Some(
            &self
                .ctx
                .i32_type()
                .const_int(42, false)
                .as_basic_value_enum(),
        ));

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
        Ok(())
    }

    fn const_expr_inference(&mut self, node: &AstNode) -> Result<i32, String> {
        match node.as_inner() {
            AstNodeInner::Exp(exp) | AstNodeInner::Cond(exp) => self.const_expr_inference(exp),
            AstNodeInner::LVal { ident, dimensions } => {
                let Some(var) = self
                    .scope_stk
                    .peek()
                    .unwrap_or_else(|| unreachable!())
                    .resolve(&ident)
                else {
                    return Err(format!("Undefined variable: {}", ident));
                };
                if !var.is_const() {
                    return Err(format!("Variable {} is not constant", ident));
                }
                match var.inner {
                    BasicValueEnum::PointerValue(pv) => {
                        let Some(name) = pv.get_name().to_str().ok() else {
                            return Err("Invalid variable name".to_string());
                        };

                        let Some(iv) = self.module.get_global(name).unwrap().get_initializer()
                        else {
                            return Err("Failed to get initializer for global variable".to_string());
                        };

                        Ok(iv
                            .into_int_value()
                            .get_zero_extended_constant()
                            .unwrap_or_else(|| unreachable!()) as i32)
                    }
                    BasicValueEnum::IntValue(iv) => Ok(iv
                        .get_zero_extended_constant()
                        .unwrap_or_else(|| unreachable!())
                        as i32),

                    _ => unreachable!(),
                }
            }
            AstNodeInner::PrimaryExp(primary_inner) => match primary_inner {
                PrimaryExpInner::Exp(exp) => self.const_expr_inference(exp),
                PrimaryExpInner::Number(number) => {
                    let AstNodeInner::Number(value) = number.as_inner() else {
                        return Err("Invalid Number node".to_string());
                    };
                    let literal: i32 = value.clone().try_into()?;
                    Ok(literal)
                }
                PrimaryExpInner::LVal(lval) => self.const_expr_inference(lval),
            },
            AstNodeInner::UnaryExp(unary_inner) => match unary_inner {
                UnaryExpInner::PrimaryExp(child) => self.const_expr_inference(child),
                UnaryExpInner::Unary { op, exp } => {
                    let rhs = self.const_expr_inference(exp)?;
                    let result = match op.as_inner() {
                        AstNodeInner::UnaryOp(unary_op) => match unary_op {
                            UnaryOpInner::Plus => rhs,
                            UnaryOpInner::Minus => -rhs,
                            UnaryOpInner::Not => {
                                if rhs == 0 {
                                    1
                                } else {
                                    0
                                }
                            }
                        },
                        _ => unreachable!(),
                    };
                    Ok(result)
                }
                _ => Err("Unimplemented unary expression".to_string()),
            },
            AstNodeInner::MulExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = match op {
                        MulOpInner::Mul => left * rhs,
                        MulOpInner::Div => left / rhs,
                        MulOpInner::Mod => left % rhs,
                    }
                }
                Ok(left)
            }
            AstNodeInner::AddExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = match op {
                        AddOpInner::Plus => left + rhs,
                        AddOpInner::Minus => left - rhs,
                    }
                }
                Ok(left)
            }
            AstNodeInner::RelExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = match op {
                        RelOpInner::Lt => {
                            if left < rhs {
                                1
                            } else {
                                0
                            }
                        }
                        RelOpInner::Gt => {
                            if left > rhs {
                                1
                            } else {
                                0
                            }
                        }
                        RelOpInner::Le => {
                            if left <= rhs {
                                1
                            } else {
                                0
                            }
                        }
                        RelOpInner::Ge => {
                            if left >= rhs {
                                1
                            } else {
                                0
                            }
                        }
                    }
                }
                Ok(left)
            }
            AstNodeInner::EqExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = match op {
                        crate::parser::EqOpInner::Eq => {
                            if left == rhs {
                                1
                            } else {
                                0
                            }
                        }
                        crate::parser::EqOpInner::Neq => {
                            if left != rhs {
                                1
                            } else {
                                0
                            }
                        }
                    }
                }
                Ok(left)
            }
            AstNodeInner::AndExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (_op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = if left != 0 && rhs != 0 { 1 } else { 0 };
                }
                Ok(left)
            }
            AstNodeInner::OrExp { lhs, ops } => {
                let mut left = self.const_expr_inference(lhs)?;
                for (_op, rhs_node) in ops {
                    let rhs = self.const_expr_inference(rhs_node)?;
                    left = if left != 0 || rhs != 0 { 1 } else { 0 };
                }
                Ok(left)
            }
            AstNodeInner::ConstExp(exp) => self.const_expr_inference(exp),

            _ => unreachable!(),
        }
    }

    fn gen_exp(&mut self, node: &AstNode) -> Result<BasicValueEnum<'ctx>, String> {
        match node.as_inner() {
            AstNodeInner::Exp(exp) | AstNodeInner::Cond(exp) => self.gen_exp(exp),
            AstNodeInner::LVal { ident, dimensions } => self.gen_lval(ident, dimensions),
            AstNodeInner::PrimaryExp(primary_inner) => match primary_inner {
                PrimaryExpInner::Exp(exp) => self.gen_exp(exp),
                PrimaryExpInner::Number(number) => self.gen_number(number),
                PrimaryExpInner::LVal(lval) => self.gen_exp(&lval),
            },
            AstNodeInner::Number(_) => self.gen_number(node),
            AstNodeInner::UnaryExp(unary_inner) => match unary_inner {
                UnaryExpInner::PrimaryExp(child) => self.gen_exp(child),
                UnaryExpInner::Unary { op, exp } => {
                    let rhs = self.gen_exp(exp)?;
                    let rhs_int = self.into_int_value_helper(rhs)?;

                    let AstNodeInner::UnaryOp(unary_op) = op.as_inner() else {
                        return Err("Invalid UnaryOp node".to_string());
                    };
                    let result = match unary_op {
                        UnaryOpInner::Plus => rhs_int,
                        UnaryOpInner::Minus => self
                            .builder
                            .build_int_neg(rhs_int, "negtmp")
                            .map_err(|e| e.to_string())?,
                        UnaryOpInner::Not => self
                            .builder
                            .build_not(rhs_int, "nottmp")
                            .map_err(|e| e.to_string())?,
                    };

                    Ok(result.as_basic_value_enum())
                }
                _ => Err("Unimplemented unary expression".to_string()),
            },
            AstNodeInner::MulExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = match op {
                        MulOpInner::Mul => self
                            .builder
                            .build_int_mul(lhs_val, rhs_val, "multmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        MulOpInner::Div => self
                            .builder
                            .build_int_signed_div(lhs_val, rhs_val, "divtmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        MulOpInner::Mod => self
                            .builder
                            .build_int_signed_rem(lhs_val, rhs_val, "modtmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                    }
                }
                Ok(left)
            }
            AstNodeInner::AddExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = match op {
                        AddOpInner::Plus => self
                            .builder
                            .build_int_add(lhs_val, rhs_val, "addtmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        AddOpInner::Minus => self
                            .builder
                            .build_int_sub(lhs_val, rhs_val, "subtmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                    }
                }
                Ok(left)
            }
            AstNodeInner::RelExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = match op {
                        RelOpInner::Lt => self
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::SLT,
                                lhs_val,
                                rhs_val,
                                "lttmp",
                            )
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Gt => self
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::SGT,
                                lhs_val,
                                rhs_val,
                                "gttmp",
                            )
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Le => self
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::SLE,
                                lhs_val,
                                rhs_val,
                                "letmp",
                            )
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Ge => self
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::SGE,
                                lhs_val,
                                rhs_val,
                                "getmp",
                            )
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                    }
                }
                Ok(left)
            }
            AstNodeInner::EqExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = match op {
                        crate::parser::EqOpInner::Eq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::EQ, lhs_val, rhs_val, "eqtmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        crate::parser::EqOpInner::Neq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::NE, lhs_val, rhs_val, "netmp")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                    }
                }
                Ok(left)
            }
            AstNodeInner::AndExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (_op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = self
                        .builder
                        .build_and(lhs_val, rhs_val, "andtmp")
                        .map_err(|e| e.to_string())?
                        .as_basic_value_enum();
                }
                Ok(left)
            }
            AstNodeInner::OrExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (_op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let lhs_val = self.into_int_value_helper(left)?;
                    let rhs_val = self.into_int_value_helper(rhs)?;
                    left = self
                        .builder
                        .build_or(lhs_val, rhs_val, "ortmp")
                        .map_err(|e| e.to_string())?
                        .as_basic_value_enum();
                }
                Ok(left)
            }

            _ => Err("Unimplemented expression type".to_string()),
        }
    }

    fn gen_lval(
        &mut self,
        ident: &String,
        dimensions: &Vec<Box<AstNode>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match self
            .scope_stk
            .peek()
            .unwrap_or_else(|| unreachable!())
            .resolve(&ident)
        {
            Some(var) => {
                return Ok(var.inner.clone());
            }
            None => return Err(format!("Undefined variable: {}", ident)),
        }
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

#[test]
fn codegen_dummy() {
    let context = Context::create();
    let mut codegen = Codegen::new("test", &context);
    codegen.gen_dummy();
    codegen.execute();
    codegen.print_to_stderr();
}

#[test]
fn codegen() {
    let src = std::fs::read_to_string("tests/codegen/test1.in").unwrap_or_default();
    let ast = parse(&src, BuildConfig::default())
        .map_err(|e| println!("{}", e))
        .unwrap_or_else(|_| panic!("Failed to parse source code"));
    let context = Context::create();
    let mut codegen = Codegen::new("module", &context);
    codegen
        .gen_comp_unit(&ast)
        .unwrap_or_else(|e| panic!("Failed to generate LLVM IR: {}", e));
    codegen.print_to_stderr();
}
