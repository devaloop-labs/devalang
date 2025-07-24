use std::collections::HashMap;
use crate::core::{
    preprocessor::module::Module,
    store::{ function::FunctionTable, variable::VariableTable },
};

#[derive(Debug, Clone)]
pub struct GlobalStore {
    pub modules: HashMap<String, Module>,
    pub variables: VariableTable,
    pub functions: FunctionTable,
}

impl GlobalStore {
    pub fn new() -> Self {
        GlobalStore {
            modules: HashMap::new(),
            functions: FunctionTable::new(),
            variables: VariableTable::new(),
        }
    }

    pub fn insert_module(&mut self, path: String, module: Module) {
        self.modules.insert(path, module);
    }

    pub fn modules_mut(&mut self) -> &mut HashMap<String, Module> {
        &mut self.modules
    }

    pub fn get_module(&self, path: &str) -> Option<&Module> {
        self.modules.get(path)
    }

    pub fn remove_module(&mut self, path: &str) -> Option<Module> {
        self.modules.remove(path)
    }
}
