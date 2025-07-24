use std::{ collections::{ HashMap, HashSet }, path::Path };
use crate::{
    core::{
        error::ErrorHandler,
        lexer::{ token::Token, Lexer },
        parser::{ driver::Parser, statement::{ Statement, StatementKind } },
        preprocessor::{ module::Module, processor::process_modules },
        shared::{ bank::BankFile, value::Value },
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

        // SECTION Injecting bank triggers if any
        if let Err(e) = self.inject_bank_triggers(&mut module, "808") {
            return Err(format!("Failed to inject bank triggers: {}", e));
        }

        global_store.modules.insert(self.entry.clone(), module.clone());

        // SECTION Error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &module.statements);

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

        // Step four : Injecting bank triggers if any
        if let Err(e) = self.inject_bank_triggers(&mut updated_module, "808") {
            return Err(format!("Failed to inject bank triggers: {}", e));
        }

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

        // SECTION Flatten all modules to get statements (+ injects)
        let statements_by_module = resolve_and_flatten_all_modules(global_store);

        (tokens_by_module, statements_by_module)
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

        // Insert module into store
        let mut module = Module::new(&path);
        module.tokens = tokens.clone();
        module.statements = statements.clone();

        let mut module_variables = module.variable_table.clone();

        // Inject triggers for each bank used in module
        for bank_name in self.extract_bank_names(&statements) {
            let module_updated = self
                .inject_bank_triggers(&mut module, &bank_name)
                .map_err(|e| format!("Failed to inject bank triggers: {}", e));

            if let Err(e) = module_updated {
                eprintln!("[warn] {}", e);
            }

            // Update the variable table with the bank variables
            if let Some(bank_variables) = module.variable_table.get(&bank_name) {
                if let Value::Map(bank_map) = bank_variables {
                    for (key, value) in bank_map {
                        module_variables.set(format!("{}.{}", bank_name, key), value.clone());
                    }
                }
            }
        }

        // Update the module's variable table
        // module.variable_table = module_variables;

        // Inject the module into the global store
        global_store.insert_module(path.clone(), module);

        // Load dependencies
        self.load_module_imports(&path, global_store);

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

    pub fn inject_bank_triggers(&self, module: &mut Module, bank_name: &str) -> Result<(), String> {
        let bank_path = Path::new("./.deva/bank").join(bank_name);
        let bank_file_path = bank_path.join("bank.toml");

        if !bank_file_path.exists() {
            return Ok(());
        }

        let content = std::fs
            ::read_to_string(&bank_file_path)
            .map_err(|e| format!("Failed to read '{}': {}", bank_file_path.display(), e))?;

        let parsed: BankFile = toml
            ::from_str(&content)
            .map_err(|e| format!("Failed to parse '{}': {}", bank_file_path.display(), e))?;

        let mut bank_map = HashMap::new();

        for bank_trigger in parsed.triggers.unwrap_or_default() {
            let trigger_name = bank_trigger.name.clone().replace("./", "");
            let bank_trigger_path = format!("devalang://bank/{}/{}", bank_name, trigger_name);

            bank_map.insert(bank_trigger.name.clone(), Value::String(bank_trigger_path.clone()));

            if module.variable_table.variables.contains_key(bank_name) {
                eprintln!(
                    "⚠️ Trigger '{}' already defined in module '{}', skipping injection.",
                    bank_name,
                    module.path
                );
                continue;
            }

            module.variable_table.set(
                format!("{}.{}", bank_name, bank_trigger.name),
                Value::String(bank_trigger_path.clone())
            );
        }

        // Inject the map under the bank name
        module.variable_table.set(bank_name.to_string(), Value::Map(bank_map));

        Ok(())
    }

    fn extract_bank_names(&self, statements: &[Statement]) -> HashSet<String> {
        let mut banks = HashSet::new();

        for stmt in statements {
            match &stmt.kind {
                StatementKind::Trigger { entity, .. } => {
                    let parts: Vec<&str> = entity.split('.').collect();
                    if parts.len() >= 2 {
                        banks.insert(parts[0].to_string());
                    }
                }

                StatementKind::Bank => {
                    if let Value::String(name) = &stmt.value {
                        banks.insert(name.clone());
                    }
                }

                StatementKind::Group { .. } => {
                    let group_body = match &stmt.value {
                        Value::Map(map) => {
                            if let Some(Value::Block(body)) = map.get("body") {
                                body
                            } else {
                                continue;
                            }
                        }
                        _ => {
                            continue;
                        }
                    };

                    let inner_banks = self.extract_bank_names(&group_body);
                    banks.extend(inner_banks);
                }

                StatementKind::If { .. } => {
                    let if_body = match &stmt.value {
                        Value::Map(map) => {
                            if let Some(Value::Block(body)) = map.get("body") {
                                body
                            } else {
                                continue;
                            }
                        }
                        _ => {
                            continue;
                        }
                    };

                    let inner_banks = self.extract_bank_names(&if_body);
                    banks.extend(inner_banks);
                }

                _ => {}
            }
        }

        banks
    }
}
