use std::collections::HashMap;

use crate::ast::Value;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VariableTable {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Box<VariableTable>>,
}

impl VariableTable {
    pub fn new() -> Self {
        VariableTable {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn with_parent(parent: VariableTable) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        if let Some(v) = self.variables.get(name) {
            return Some(v);
        }
        let mut current = &self.parent;
        while let Some(boxed) = current {
            if let Some(v) = boxed.variables.get(name) {
                return Some(v);
            }
            current = &boxed.parent;
        }
        None
    }

    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ImportTable {
    pub imports: HashMap<String, Value>,
}

impl ImportTable {
    pub fn new() -> Self {
        ImportTable {
            imports: HashMap::new(),
        }
    }

    pub fn add_import(&mut self, name: String, value: Value) {
        self.imports.insert(name, value);
    }

    pub fn get_import(&self, name: &str) -> Option<&Value> {
        self.imports.get(name)
    }

    pub fn remove_import(&mut self, name: &str) -> Option<Value> {
        self.imports.remove(name)
    }
}

use crate::ast::Statement;

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

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExportTable {
    pub exports: HashMap<String, Value>,
}

impl ExportTable {
    pub fn new() -> Self {
        ExportTable {
            exports: HashMap::new(),
        }
    }

    pub fn add_export(&mut self, name: String, value: Value) {
        self.exports.insert(name, value);
    }

    pub fn get_export(&self, name: &str) -> Option<&Value> {
        self.exports.get(name)
    }

    pub fn remove_export(&mut self, name: &str) -> Option<Value> {
        self.exports.remove(name)
    }
}

// GlobalStore lives in core to avoid cyclic dependencies (it references Module).
