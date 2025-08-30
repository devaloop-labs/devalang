use std::collections::HashMap;

use devalang_types::Value;

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
