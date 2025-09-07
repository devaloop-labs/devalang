#[cfg(feature = "cli")]
use crate::core::preprocessor::resolver::driver::{
    resolve_all_modules, resolve_and_flatten_all_modules,
};
// resolve_relative_path moved to loader_helpers
use crate::core::{
    error::ErrorHandler,
    lexer::{driver::Lexer, token::Token},
    parser::{driver::parser::Parser, statement::Statement},
    preprocessor::{module::Module, processor::handlers::process_modules},
    store::global::GlobalStore,
};
use devalang_utils::path::normalize_path;
use std::{collections::HashMap, path::Path};

mod inject;
mod loader_helpers;

pub struct ModuleLoader {
    pub entry: String,
    pub output: String,
    pub base_dir: String,
}

impl ModuleLoader {
    pub fn new(entry: &str, output: &str) -> Self {
        let base_dir = Path::new(entry)
            .parent()
            .unwrap_or(Path::new(""))
            .to_string_lossy()
            .replace('\\', "/");

        Self {
            entry: entry.to_string(),
            output: output.to_string(),
            base_dir,
        }
    }

    pub fn from_raw_source(
        entry_path: &str,
        output_path: &str,
        content: &str,
        global_store: &mut GlobalStore,
    ) -> Self {
        let normalized_entry_path = normalize_path(entry_path);

        let mut module = Module::new(entry_path);
        module.content = content.to_string();

        // Insert a module stub containing the provided content into the
        // global store. This is used by the WASM APIs and tests which
        // operate on in-memory sources instead of files on disk.
        global_store.insert_module(normalized_entry_path.to_string(), module);

        Self {
            entry: normalized_entry_path.to_string(),
            output: output_path.to_string(),
            base_dir: "".to_string(),
        }
    }

    pub fn extract_statements_map(
        &self,
        global_store: &GlobalStore,
    ) -> HashMap<String, Vec<Statement>> {
        global_store
            .modules
            .iter()
            .map(|(path, module)| (path.clone(), module.statements.clone()))
            .collect()
    }

    pub fn load_single_module(&self, global_store: &mut GlobalStore) -> Result<Module, String> {
        let mut module = global_store
            .modules
            .remove(&self.entry)
            .ok_or_else(|| format!("Module not found in store for path: {}", self.entry))?;

        // SECTION Lexing the module content
        let lexer = Lexer::new();
        let tokens = lexer
            .lex_from_source(&module.content)
            .map_err(|e| format!("Lexer failed: {}", e))?;

        module.tokens = tokens.clone();

        // SECTION Parsing tokens into statements
        let mut parser = Parser::new();
        parser.set_current_module(self.entry.clone());
        let statements = parser.parse_tokens(tokens, global_store);
        module.statements = statements;

        // SECTION Injecting bank triggers if any (legacy default for single-module run)
        if let Err(e) = inject::inject_bank_triggers(&mut module, "808", None) {
            return Err(format!("Failed to inject bank triggers: {}", e));
        }

        for (plugin_name, alias) in inject::extract_plugin_uses(&module.statements) {
            inject::load_plugin_and_register(&mut module, &plugin_name, &alias, global_store);
        }

        global_store
            .modules
            .insert(self.entry.clone(), module.clone());

        // SECTION Error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &module.statements);

        Ok(module)
    }

    pub fn load_wasm_module(&self, global_store: &mut GlobalStore) -> Result<(), String> {
        // Step one : Load the module from the global store
        let module = {
            let module_ref = global_store
                .modules
                .get(&self.entry)
                .ok_or_else(|| format!("❌ Module not found for path: {}", self.entry))?;

            Module::from_existing(&self.entry, module_ref.content.clone())
        };

        // Step two : lexing
        let lexer = Lexer::new();
        let tokens = lexer
            .lex_from_source(&module.content)
            .map_err(|e| format!("Lexer failed: {}", e))?;

        // Step three : parsing
        let mut parser = Parser::new();
        parser.set_current_module(self.entry.clone());

        let statements = parser.parse_tokens(tokens.clone(), global_store);

        let mut updated_module = module;
        updated_module.tokens = tokens;
        updated_module.statements = statements;

        // Step four : Injecting bank triggers if any
        if let Err(e) = inject::inject_bank_triggers(&mut updated_module, "808", None) {
            return Err(format!("Failed to inject bank triggers: {}", e));
        }

        // Insert the updated module into the global store before processing so
        // process_modules can operate on it and populate variable_table, imports,
        // and other derived structures.
        global_store
            .modules
            .insert(self.entry.clone(), updated_module.clone());

        // Process modules to populate module.variable_table, import/export tables,
        // and other derived structures so runtime execution can resolve groups/synths.
        process_modules(self, global_store);

        for (plugin_name, alias) in inject::extract_plugin_uses(&updated_module.statements) {
            inject::load_plugin_and_register(
                &mut updated_module,
                &plugin_name,
                &alias,
                global_store,
            );
        }

        // Step four : error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &updated_module.statements);

        // Final step : also expose module-level variables and functions into the global store
        // so runtime evaluation (render_audio) can find group/synth definitions.
        // Use the module instance that was actually processed by `process_modules`
        // (it lives in `global_store.modules`) because `updated_module` is a local
        // clone and won't contain the mutations applied by `process_modules`.
        if let Some(stored_module) = global_store.modules.get(&self.entry) {
            global_store
                .variables
                .variables
                .extend(stored_module.variable_table.variables.clone());
            global_store
                .functions
                .functions
                .extend(stored_module.function_table.functions.clone());
        } else {
            // Fallback to the local updated_module if for any reason the module
            // wasn't inserted into the store (defensive programming).
            global_store
                .variables
                .variables
                .extend(updated_module.variable_table.variables.clone());
            global_store
                .functions
                .functions
                .extend(updated_module.function_table.functions.clone());
        }

        Ok(())
    }

    #[cfg(feature = "cli")]
    pub fn load_all_modules(
        &self,
        global_store: &mut GlobalStore,
    ) -> (HashMap<String, Vec<Token>>, HashMap<String, Vec<Statement>>) {
        // SECTION Load the entry module and its dependencies
        let tokens_by_module = self.load_module_recursively(&self.entry, global_store);

        // SECTION Process and resolve modules
        process_modules(self, global_store);
        resolve_all_modules(self, global_store);

        // SECTION Flatten all modules to get statements (+ injects)
        let statements_by_module = resolve_and_flatten_all_modules(global_store);

        (tokens_by_module, statements_by_module)
    }

    #[cfg(feature = "cli")]
    fn load_module_recursively(
        &self,
        raw_path: &str,
        global_store: &mut GlobalStore,
    ) -> HashMap<String, Vec<Token>> {
        crate::core::preprocessor::loader::loader_helpers::load_module_recursively(
            raw_path,
            global_store,
        )
    }

    #[cfg(feature = "cli")]
    #[allow(dead_code)]
    fn load_module_imports(&self, path: &String, global_store: &mut GlobalStore) {
        crate::core::preprocessor::loader::loader_helpers::load_module_imports(path, global_store)
    }
}
