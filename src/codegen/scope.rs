use crate::codegen::symbol_table::BasicValueEnumWrapper;
use crate::codegen::symbol_table::SymbolTable;
use inkwell::values::BasicValueEnum;

#[derive(Debug, Clone)]
pub struct Scope<'ctx> {
    symtb: SymbolTable<'ctx>,
    parent: Option<Box<Scope<'ctx>>>,
}

impl Default for Scope<'_> {
    fn default() -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent: None,
        }
    }
}

impl<'ctx> Scope<'ctx> {
    pub fn new(parent: Option<Box<Scope<'ctx>>>) -> Self {
        Self {
            symtb: SymbolTable::default(),
            parent,
        }
    }

    pub fn define(
        &mut self,
        name: &String,
        val_enum: BasicValueEnumWrapper<'ctx>,
    ) -> Result<(), String> {
        self.symtb.define(name, val_enum)
    }

    pub fn resolve(&self, name: &String) -> Option<&BasicValueEnumWrapper<'ctx>> {
        if let Some(var) = self.symtb.resolve(name) {
            Some(var)
        } else if let Some(ref parent) = self.parent {
            parent.resolve(name)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ScopeStack<'ctx> {
    top: Option<Box<Scope<'ctx>>>,
    len: usize,
}

impl Default for ScopeStack<'_> {
    fn default() -> Self {
        let mut default = Self { top: None, len: 0 };
        default.push(); // global scope
        default
    }
}

impl<'ctx> ScopeStack<'ctx> {
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

    pub fn peek(&self) -> Option<&Scope<'ctx>> {
        self.top.as_deref()
    }

    pub fn peek_mut(&mut self) -> Option<&mut Scope<'ctx>> {
        self.top.as_deref_mut()
    }
}
