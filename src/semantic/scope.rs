use crate::semantic::SemanticError;
use crate::semantic::symbol_table::{SymbolTable, Type, VariableMetadata};

#[derive(Debug, Clone)]
pub struct Scope {
    symtb: SymbolTable,
    parent: Option<Box<Scope>>,
    ret_ty: Option<Box<Type>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent: None,
            ret_ty: None,
        }
    }
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>, ret_ty: Option<Box<Type>>) -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent,
            ret_ty,
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
        default.push(None); // global scope
        default
    }
}

impl ScopeStack {
    pub fn push(&mut self, ret_ty: Option<Box<Type>>) {
        self.top = Some(Box::new(Scope::new(self.top.take(), ret_ty)));
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

    pub fn return_type(&self) -> Option<&Type> {
        self.top
            .as_deref()
            .and_then(|scope| scope.ret_ty.as_deref())
    }
}
