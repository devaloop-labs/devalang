use std::collections::HashMap;

use devalang_types::Value;

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
        // First try the local table
        if let Some(v) = self.variables.get(name) {
            return Some(v);
        }

        // Then walk parent chain if present
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
