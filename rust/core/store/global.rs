use crate::core::{
    parser::statement::Statement,
    preprocessor::module::Module,
    store::{function::FunctionTable, variable::VariableTable},
};
use devalang_types::PluginInfo;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GlobalStore {
    pub modules: HashMap<String, Module>,
    pub variables: VariableTable,
    pub functions: FunctionTable,
    pub events: HashMap<String, Vec<Statement>>,
    pub plugins: HashMap<String, (PluginInfo, Vec<u8>)>,
}

impl Default for GlobalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStore {
    pub fn new() -> Self {
        GlobalStore {
            modules: HashMap::new(),
            functions: FunctionTable::new(),
            variables: VariableTable::new(),
            events: HashMap::new(),
            plugins: HashMap::new(),
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

    pub fn register_event_handler(&mut self, event: &str, handler: Statement) {
        self.events
            .entry(event.to_string())
            .or_default()
            .push(handler);
    }

    pub fn get_event_handlers(&self, event: &str) -> Option<&Vec<Statement>> {
        self.events.get(event)
    }
}
