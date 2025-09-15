use crate::parser::semantic;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Array(ArrayType),
    Function(FunctionType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayType {
    pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub ret_ty: Box<Type>,
    pub params_ty: Vec<Box<Type>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableMetadata {
    pub ty: Type,
    pub addr: usize,
    pub declpos: (usize, usize),
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    meta: HashMap<String, VariableMetadata>,
}

impl SymbolTable {
    pub fn define(
        &mut self,
        name: &String,
        var_meta: VariableMetadata,
    ) -> Result<(), semantic::SemanticError> {
        if self.meta.contains_key(name) {
            match var_meta.ty {
                Type::Int | Type::Array(_) => Err(semantic::SemanticError::RedefinedVariable),
                Type::Function(_) => Err(semantic::SemanticError::RedefinedFunction),
            }
        } else {
            self.meta.insert(name.clone(), var_meta);
            Ok(())
        }
    }

    pub fn resove(&self, name: &String) -> Option<&VariableMetadata> {
        if self.meta.contains_key(name) {
            self.meta.get(name)
        } else {
            None
        }
    }
}
