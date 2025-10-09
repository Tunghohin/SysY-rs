use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BasicValueEnumWrapper<'ctx> {
    pub inner: BasicValueEnum<'ctx>,
    pub constness: bool,
}

impl<'ctx> BasicValueEnumWrapper<'ctx> {
    pub fn is_const(&self) -> bool {
        self.constness
    }
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'ctx> {
    meta: HashMap<String, BasicValueEnumWrapper<'ctx>>,
}

impl<'ctx> SymbolTable<'ctx> {
    pub fn define(
        &mut self,
        name: &String,
        var_meta: BasicValueEnumWrapper<'ctx>,
    ) -> Result<(), String> {
        if self.meta.contains_key(name) {
            Err(format!("Redefined variable: {}", name))
        } else {
            self.meta.insert(name.clone(), var_meta);
            Ok(())
        }
    }

    pub fn resolve(&self, name: &String) -> Option<&BasicValueEnumWrapper<'ctx>> {
        if self.meta.contains_key(name) {
            self.meta.get(name)
        } else {
            None
        }
    }
}
