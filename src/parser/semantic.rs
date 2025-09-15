use crate::parser::semantic;
use crate::parser::symbol_table::{SymbolTable, VariableMetadata};

pub enum SemanticError {
    UndefinedVariable,
    UndefinedFunction,
    RedefinedVariable,
    RedefinedFunction,
}

#[derive(Debug, Clone)]
pub struct Scope {
    symtb: SymbolTable,
    parent: Option<Box<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent: None,
        }
    }
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>) -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent,
        }
    }

    fn define(
        &mut self,
        name: &String,
        var_meta: VariableMetadata,
    ) -> Result<(), semantic::SemanticError> {
        self.symtb.define(name, var_meta)
    }

    fn resove(&self, name: &String) -> Option<&VariableMetadata> {
        if let Some(var) = self.symtb.resove(name) {
            Some(var)
        } else if let Some(ref parent) = self.parent {
            parent.resove(name)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self {
            stack: vec![Scope::default()],
        }
    }
}

impl ScopeStack {
    fn push(&mut self) {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct SemanticChecker {
    scope_stk: ScopeStack,
}

impl SemanticChecker {}
