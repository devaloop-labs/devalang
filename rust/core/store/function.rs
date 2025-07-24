use std::collections::HashMap;
use crate::core::parser::statement::Statement;

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionTable {
    pub functions: HashMap<String, FunctionDef>,
}

impl FunctionTable {
    pub fn new() -> Self {
        FunctionTable {
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, function: FunctionDef) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }

    pub fn remove_function(&mut self, name: &str) -> Option<FunctionDef> {
        self.functions.remove(name)
    }
}