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

    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::replace(&mut self.errors, Vec::new())
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

    pub fn check(&mut self, node: &AstNode) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
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
                            dimensions: _,
                            init_val: _,
                        } => {
                            let _ = self
                                .scope_stk
                                .peek_mut()
                                .unwrap_or_else(|| unreachable!())
                                .define(
                                    &ident,
                                    VariableMetadata {
                                        ty: match btype.as_inner() {
                                            AstNodeInner::BType(t) if t == "int" => Type::Int,
                                            _ => {
                                                unreachable!()
                                            }
                                        },
                                    },
                                )
                                .map_err(|e| {
                                    errors.push(e);
                                });
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
                            dimensions: _,
                            init_val: _,
                        } => {
                            let _ = self
                                .scope_stk
                                .peek_mut()
                                .unwrap_or_else(|| unreachable!())
                                .define(
                                    &ident,
                                    VariableMetadata {
                                        ty: match btype.as_inner() {
                                            AstNodeInner::BType(t) if t == "int" => Type::Int,
                                            _ => {
                                                unreachable!()
                                            }
                                        },
                                    },
                                )
                                .map_err(|e| {
                                    errors.push(e);
                                });
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            AstNodeInner::FuncDef {
                // no body
                func_type,
                ident,
                params: _,
                body: _,
            } => {
                let _ = self
                    .scope_stk
                    .peek_mut()
                    .unwrap_or_else(|| unreachable!())
                    .define(
                        &ident,
                        VariableMetadata {
                            ty: Type::Function(symbol_table::FunctionType {
                                ret_ty: Box::new(match func_type.as_inner() {
                                    AstNodeInner::FuncType(t) if t == "int" => Type::Int,
                                    AstNodeInner::FuncType(t) if t == "void" => Type::Void,
                                    _ => {
                                        unreachable!()
                                    }
                                }),
                                params_ty: vec![], // TODO: handle parameters
                            }),
                        },
                    )
                    .map_err(|e| {
                        errors.push(e);
                    });

                // entry parameters scope
                self.entry_scope();
            }
            AstNodeInner::LVal {
                ident,
                dimensions: _,
            } => {
                if let Some(scope) = self.scope_stk.peek() {
                    if scope.resove(ident).is_none() {
                        let e = SemanticError::UndefinedVariable(ident.clone());
                        self.push_error(e.clone().into(), node.line_col().0, Some(e.into()))
                    }
                }
            }
            AstNodeInner::Exp(exp_inner)
            | AstNodeInner::ConstExp(exp_inner)
            | AstNodeInner::Cond(exp_inner) => {
                let _ = self.type_inference(exp_inner).map_err(|e| {
                    errors.push(e);
                });
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn push_error(&mut self, typeid: i32, line: usize, message: Option<String>) {
        self.errors.push(format!(
            "Error type {} at Line {}: {}",
            typeid,
            line,
            message.unwrap_or_default()
        ));
    }
}

#[test]
fn test_semantic_single() {
    let src = std::fs::read_to_string("./tests/semantic/sample1.in").unwrap_or_default();
    // let _ = display_ast(&src);
    let _ = parse(&src, BuildConfig::default()).map_err(|s| println!("{}", s));
}
