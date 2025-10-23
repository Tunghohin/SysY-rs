use crate::parser;
use crate::parser::BuildConfig;
use crate::parser::PrimaryExpInner;
use crate::parser::ast::{
    AddOpInner, AstNode, AstNodeInner, ConstInitValInner, InitValInner, MulOpInner, RelOpInner,
    StmtInner, UnaryExpInner, UnaryOpInner,
};
use crate::parser::parse;

use crate::codegen::scope::ScopeStack;
use crate::codegen::symbol_table::{Symbol, SymbolValue};
use core::panic;
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
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct LoopCtx<'ctx> {
    cond_bb: inkwell::basic_block::BasicBlock<'ctx>,
    after_bb: inkwell::basic_block::BasicBlock<'ctx>,
}

pub struct LLVMIRGenerator<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    scope_stk: ScopeStack<'ctx>,
    loop_stk: Vec<LoopCtx<'ctx>>,
}

impl<'ctx> LLVMIRGenerator<'ctx> {
    pub fn new(module_name: &str, ctx: &'ctx Context) -> Self {
        let module = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        Self {
            ctx,
            module,
            builder,
            scope_stk: ScopeStack::default(),
            loop_stk: Vec::new(),
        }
    }

    pub fn optimize(&mut self) {
        let pass_manager: PassManager<Module<'ctx>> = PassManager::create(());
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_instruction_combining_pass();

        pass_manager.run_on(&self.module);
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

    pub fn execute(&self) -> Result<i32, String> {
        let engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| e.to_string())?;
        self.module.verify().map_err(|e| e.to_string())?;
        unsafe {
            let main = engine
                .get_function::<unsafe extern "C" fn() -> i32>("main")
                .map_err(|e| e.to_string())?;
            Ok(main.call())
        }
    }

    pub fn print_to_stderr(&self) {
        self.module.print_to_stderr();
    }

    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    fn into_int_value_helper(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, String> {
        match val {
            BasicValueEnum::IntValue(iv) => Ok(iv),
            BasicValueEnum::PointerValue(pv) => {
                let loaded = self.builder.build_load(pv, "").map_err(|e| e.to_string())?;
                match loaded {
                    BasicValueEnum::IntValue(iv) => Ok(iv),
                    _ => Err("Expected IntValue after loading from PointerValue".to_string()),
                }
            }
            _ => Err("Expected IntValue".to_string()),
        }
    }

    pub fn gen_ir(&mut self, ast: &AstNode) -> Result<(), String> {
        self.gen_comp_unit(ast)
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
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

        if !matches!(btype.as_inner(), AstNodeInner::BType(t) if t == "int") {
            return Err("Only int type is supported".to_string());
        }

        if !dimensions.is_empty() {
            let AstNodeInner::ConstInitVal(ConstInitValInner::InitList(list_inner)) =
                init_val.as_inner()
            else {
                return Err("Only InitList is supported in ConstInitVal for arrays".to_string());
            };

            let totol_size = dimensions
                .iter()
                .map(|node| self.const_expr_inference(node).unwrap() as u32)
                .product::<u32>();
            let ty = self.ctx.i32_type().array_type(totol_size as u32);
            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
            } else {
                let local_ptr = self
                    .builder
                    .build_alloca(ty, "")
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let AstNodeInner::ConstInitVal(ConstInitValInner::ConstExp(exp_inner)) =
                init_val.as_inner()
            else {
                return Err("Only ConstExp is supported in ConstInitVal".to_string());
            };

            let init_val = self.const_expr_inference(exp_inner)?;
            let ty = self.ctx.i32_type();
            let value = ty.const_int(init_val as u64, false);

            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
                global_val.set_initializer(&value);
                // skip const check for special judge rule
                global_val.set_constant(false);
                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        Symbol::new(
                            SymbolValue::Constant {
                                value: global_val.as_basic_value_enum(),
                                ty: ty.into(),
                            },
                            is_global,
                        ),
                    )?;
            } else {
                let local_ptr = self
                    .builder
                    .build_alloca(ty, "")
                    .map_err(|e| e.to_string())?;
                if value.get_type().get_bit_width() == 1 {
                    let zext_val = self
                        .builder
                        .build_int_z_extend(value, self.ctx.i32_type(), "")
                        .map_err(|e| e.to_string())?;
                    self.builder
                        .build_store(local_ptr, zext_val)
                        .map_err(|e| e.to_string())?;
                } else {
                    self.builder
                        .build_store(local_ptr, value)
                        .map_err(|e| e.to_string())?;
                }

                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        Symbol::new(
                            SymbolValue::Variable {
                                ptr: local_ptr,
                                ty: ty.into(),
                                is_const: true,
                            },
                            false,
                        ),
                    )?;
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

        if !matches!(btype.as_inner(), AstNodeInner::BType(t) if t == "int") {
            return Err("Only int type is supported".to_string());
        }

        if !dimensions.is_empty() {
            let totol_size = dimensions
                .iter()
                .map(|node| self.const_expr_inference(node).unwrap() as u32)
                .product::<u32>();
            let ty = self.ctx.i32_type().array_type(totol_size as u32);
            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
            } else {
                let local_ptr = self
                    .builder
                    .build_alloca(ty, "")
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let ty = self.ctx.i32_type();

            if is_global {
                let global_val = self.module.add_global(ty, None, ident.as_str());
                if init_val.is_some() {
                    let init_val = init_val.as_ref().unwrap();
                    let AstNodeInner::InitVal(InitValInner::ConstExp(exp_inner)) =
                        init_val.as_inner()
                    else {
                        return Err("Only ConstExp is supported in InitVal".to_string());
                    };
                    let init_val = self.const_expr_inference(exp_inner)?;
                    let value = ty.const_int(init_val as u64, false);
                    global_val.set_initializer(&value);
                } else {
                    let init_val = ty.const_int(0, false);
                    global_val.set_initializer(&init_val);
                }

                global_val.set_constant(false);
                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        Symbol::new(
                            SymbolValue::Variable {
                                ptr: global_val.as_pointer_value(),
                                ty: ty.into(),
                                is_const: false,
                            },
                            true,
                        ),
                    )?;
            } else {
                let local_ptr = self
                    .builder
                    .build_alloca(ty, "")
                    .map_err(|e| e.to_string())?;
                if init_val.is_some() {
                    let init_val = init_val.as_ref().unwrap();
                    let AstNodeInner::InitVal(InitValInner::ConstExp(exp_inner)) =
                        init_val.as_inner()
                    else {
                        return Err("Only ConstExp is supported in InitVal".to_string());
                    };
                    let value = self.gen_exp(exp_inner)?;

                    if value.into_int_value().get_type().get_bit_width() == 1 {
                        let zext_val = self
                            .builder
                            .build_int_z_extend(value.into_int_value(), self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                        self.builder
                            .build_store(local_ptr, zext_val)
                            .map_err(|e| e.to_string())?;
                    } else {
                        self.builder
                            .build_store(local_ptr, value)
                            .map_err(|e| e.to_string())?;
                    }
                }

                self.scope_stk
                    .peek_mut()
                    .ok_or("Scope stack is empty".to_string())?
                    .define(
                        ident,
                        Symbol::new(
                            SymbolValue::Variable {
                                ptr: local_ptr,
                                ty: ty.into(),
                                is_const: false,
                            },
                            false,
                        ),
                    )?;
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

        let (param_types, param_names) = if let Some(params_node) = params {
            if let AstNodeInner::FuncFParams(params_list) = params_node.as_inner() {
                let mut types = Vec::new();
                let mut names = Vec::new();

                for param in params_list {
                    if let AstNodeInner::FuncFParam {
                        btype,
                        ident,
                        is_array,
                        dimensions,
                    } = param.as_inner()
                    {
                        types.push(self.ctx.i32_type().into());
                        names.push(ident.clone());
                    } else {
                        return Err("Invalid function parameter".to_string());
                    }
                }

                (types, names)
            } else {
                return Err("Invalid function parameters".to_string());
            }
        } else {
            (vec![], vec![])
        };

        let fn_type = if let Some(ret_ty) = ret_type {
            self.ctx.i32_type().fn_type(&param_types, false)
        } else {
            self.ctx.void_type().fn_type(&param_types, false)
        };

        let function = self.module.add_function(ident.as_str(), fn_type, None);
        self.scope_stk
            .peek_mut()
            .ok_or("Scope stack is empty".to_string())?
            .define(ident, Symbol::new(SymbolValue::Function(function), true))?;

        // enter parameters scope
        self.scope_stk.push();

        let entry_bb = self
            .ctx
            .append_basic_block(function, format!("{}_entry", ident).as_str());
        self.builder.position_at_end(entry_bb);

        for (i, param_name) in param_names.iter().enumerate() {
            let param_value = function
                .get_nth_param(i as u32)
                .ok_or(format!("Failed to get parameter {}", i))?;

            let param_ptr = self
                .builder
                .build_alloca(self.ctx.i32_type(), "")
                .map_err(|e| format!("Failed to allocate parameter: {:?}", e))?;

            self.builder
                .build_store(param_ptr, param_value)
                .map_err(|e| format!("Failed to store parameter value: {:?}", e))?;

            self.scope_stk
                .peek_mut()
                .ok_or("Parameter scope is empty".to_string())?
                .define(
                    param_name,
                    Symbol::new(
                        SymbolValue::Variable {
                            ptr: param_ptr,
                            ty: self.ctx.i32_type().into(),
                            is_const: false,
                        },
                        false,
                    ),
                )?;
        }

        // enter function body scope
        self.scope_stk.push();

        let AstNodeInner::Block(items) = body.as_inner() else {
            return Err("Invalid function body".to_string());
        };

        for item in items {
            let AstNodeInner::BlockItem(block_item) = item.as_inner() else {
                return Err("Unexpected node in function body".to_string());
            };
            match &*block_item.as_inner() {
                AstNodeInner::Stmt(stmt_inner) => self.gen_stmt(stmt_inner)?,
                AstNodeInner::Decl(decl) => self.gen_decl(&block_item, false)?,
                _ => return Err("Unexpected node in function body".to_string()),
            }
        }

        // exit function body scope
        self.scope_stk.pop();

        // void return if no return statement
        if ret_type.is_none() {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder.build_return(None).map_err(|e| e.to_string())?;
            }
        } else {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder
                    .build_return(Some(&self.ctx.i32_type().const_int(0, false)))
                    .map_err(|e| e.to_string())?;
            }
        }

        // exit parameters scope
        self.scope_stk.pop();

        Ok(())
    }

    fn gen_stmt(&mut self, stmt_inner: &StmtInner) -> Result<(), String> {
        match stmt_inner {
            StmtInner::Assign { lval, exp } => {
                let exp_val = self.gen_exp(exp)?;
                let AstNodeInner::LVal { ident, dimensions } = &(*lval).as_inner() else {
                    return Err("Invalid LVal in assignment".to_string());
                };
                let lval_ptr = match self.scope_stk.peek().unwrap().resolve(ident) {
                    Some(var) => match var.value {
                        SymbolValue::Variable { ptr, .. } => ptr,
                        SymbolValue::Constant { value, .. } => {
                            // skip const check for special judge rule
                            value.into_pointer_value()
                        }
                        _ => return Err("Unsupported symbol type".to_string()),
                    },
                    None => return Err(format!("Undefined variable: {}", ident)),
                };

                if exp_val.into_int_value().get_type().get_bit_width() == 1 {
                    let zext_val = self
                        .builder
                        .build_int_z_extend(exp_val.into_int_value(), self.ctx.i32_type(), "")
                        .map_err(|e| e.to_string())?;
                    self.builder
                        .build_store(lval_ptr, zext_val)
                        .map_err(|e| e.to_string())?;
                } else {
                    self.builder
                        .build_store(lval_ptr, exp_val)
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            }
            StmtInner::Exp(opt_exp) => {
                let Some(exp) = opt_exp else {
                    return Ok(());
                };
                let val = self.gen_exp(exp)?;

                Ok(())
            }
            StmtInner::Block(block_inner) => self.gen_block(block_inner),
            StmtInner::Return(opt_exp) => {
                let Some(exp) = opt_exp else {
                    self.builder.build_return(None).map_err(|e| e.to_string())?;
                    return Ok(());
                };
                let ret_val = self.gen_exp(exp)?;
                let ret_val = if ret_val.is_pointer_value() {
                    self.builder
                        .build_load(ret_val.into_pointer_value(), "")
                        .map_err(|e| e.to_string())?
                } else {
                    ret_val
                };
                self.builder
                    .build_return(Some(&ret_val))
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            StmtInner::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                let parent_fn = self
                    .builder
                    .get_insert_block()
                    .and_then(|b| b.get_parent())
                    .ok_or("Failed to get parent function".to_string())?;

                let then_bb = self.ctx.append_basic_block(parent_fn, "if_true");
                let else_bb = if else_stmt.is_some() {
                    Some(self.ctx.append_basic_block(parent_fn, "if_false"))
                } else {
                    None
                };

                let cond_val = self.gen_exp(cond)?;
                let cond_val = cond_val.into_int_value();
                let merge_bb = self.ctx.append_basic_block(parent_fn, "next");

                if let Some(else_bb) = else_bb {
                    self.builder
                        .build_conditional_branch(cond_val, then_bb, else_bb)
                        .map_err(|e| e.to_string())?;
                } else {
                    self.builder
                        .build_conditional_branch(cond_val, then_bb, merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(then_bb);
                let AstNodeInner::Stmt(then_inner) = then_stmt.as_inner() else {
                    return Err("Invalid then statement".to_string());
                };
                self.gen_stmt(then_inner)?;
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder
                        .build_unconditional_branch(merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                if let Some(else_stmt) = else_stmt {
                    self.builder.position_at_end(else_bb.unwrap());
                    let AstNodeInner::Stmt(else_inner) = else_stmt.as_inner() else {
                        return Err("Invalid else statement".to_string());
                    };
                    self.gen_stmt(else_inner)?;
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_none()
                    {
                        self.builder
                            .build_unconditional_branch(merge_bb)
                            .map_err(|e| e.to_string())?;
                    }
                }

                self.builder.position_at_end(merge_bb);

                Ok(())
            }

            StmtInner::While { cond, stmt } => {
                let parent_fn = self
                    .builder
                    .get_insert_block()
                    .and_then(|b| b.get_parent())
                    .ok_or("Failed to get parent function".to_string())?;

                let cond_bb = self.ctx.append_basic_block(parent_fn, "while_cond");
                let body_bb = self.ctx.append_basic_block(parent_fn, "while_body");
                let after_bb = self.ctx.append_basic_block(parent_fn, "while_after");

                self.builder
                    .build_unconditional_branch(cond_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(cond_bb);
                let cond_val = self.gen_exp(cond)?;
                let cond_val = cond_val.into_int_value();
                self.builder
                    .build_conditional_branch(cond_val, body_bb, after_bb)
                    .map_err(|e| e.to_string())?;

                self.loop_stk.push(LoopCtx { cond_bb, after_bb });

                self.builder.position_at_end(body_bb);
                let AstNodeInner::Stmt(body_inner) = stmt.as_inner() else {
                    return Err("Invalid while body statement".to_string());
                };
                self.gen_stmt(body_inner)?;

                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder
                        .build_unconditional_branch(cond_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.loop_stk.pop();

                self.builder.position_at_end(after_bb);

                Ok(())
            }
            StmtInner::Break => {
                let loop_ctx = self
                    .loop_stk
                    .last()
                    .ok_or("Break not in a loop".to_string())?;
                self.builder
                    .build_unconditional_branch(loop_ctx.after_bb)
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            StmtInner::Continue => {
                let loop_ctx = self
                    .loop_stk
                    .last()
                    .ok_or("Continue not in a loop".to_string())?;
                self.builder
                    .build_unconditional_branch(loop_ctx.cond_bb)
                    .map_err(|e| e.to_string())?;
                Ok(())
            }

            _ => unimplemented!(),
        }
    }

    fn const_expr_inference(&mut self, node: &AstNode) -> Result<i32, String> {
        match node.as_inner() {
            AstNodeInner::Exp(exp) | AstNodeInner::Cond(exp) => self.const_expr_inference(exp),
            AstNodeInner::LVal { ident, dimensions } => {
                let Some(var) = self.scope_stk.peek().unwrap().resolve(&ident) else {
                    return Err(format!("Undefined variable: {}", ident));
                };
                if !var.is_constant() {
                    return Err(format!("Variable {} is not constant", ident));
                }
                match var.value {
                    SymbolValue::Constant { value, ty } => {
                        let BasicValueEnum::PointerValue(pv) = value else {
                            return Err("Expected PointerValue for constant".to_string());
                        };
                        let Some(name) = pv.get_name().to_str().ok() else {
                            return Err("Invalid variable name".to_string());
                        };

                        let Some(iv) = self.module.get_global(name).unwrap().get_initializer()
                        else {
                            return Err("Failed to get initializer for global variable".to_string());
                        };
                        Ok(iv.into_int_value().get_zero_extended_constant().unwrap() as i32)
                    }
                    _ => Err("Unsupported symbol type".to_string()),
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
                        _ => Err("Invalid UnaryOp node".to_string())?,
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

            _ => Err("Unsupported expression type".to_string()),
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
                            .build_int_neg(rhs_int, "")
                            .map_err(|e| e.to_string())?,
                        UnaryOpInner::Not => self
                            .builder
                            .build_not(rhs_int, "")
                            .map_err(|e| e.to_string())?,
                    };

                    Ok(result.as_basic_value_enum())
                }
                UnaryExpInner::FuncCall { ident, args } => {
                    let func = {
                        let Some(Symbol {
                            value: SymbolValue::Function(func),
                            ..
                        }) = self.scope_stk.peek().unwrap().resolve(ident)
                        else {
                            return Err(format!("Undefined function: {}", ident));
                        };
                        *func
                    };

                    let arg_values = if let Some(arg_nodes) = args {
                        let mut values = Vec::new();
                        for arg_node in arg_nodes {
                            let val = self.gen_exp(arg_node)?;
                            values.push(val.into());
                        }
                        values
                    } else {
                        vec![]
                    };

                    let call_site = self
                        .builder
                        .build_call(func, &arg_values, "")
                        .map_err(|e| e.to_string())?;
                    match func.get_type().get_return_type() {
                        Some(_ret_ty) => Ok(call_site.try_as_basic_value().left().unwrap()),
                        None => Ok(self.ctx.i32_type().const_int(0, false).into()), // void function returns 0
                    }
                }
            },
            AstNodeInner::MulExp { lhs, ops } => {
                let mut left = self.gen_exp(lhs)?;
                for (op, rhs_node) in ops {
                    let rhs = self.gen_exp(rhs_node)?;
                    let mut lhs_val = self.into_int_value_helper(left)?;
                    let mut rhs_val = self.into_int_value_helper(rhs)?;

                    if lhs_val.get_type().get_bit_width() == 1 {
                        lhs_val = self
                            .builder
                            .build_int_z_extend(lhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }
                    if rhs_val.get_type().get_bit_width() == 1 {
                        rhs_val = self
                            .builder
                            .build_int_z_extend(rhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }

                    left = match op {
                        MulOpInner::Mul => self
                            .builder
                            .build_int_mul(lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        MulOpInner::Div => self
                            .builder
                            .build_int_signed_div(lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        MulOpInner::Mod => self
                            .builder
                            .build_int_signed_rem(lhs_val, rhs_val, "")
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
                    let mut lhs_val = self.into_int_value_helper(left)?;
                    let mut rhs_val = self.into_int_value_helper(rhs)?;

                    if lhs_val.get_type().get_bit_width() == 1 {
                        lhs_val = self
                            .builder
                            .build_int_z_extend(lhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }
                    if rhs_val.get_type().get_bit_width() == 1 {
                        rhs_val = self
                            .builder
                            .build_int_z_extend(rhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }

                    left = match op {
                        AddOpInner::Plus => self
                            .builder
                            .build_int_add(lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        AddOpInner::Minus => self
                            .builder
                            .build_int_sub(lhs_val, rhs_val, "")
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
                    let mut lhs_val = self.into_int_value_helper(left)?;
                    let mut rhs_val = self.into_int_value_helper(rhs)?;

                    if lhs_val.get_type().get_bit_width() == 1 {
                        lhs_val = self
                            .builder
                            .build_int_z_extend(lhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }
                    if rhs_val.get_type().get_bit_width() == 1 {
                        rhs_val = self
                            .builder
                            .build_int_z_extend(rhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }

                    left = match op {
                        RelOpInner::Lt => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SLT, lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Gt => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGT, lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Le => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SLE, lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        RelOpInner::Ge => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGE, lhs_val, rhs_val, "")
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
                    let mut lhs_val = self.into_int_value_helper(left)?;
                    let mut rhs_val = self.into_int_value_helper(rhs)?;

                    if lhs_val.get_type().get_bit_width() == 1 {
                        lhs_val = self
                            .builder
                            .build_int_z_extend(lhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }
                    if rhs_val.get_type().get_bit_width() == 1 {
                        rhs_val = self
                            .builder
                            .build_int_z_extend(rhs_val, self.ctx.i32_type(), "")
                            .map_err(|e| e.to_string())?;
                    }

                    left = match op {
                        crate::parser::EqOpInner::Eq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::EQ, lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                        crate::parser::EqOpInner::Neq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::NE, lhs_val, rhs_val, "")
                            .map_err(|e| e.to_string())?
                            .as_basic_value_enum(),
                    }
                }
                Ok(left)
            }
            AstNodeInner::AndExp { lhs, ops } => {
                if ops.is_empty() {
                    return self.gen_exp(lhs);
                }

                let parent_fn = self
                    .builder
                    .get_insert_block()
                    .and_then(|b| b.get_parent())
                    .ok_or("Failed to get parent function".to_string())?;

                let merge_bb = self.ctx.append_basic_block(parent_fn, "and_merge");

                let mut phi_incoming = Vec::new();

                let mut current_val = self.gen_exp(lhs)?;
                let mut current_int = self.into_int_value_helper(current_val)?;

                if current_int.get_type().get_bit_width() == 1 {
                    current_int = self
                        .builder
                        .build_int_z_extend(current_int, self.ctx.i32_type(), "")
                        .map_err(|e| e.to_string())?;
                }

                for (i, (_op, rhs_node)) in ops.iter().enumerate() {
                    let current_cond = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::NE,
                            current_int,
                            self.ctx.i32_type().const_int(0, false),
                            "and_cond",
                        )
                        .map_err(|e| e.to_string())?;

                    let current_block = self.builder.get_insert_block().unwrap();

                    if i == ops.len() - 1 {
                        let rhs_bb = self.ctx.append_basic_block(parent_fn, "and_rhs");

                        self.builder
                            .build_conditional_branch(current_cond, rhs_bb, merge_bb)
                            .map_err(|e| e.to_string())?;

                        phi_incoming.push((current_int, current_block));

                        self.builder.position_at_end(rhs_bb);
                        let rhs_val = self.gen_exp(rhs_node)?;
                        let mut rhs_int = self.into_int_value_helper(rhs_val)?;

                        if rhs_int.get_type().get_bit_width() == 1 {
                            rhs_int = self
                                .builder
                                .build_int_z_extend(rhs_int, self.ctx.i32_type(), "")
                                .map_err(|e| e.to_string())?;
                        }

                        let rhs_block = self.builder.get_insert_block().unwrap();
                        phi_incoming.push((rhs_int, rhs_block));

                        self.builder
                            .build_unconditional_branch(merge_bb)
                            .map_err(|e| e.to_string())?;
                    } else {
                        let continue_bb = self.ctx.append_basic_block(parent_fn, "and_continue");

                        self.builder
                            .build_conditional_branch(current_cond, continue_bb, merge_bb)
                            .map_err(|e| e.to_string())?;

                        phi_incoming.push((current_int, current_block));

                        self.builder.position_at_end(continue_bb);
                        let next_val = self.gen_exp(rhs_node)?;
                        current_int = self.into_int_value_helper(next_val)?;

                        if current_int.get_type().get_bit_width() == 1 {
                            current_int = self
                                .builder
                                .build_int_z_extend(current_int, self.ctx.i32_type(), "")
                                .map_err(|e| e.to_string())?;
                        }

                        current_val = current_int.as_basic_value_enum();
                    }
                }

                self.builder.position_at_end(merge_bb);
                let phi = self
                    .builder
                    .build_phi(self.ctx.i32_type(), "and_result")
                    .map_err(|e| e.to_string())?;

                for (value, block) in phi_incoming {
                    phi.add_incoming(&[(&value, block)]);
                }

                let result = phi.as_basic_value().into_int_value();
                let bool_result = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        result,
                        self.ctx.i32_type().const_int(0, false),
                        "and_bool",
                    )
                    .map_err(|e| e.to_string())?;

                Ok(bool_result.as_basic_value_enum())
            }

            AstNodeInner::OrExp { lhs, ops } => {
                if ops.is_empty() {
                    return self.gen_exp(lhs);
                }

                let parent_fn = self
                    .builder
                    .get_insert_block()
                    .and_then(|b| b.get_parent())
                    .ok_or("Failed to get parent function".to_string())?;

                let merge_bb = self.ctx.append_basic_block(parent_fn, "or_merge");

                let mut phi_incoming = Vec::new();

                let mut current_val = self.gen_exp(lhs)?;
                let mut current_int = self.into_int_value_helper(current_val)?;

                if current_int.get_type().get_bit_width() == 1 {
                    current_int = self
                        .builder
                        .build_int_z_extend(current_int, self.ctx.i32_type(), "")
                        .map_err(|e| e.to_string())?;
                }

                for (i, (_op, rhs_node)) in ops.iter().enumerate() {
                    let current_cond = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::EQ,
                            current_int,
                            self.ctx.i32_type().const_int(0, false),
                            "or_cond",
                        )
                        .map_err(|e| e.to_string())?;

                    let current_block = self.builder.get_insert_block().unwrap();

                    if i == ops.len() - 1 {
                        let rhs_bb = self.ctx.append_basic_block(parent_fn, "or_rhs");

                        self.builder
                            .build_conditional_branch(current_cond, rhs_bb, merge_bb)
                            .map_err(|e| e.to_string())?;

                        phi_incoming.push((current_int, current_block));

                        self.builder.position_at_end(rhs_bb);
                        let rhs_val = self.gen_exp(rhs_node)?;
                        let mut rhs_int = self.into_int_value_helper(rhs_val)?;

                        if rhs_int.get_type().get_bit_width() == 1 {
                            rhs_int = self
                                .builder
                                .build_int_z_extend(rhs_int, self.ctx.i32_type(), "")
                                .map_err(|e| e.to_string())?;
                        }

                        let rhs_block = self.builder.get_insert_block().unwrap();
                        phi_incoming.push((rhs_int, rhs_block));

                        self.builder
                            .build_unconditional_branch(merge_bb)
                            .map_err(|e| e.to_string())?;
                    } else {
                        let continue_bb = self.ctx.append_basic_block(parent_fn, "or_continue");

                        self.builder
                            .build_conditional_branch(current_cond, continue_bb, merge_bb)
                            .map_err(|e| e.to_string())?;

                        phi_incoming.push((current_int, current_block));

                        self.builder.position_at_end(continue_bb);
                        let next_val = self.gen_exp(rhs_node)?;
                        current_int = self.into_int_value_helper(next_val)?;

                        if current_int.get_type().get_bit_width() == 1 {
                            current_int = self
                                .builder
                                .build_int_z_extend(current_int, self.ctx.i32_type(), "")
                                .map_err(|e| e.to_string())?;
                        }

                        current_val = current_int.as_basic_value_enum();
                    }
                }

                self.builder.position_at_end(merge_bb);
                let phi = self
                    .builder
                    .build_phi(self.ctx.i32_type(), "or_result")
                    .map_err(|e| e.to_string())?;

                for (value, block) in phi_incoming {
                    phi.add_incoming(&[(&value, block)]);
                }

                let result = phi.as_basic_value().into_int_value();
                let bool_result = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        result,
                        self.ctx.i32_type().const_int(0, false),
                        "or_bool",
                    )
                    .map_err(|e| e.to_string())?;

                Ok(bool_result.as_basic_value_enum())
            }

            _ => Err("Unimplemented expression type".to_string()),
        }
    }

    fn gen_block(&mut self, node: &AstNode) -> Result<(), String> {
        let AstNodeInner::Block(stmts) = node.as_inner() else {
            return Err("Invalid Block node".to_string());
        };

        self.scope_stk.push();

        for stmt in stmts {
            let AstNodeInner::BlockItem(block_item) = stmt.as_inner() else {
                return Err("Unexpected node in function body".to_string());
            };
            match &*block_item.as_inner() {
                AstNodeInner::Stmt(stmt_inner) => self.gen_stmt(stmt_inner)?,
                AstNodeInner::Decl(decl) => self.gen_decl(&block_item, false)?,
                _ => return Err("Unexpected node in function body".to_string()),
            }
        }

        self.scope_stk.pop();

        Ok(())
    }

    fn gen_lval(
        &mut self,
        ident: &String,
        dimensions: &Vec<Box<AstNode>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match self.scope_stk.peek().unwrap().resolve(&ident) {
            Some(var) => match var.value {
                SymbolValue::Variable { ptr, ty, .. } => {
                    let loaded = self
                        .builder
                        .build_load(ptr, "")
                        .map_err(|e| e.to_string())?;
                    Ok(loaded)
                }
                SymbolValue::Constant { value, ty } => Ok(value),
                _ => Err("Unsupported symbol type".to_string()),
            },
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
