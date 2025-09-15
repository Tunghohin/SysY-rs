use std::collections::HashMap;

#[derive(Debug)]
enum Type {
    Int,
}

#[derive(Debug, Default)]
pub struct SemanticChecker {
    symtb: SymbolTable,
}

impl SemanticChecker {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    rtype: HashMap<String, Type>,
    place: HashMap<String, usize>,
}

impl SymbolTable {
    fn new() -> Self {
        Self::default()
    }
}
