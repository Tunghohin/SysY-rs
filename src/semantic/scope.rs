use crate::semantic::SemanticError;
use crate::semantic::symbol_table::{SymbolTable, VariableMetadata};

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

    pub fn define(
        &mut self,
        name: &String,
        var_meta: VariableMetadata,
    ) -> Result<(), SemanticError> {
        self.symtb.define(name, var_meta)
    }

    pub fn resove(&self, name: &String) -> Option<&VariableMetadata> {
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
    top: Option<Box<Scope>>,
    len: usize,
}

impl Default for ScopeStack {
    fn default() -> Self {
        let mut default = Self { top: None, len: 0 };
        default.push(); // global scope
        default
    }
}

impl ScopeStack {
    pub fn push(&mut self) {
        self.top = Some(Box::new(Scope::new(self.top.take())));
        self.len += 1;
    }

    pub fn pop(&mut self) {
        self.top.take().map(|scope| {
            self.top = scope.parent;
            self.len -= 1;
        });
    }

    pub fn peek(&self) -> Option<&Scope> {
        self.top.as_deref()
    }

    pub fn peek_mut(&mut self) -> Option<&mut Scope> {
        self.top.as_deref_mut()
    }
}
