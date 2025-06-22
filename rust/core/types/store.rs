use std::collections::HashMap;

use crate::core::types::{module::Module, variable::VariableValue};

#[derive(Debug, Default)]
pub struct GlobalStore {
    pub modules: HashMap<String, Module>,
}

#[derive(Debug, Default, Clone)]
pub struct VariableTable {
    pub variables: HashMap<String, VariableValue>,
}

impl VariableTable {
    pub fn new() -> Self {
        VariableTable {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: VariableValue) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<VariableValue> {
        self.variables.remove(name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExportTable {
    pub exports: HashMap<String, VariableValue>,
}

impl ExportTable {
    pub fn new() -> Self {
        ExportTable {
            exports: HashMap::new(),
        }
    }

    pub fn add_export(&mut self, name: String, value: VariableValue) {
        self.exports.insert(name, value);
    }

    pub fn get_export(&self, name: &str) -> Option<&VariableValue> {
        self.exports.get(name)
    }

    pub fn remove_export(&mut self, name: &str) -> Option<VariableValue> {
        self.exports.remove(name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImportTable {
    pub imports: HashMap<String, VariableValue>,
}

impl ImportTable {
    pub fn new() -> Self {
        ImportTable {
            imports: HashMap::new(),
        }
    }

    pub fn add_import(&mut self, name: String, value: VariableValue) {
        self.imports.insert(name, value);
    }

    pub fn get_import(&self, name: &str) -> Option<&VariableValue> {
        self.imports.get(name)
    }

    pub fn remove_import(&mut self, name: &str) -> Option<VariableValue> {
        self.imports.remove(name)
    }
}