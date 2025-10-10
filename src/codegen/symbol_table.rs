use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolValue<'ctx> {
    Variable {
        ptr: PointerValue<'ctx>,
        ty: BasicTypeEnum<'ctx>,
        is_const: bool,
    },
    Function(FunctionValue<'ctx>),
    Constant {
        value: BasicValueEnum<'ctx>,
        ty: BasicTypeEnum<'ctx>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol<'ctx> {
    pub value: SymbolValue<'ctx>,
    pub is_global: bool,
}

impl<'ctx> Symbol<'ctx> {
    pub fn new(value: SymbolValue<'ctx>, is_global: bool) -> Self {
        Self { value, is_global }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self.value, SymbolValue::Variable { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self.value, SymbolValue::Function(_))
    }

    pub fn is_constant(&self) -> bool {
        matches!(self.value, SymbolValue::Constant { .. })
    }

    pub fn get_type(&self) -> Option<BasicTypeEnum<'ctx>> {
        match &self.value {
            SymbolValue::Variable { ty, .. } => Some(*ty),
            SymbolValue::Constant { ty, .. } => Some(*ty),
            SymbolValue::Function(func) => Some(func.get_type().get_return_type()?.into()),
        }
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'ctx> {
    meta: HashMap<String, Symbol<'ctx>>,
}

impl<'ctx> SymbolTable<'ctx> {
    pub fn define(&mut self, name: &String, var_meta: Symbol<'ctx>) -> Result<(), String> {
        if self.meta.contains_key(name) {
            Err(format!("Redefined variable: {}", name))
        } else {
            self.meta.insert(name.clone(), var_meta);
            Ok(())
        }
    }

    pub fn resolve(&self, name: &String) -> Option<&Symbol<'ctx>> {
        if self.meta.contains_key(name) {
            self.meta.get(name)
        } else {
            None
        }
    }
}
