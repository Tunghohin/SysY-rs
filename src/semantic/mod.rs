pub mod scope;
pub mod symbol_table;

use crate::parser::ast::{self, AstNode, AstNodeInner};
use crate::parser::{BuildConfig, display_ast, parse};
use crate::semantic::scope::{Scope, ScopeStack};
use crate::semantic::symbol_table::{SymbolTable, Type, VariableMetadata};
use std::io::Write;
use std::process::id;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    RedefinedVariable(String),
    RedefinedFunction(String),
    TypeMismatchedForAssignment,
    TypeMismatchedForOperands,
    TypeMismatchedForReturn,
    FunctionArgTypeMismatched(String),
    NotAnArray(String),
    NotAFunction(String),
    NotALValue(String),
}

impl Into<i32> for SemanticError {
    fn into(self) -> i32 {
        match self {
            SemanticError::UndefinedVariable(_) => 1,
            SemanticError::UndefinedFunction(_) => 2,
            SemanticError::RedefinedVariable(_) => 3,
            SemanticError::RedefinedFunction(_) => 4,
            SemanticError::TypeMismatchedForAssignment => 5,
            SemanticError::TypeMismatchedForOperands => 6,
            SemanticError::TypeMismatchedForReturn => 7,
            SemanticError::FunctionArgTypeMismatched(_) => 8,
            SemanticError::NotAnArray(_) => 9,
            SemanticError::NotAFunction(_) => 10,
            SemanticError::NotALValue(_) => 11,
        }
    }
}

impl Into<String> for SemanticError {
    fn into(self) -> String {
        match self {
            SemanticError::UndefinedVariable(name) => format!("Undefined variable '{}'.", name),
            SemanticError::UndefinedFunction(name) => format!("Undefined function '{}'.", name),
            SemanticError::RedefinedVariable(name) => format!("Redefined variable '{}'.", name),
            SemanticError::RedefinedFunction(name) => format!("Redefined function '{}'.", name),
            SemanticError::TypeMismatchedForAssignment => {
                "Type mismatched for assignment.".to_string()
            }
            SemanticError::TypeMismatchedForOperands => "Type mismatched for operands.".to_string(),
            SemanticError::TypeMismatchedForReturn => "Type mismatched for return.".to_string(),
            SemanticError::FunctionArgTypeMismatched(name) => {
                format!("Function '{}' argument type mismatched.", name)
            }
            SemanticError::NotAnArray(name) => format!("'{}' is not an array.", name),
            SemanticError::NotAFunction(name) => format!("'{}' is not a function.", name),
            SemanticError::NotALValue(name) => format!("'{}' is not a lvalue.", name),
        }
    }
}

#[derive(Debug, Default)]
pub struct SemanticChecker {
    scope_stk: ScopeStack,
}

impl SemanticChecker {
    pub fn entry_scope(&mut self) {
        self.scope_stk.push();
    }

    pub fn exit_scope(&mut self) {
        self.scope_stk.pop();
    }

    // maybe using cache to store inferred types to optimize performance
    pub fn type_inference(&self, node: &AstNode) -> Result<Type, SemanticError> {
        match node.as_inner() {
            AstNodeInner::Exp(exp_inner) => self.type_inference(exp_inner),
            AstNodeInner::Cond(cond_inner) => self.type_inference(cond_inner),
            AstNodeInner::PrimaryExp(primary_exp_inner) => match primary_exp_inner {
                ast::PrimaryExpInner::Number(_) => Ok(Type::Int),
                ast::PrimaryExpInner::LVal(lval) => match lval.as_inner() {
                    AstNodeInner::LVal {
                        ident,
                        dimensions: _,
                    } => self
                        .scope_stk
                        .peek()
                        .unwrap_or_else(|| unreachable!())
                        .resove(&ident)
                        .map_or_else(
                            || Err(SemanticError::UndefinedVariable(ident.clone())),
                            |meta| Ok(meta.ty.clone()),
                        ),
                    _ => unreachable!(),
                },
                ast::PrimaryExpInner::Exp(exp) => self.type_inference(exp),
            },
            AstNodeInner::UnaryExp(unary_exp_inner) => match unary_exp_inner {
                ast::UnaryExpInner::FuncCall { ident, args: _ } => self
                    .scope_stk
                    .peek()
                    .unwrap_or_else(|| unreachable!())
                    .resove(&ident)
                    .map_or_else(
                        || Err(SemanticError::UndefinedFunction(ident.clone())),
                        |meta| Ok(meta.ty.clone()),
                    ),
                ast::UnaryExpInner::Unary { op: _, exp } => match self.type_inference(exp)? {
                    Type::Int => Ok(Type::Int),
                    _ => Err(SemanticError::TypeMismatchedForOperands),
                },
                ast::UnaryExpInner::PrimaryExp(exp) => self.type_inference(exp),
            },
            AstNodeInner::MulExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::AddExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::RelExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::EqExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::AndExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::OrExp { lhs, ops } => {
                let lhs_type = self.type_inference(lhs)?;
                for (_op, rhs) in ops {
                    let rhs_type = self.type_inference(rhs)?;
                    if lhs_type != Type::Int || rhs_type != Type::Int {
                        return Err(SemanticError::TypeMismatchedForOperands);
                    }
                }
                Ok(lhs_type)
            }
            AstNodeInner::ConstExp(exp_inner) => self.type_inference(exp_inner),
            _ => unreachable!(),
        }
    }

    pub fn check(&mut self, node: &AstNode) -> Result<(), SemanticError> {
        match node.as_inner() {
            AstNodeInner::Stmt(stmt_inner) => match &**stmt_inner {
                ast::StmtInner::Assign { lval, exp: _ } => {
                    let ident = match lval.as_inner() {
                        AstNodeInner::LVal {
                            ident,
                            dimensions: _,
                        } => ident,
                        _ => unreachable!(),
                    };
                    self.scope_stk
                        .peek()
                        .unwrap_or_else(|| unreachable!())
                        .resove(ident)
                        .map_or_else(
                            || Err(SemanticError::UndefinedVariable(ident.clone())),
                            |_| Ok(()),
                        )
                }
                ast::StmtInner::Block(_block) => Ok(()),
                ast::StmtInner::If {
                    cond,
                    then_stmt: _,
                    else_stmt: _,
                } => self
                    .type_inference(cond)
                    .map_or_else(|e| Err(e), |_| Ok(())),
                ast::StmtInner::While { cond, stmt: _ } => self
                    .type_inference(cond)
                    .map_or_else(|e| Err(e), |_| Ok(())),
                ast::StmtInner::Break | ast::StmtInner::Continue => Ok(()),
                ast::StmtInner::Return(exp) | ast::StmtInner::Exp(exp) => {
                    if let Some(exp) = exp {
                        self.type_inference(exp).map_or_else(|e| Err(e), |_| Ok(()))
                    } else {
                        Ok(())
                    }
                }
            },
            AstNodeInner::ConstDecl { btype, const_defs } => {
                let ty = match btype.as_inner() {
                    AstNodeInner::BType(ty) => match ty.as_str() {
                        "int" => Type::Int,
                        "void" => Type::Void,
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                for def in const_defs {
                    match def.as_inner() {
                        AstNodeInner::VarDef {
                            ident,
                            dimensions,
                            init_val: _,
                        } => {
                            let var_type = if dimensions.is_empty() {
                                ty.clone()
                            } else {
                                Type::Array(symbol_table::ArrayType {
                                    ty: Box::new(ty.clone()),
                                    num_elements: None,
                                })
                            };
                            let var_meta = VariableMetadata { ty: var_type };
                            let res = self
                                .scope_stk
                                .peek_mut()
                                .unwrap_or_else(|| unreachable!())
                                .define(ident, var_meta);
                            if res.is_err() {
                                return Err(SemanticError::RedefinedVariable(ident.clone()));
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                Ok(())
            }
            AstNodeInner::VarDecl { btype, var_defs } => {
                let ty = match btype.as_inner() {
                    AstNodeInner::BType(ty) => match ty.as_str() {
                        "int" => Type::Int,
                        "void" => Type::Void,
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                for def in var_defs {
                    match def.as_inner() {
                        AstNodeInner::VarDef {
                            ident,
                            dimensions,
                            init_val: _,
                        } => {
                            let var_type = if dimensions.is_empty() {
                                ty.clone()
                            } else {
                                Type::Array(symbol_table::ArrayType {
                                    ty: Box::new(ty.clone()),
                                    num_elements: None,
                                })
                            };
                            let var_meta = VariableMetadata { ty: var_type };
                            let res = self
                                .scope_stk
                                .peek_mut()
                                .unwrap_or_else(|| unreachable!())
                                .define(ident, var_meta);
                            if res.is_err() {
                                return Err(SemanticError::RedefinedVariable(ident.clone()));
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_semantic_single() {
    let src = std::fs::read_to_string("./tests/semantic/sample1.in").unwrap_or_default();
    // let _ = display_ast(&src);
    let _ = parse(&src, BuildConfig::default()).map_err(|s| println!("{}", s));
}
