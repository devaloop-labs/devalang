use std::collections::HashMap;

use devalang_types::Value;

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
