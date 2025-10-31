/// Global store module - manages global state and module registry
use crate::language::syntax::ast::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "cli")]
use crate::engine::plugin::loader::PluginInfo;

/// Global store for managing application state
#[derive(Clone)]
pub struct GlobalStore {
    variables: Arc<RwLock<HashMap<String, Value>>>,
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
    #[cfg(feature = "cli")]
    plugins: Arc<RwLock<HashMap<String, (PluginInfo, Vec<u8>)>>>,
}

#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub path: String,
    pub variables: HashMap<String, Value>,
}

impl GlobalStore {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "cli")]
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_variable(&self, name: String, value: Value) {
        if let Ok(mut vars) = self.variables.write() {
            vars.insert(name, value);
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<Value> {
        if let Ok(vars) = self.variables.read() {
            vars.get(name).cloned()
        } else {
            None
        }
    }

    pub fn register_module(&self, path: String, info: ModuleInfo) {
        if let Ok(mut mods) = self.modules.write() {
            mods.insert(path, info);
        }
    }

    pub fn get_module(&self, path: &str) -> Option<ModuleInfo> {
        if let Ok(mods) = self.modules.read() {
            mods.get(path).cloned()
        } else {
            None
        }
    }

    #[cfg(feature = "cli")]
    pub fn register_plugin(&self, key: String, info: PluginInfo, wasm_bytes: Vec<u8>) {
        if let Ok(mut plugins) = self.plugins.write() {
            plugins.insert(key, (info, wasm_bytes));
        }
    }

    #[cfg(feature = "cli")]
    pub fn get_plugin(&self, key: &str) -> Option<(PluginInfo, Vec<u8>)> {
        if let Ok(plugins) = self.plugins.read() {
            plugins.get(key).cloned()
        } else {
            None
        }
    }

    pub fn clear(&self) {
        if let Ok(mut vars) = self.variables.write() {
            vars.clear();
        }
        if let Ok(mut mods) = self.modules.write() {
            mods.clear();
        }
    }
}

impl Default for GlobalStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "test_shared_store.rs"]
mod tests;
