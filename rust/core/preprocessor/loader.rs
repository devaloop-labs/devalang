use std::{ collections::HashMap, path::Path };
use crate::{
    core::{
        error::ErrorHandler,
        lexer::{ token::Token, Lexer },
        parser::{ statement::{ Statement, StatementKind }, driver::Parser },
        preprocessor::{ module::Module, processor::process_modules },
        store::global::GlobalStore,
        utils::path::normalize_path,
    },
    utils::logger::Logger,
};
use crate::core::preprocessor::resolver::driver::{
    resolve_all_modules,
    resolve_and_flatten_all_modules,
};
use crate::core::utils::path::resolve_relative_path;

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
            base_dir: base_dir,
        }
    }

    pub fn from_raw_source(
        entry_path: &str,
        output_path: &str,
        content: &str,
        global_store: &mut GlobalStore
    ) -> Self {
        let normalized_entry_path = normalize_path(entry_path);

        let mut module = Module::new(&entry_path);
        module.content = content.to_string();

        global_store.insert_module(normalized_entry_path.to_string(), module);

        Self {
            entry: normalized_entry_path.to_string(),
            output: output_path.to_string(),
            base_dir: "".to_string(),
        }
    }

    pub fn extract_statements_map(
        &self,
        global_store: &GlobalStore
    ) -> HashMap<String, Vec<Statement>> {
        global_store.modules
            .iter()
            .map(|(path, module)| (path.clone(), module.statements.clone()))
            .collect()
    }

    pub fn load_single_module(&self, global_store: &mut GlobalStore) -> Result<Module, String> {
        let mut module = global_store.modules
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

        // SECTION Error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &module.statements);

        global_store.modules.insert(self.entry.clone(), module.clone());

        Ok(module)
    }
    
    pub fn load_wasm_module(&self, global_store: &mut GlobalStore) -> Result<(), String> {
        // Step one : Load the module from the global store
        let module = {
            let module_ref = global_store.modules
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

        // Step four : error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &updated_module.statements);

        // Final step : insert the updated module back into the global store
        global_store.modules.insert(self.entry.clone(), updated_module);

        Ok(())
    }

    #[cfg(feature = "cli")]
    pub fn load_all_modules(
        &self,
        global_store: &mut GlobalStore
    ) -> (HashMap<String, Vec<Token>>, HashMap<String, Vec<Statement>>) {
        // SECTION Load the entry module and its dependencies

        let tokens_by_module = self.load_module_recursively(&self.entry, global_store);

        // SECTION Process and resolve modules
        process_modules(self, global_store);
        resolve_all_modules(self, global_store);

        let statemnts_by_module = resolve_and_flatten_all_modules(global_store);

        (tokens_by_module, statemnts_by_module)
    }

    #[cfg(feature = "cli")]
    fn load_module_recursively(
        &self,
        raw_path: &str,
        global_store: &mut GlobalStore
    ) -> HashMap<String, Vec<Token>> {
        let path = normalize_path(raw_path);

        // Check if already loaded
        if global_store.modules.contains_key(&path) {
            return HashMap::new();
        }

        let lexer = Lexer::new();
        let tokens = lexer.lex_tokens(&path);

        let mut parser = Parser::new();
        parser.set_current_module(path.clone());

        let statements = parser.parse_tokens(tokens.clone(), global_store);

        // Error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &statements);

        if error_handler.has_errors() {
            let logger = Logger::new();
            for error in error_handler.get_errors() {
                let trace = format!("{}:{}:{}", path, error.line, error.column);
                logger.log_error_with_stacktrace(&error.message, &trace);
            }
        }

        // Insert module into store
        let mut module = Module::new(&path);
        module.tokens = tokens.clone();
        module.statements = statements.clone();
        global_store.insert_module(path.clone(), module);

        // Load dependencies
        self.load_module_imports(&path, global_store);

        // Return tokens per module
        global_store.modules
            .iter()
            .map(|(p, m)| (p.clone(), m.tokens.clone()))
            .collect()
    }

    #[cfg(feature = "cli")]
    fn load_module_imports(&self, path: &String, global_store: &mut GlobalStore) {
        let import_paths: Vec<String> = {
            let current_module = match global_store.modules.get(path) {
                Some(module) => module,
                None => {
                    eprintln!("[warn] Cannot resolve imports: module '{}' not found in store", path);
                    return;
                }
            };

            current_module.statements
                .iter()
                .filter_map(|stmt| {
                    if let StatementKind::Import { source, .. } = &stmt.kind {
                        Some(source.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        for import_path in import_paths {
            let resolved = resolve_relative_path(path, &import_path);
            self.load_module_recursively(&resolved, global_store);
        }
    }
}
