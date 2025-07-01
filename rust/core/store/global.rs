use std::collections::HashMap;
use crate::core::preprocessor::module::Module;

#[derive(Debug, Default, Clone)]
pub struct GlobalStore {
    pub modules: HashMap<String, Module>,
}

impl GlobalStore {
    pub fn new() -> Self {
        GlobalStore {
            modules: HashMap::new(),
        }
    }

    pub fn resolve_all_imports(&mut self) {
        for module in self.modules.values_mut() {
            for (name, source_path) in &module.import_table.imports {
                println!("Resolving import: {} from {:?}", name, source_path);
            }
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
