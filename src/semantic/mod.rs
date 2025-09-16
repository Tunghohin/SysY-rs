pub mod scope;
pub mod symbol_table;
use std::process::id;

use crate::parser::ast::{self, AstNode, AstNodeInner};
use crate::parser::{BuildConfig, parse};
use crate::semantic::scope::{Scope, ScopeStack};
use crate::semantic::symbol_table::{SymbolTable, Type, VariableMetadata};

pub enum SemanticError {
    UndefinedVariable,
    UndefinedFunction,
    RedefinedVariable,
    RedefinedFunction,
}

#[derive(Debug, Default)]
pub struct SemanticChecker {
    scope_stk: ScopeStack,
    errors: Vec<String>,
}

impl SemanticChecker {
    pub fn entry_scope(&mut self) {
        self.scope_stk.push();
    }

    pub fn exit_scope(&mut self) {
        self.scope_stk.pop();
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn check(&mut self, node: &AstNode) {
        // Implement semantic checks here
        match node.as_inner() {
            AstNodeInner::ConstDecl {
                btype,
                const_defs: defs,
            } => {
                for def in defs {
                    match def.as_inner() {
                        AstNodeInner::ConstDef {
                            ident,
                            dimensions,
                            init_val,
                        } => {
                            if let Some(scope) = self.scope_stk.peek_mut() {
                                let _ = scope
                                    .define(&ident, VariableMetadata { ty: Type::Int })
                                    .map_err(|_| self.push_error(node.line_col().0));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            AstNodeInner::VarDecl { btype, var_defs } => {
                for var_def in var_defs {
                    match var_def.as_inner() {
                        AstNodeInner::VarDef {
                            ident,
                            dimensions,
                            init_val,
                        } => {
                            if let Some(scope) = self.scope_stk.peek_mut() {
                                let _ = scope
                                    .define(&ident, VariableMetadata { ty: Type::Int })
                                    .map_err(|_| self.push_error(node.line_col().0));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            AstNodeInner::FuncDef {
                func_type,
                ident,
                params,
                body,
            } => {
                if let Some(scope) = self.scope_stk.peek_mut() {
                    let _ = scope
                        .define(&ident, VariableMetadata { ty: Type::Int })
                        .map_err(|_| self.push_error(node.line_col().0));
                }
            }
            AstNodeInner::LVal { ident, dimensions } => {
                if let Some(scope) = self.scope_stk.peek() {
                    if scope.resove(ident).is_none() {
                        self.push_error(node.line_col().0);
                    }
                }
            }
            AstNodeInner::UnaryExp(unary_exp_type) => match unary_exp_type {
                ast::UnaryExpType::FuncCall { ident, args } => {
                    if let Some(scope) = self.scope_stk.peek() {
                        if scope.resove(ident).is_none() {
                            self.push_error(node.line_col().0);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn push_error(&mut self, line: usize) {
        self.errors.push(format!("Semantic error at line {}", line));
        println!("{}", format!("Semantic error at line {}", line).to_string());
    }
}

#[test]
fn test_semantic_single() {
    let src = std::fs::read_to_string("./tests/semantic/sample1.in").unwrap_or_default();
    parse(&src, BuildConfig::default()).expect("Failed to parse and build AST");
}
