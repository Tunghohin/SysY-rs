use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'ctx> {
    meta: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> SymbolTable<'ctx> {
    pub fn define(&mut self, name: &String, var_meta: BasicValueEnum<'ctx>) -> Result<(), String> {
        if self.meta.contains_key(name) {
            Err(format!("Redefined variable: {}", name))
        } else {
            self.meta.insert(name.clone(), var_meta);
            Ok(())
        }
    }

    pub fn resolve(&self, name: &String) -> Option<&BasicValueEnum<'ctx>> {
        if self.meta.contains_key(name) {
            self.meta.get(name)
        } else {
            None
        }
    }
}
