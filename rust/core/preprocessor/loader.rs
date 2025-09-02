#[cfg(feature = "cli")]
use crate::core::preprocessor::resolver::driver::{
    resolve_all_modules, resolve_and_flatten_all_modules,
};
#[cfg(feature = "cli")]
use crate::core::utils::path::resolve_relative_path;
#[cfg_attr(not(feature = "cli"), allow(unused_imports))]
use crate::core::{
    error::ErrorHandler,
    lexer::{driver::Lexer, token::Token},
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    plugin::loader::load_plugin,
    preprocessor::{module::Module, processor::process_modules},
    store::global::GlobalStore,
    utils::path::normalize_path,
};
use devalang_types::{BankFile, Value};
#[cfg(feature = "cli")]
use devalang_utils::logger::{LogLevel, Logger};
use std::{collections::HashMap, path::Path};

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
        if let Err(e) = self.inject_bank_triggers(&mut module, "808", None) {
            return Err(format!("Failed to inject bank triggers: {}", e));
        }

        for (plugin_name, alias) in self.extract_plugin_uses(&module.statements) {
            self.load_plugin_and_register(&mut module, &plugin_name, &alias, global_store);
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
        if let Err(e) = self.inject_bank_triggers(&mut updated_module, "808", None) {
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

        for (plugin_name, alias) in self.extract_plugin_uses(&updated_module.statements) {
            self.load_plugin_and_register(&mut updated_module, &plugin_name, &alias, global_store);
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
        let path = normalize_path(raw_path);

        // Check if already loaded
        if global_store.modules.contains_key(&path) {
            return HashMap::new();
        }

        let lexer = Lexer::new();
        let tokens = match lexer.lex_tokens(&path) {
            Ok(t) => t,
            Err(e) => {
                let logger = Logger::new();
                logger.log_message(LogLevel::Error, &format!("Failed to lex '{}': {}", path, e));
                return HashMap::new();
            }
        };

        let mut parser = Parser::new();
        parser.set_current_module(path.clone());

        let statements = parser.parse_tokens(tokens.clone(), global_store);

        // Insert module into store
        let mut module = Module::new(&path);
        module.tokens = tokens.clone();
        module.statements = statements.clone();

        // Inject triggers for each bank used in module, respecting aliases
        for (bank_name, alias_opt) in self.extract_bank_decls(&statements) {
            if let Err(e) = self.inject_bank_triggers(&mut module, &bank_name, alias_opt) {
                eprintln!("Failed to inject bank triggers for '{}': {}", bank_name, e);
            }
        }

        for (plugin_name, alias) in self.extract_plugin_uses(&statements) {
            self.load_plugin_and_register(&mut module, &plugin_name, &alias, global_store);
        }

        // Inject module variables and functions into global store
        global_store
            .variables
            .variables
            .extend(module.variable_table.variables.clone());
        global_store
            .functions
            .functions
            .extend(module.function_table.functions.clone());

        // Inject the module into the global store
        global_store.insert_module(path.clone(), module);

        // Load dependencies
        self.load_module_imports(&path, global_store);

        // Error handling (use the module now in the store to include injected errors)
        let mut error_handler = ErrorHandler::new();
        if let Some(current_module) = global_store.modules.get(&path) {
            error_handler.detect_from_statements(&mut parser, &current_module.statements);
        } else {
            error_handler.detect_from_statements(&mut parser, &statements);
        }

        if error_handler.has_errors() {
            let logger = Logger::new();
            for error in error_handler.get_errors() {
                let trace = format!("{}:{}:{}", path, error.line, error.column);
                logger.log_error_with_stacktrace(&error.message, &trace);
            }
        }

        // Return tokens per module
        global_store
            .modules
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
                    eprintln!(
                        "[warn] Cannot resolve imports: module '{}' not found in store",
                        path
                    );
                    return;
                }
            };

            current_module
                .statements
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

    pub fn inject_bank_triggers(
        &self,
        module: &mut Module,
        bank_name: &str,
        alias_override: Option<String>,
    ) -> Result<Module, String> {
        let default_alias = bank_name
            .split('.')
            .next_back()
            .unwrap_or(bank_name)
            .to_string();
        let alias_ref = alias_override.as_deref().unwrap_or(&default_alias);

        let bank_path = match devalang_utils::path::get_deva_dir() {
            Ok(dir) => dir.join("banks").join(bank_name),
            Err(_) => Path::new("./.deva").join("banks").join(bank_name),
        };
        let bank_toml_path = bank_path.join("bank.toml");

        if !bank_toml_path.exists() {
            return Ok(module.clone());
        }

        let content = std::fs::read_to_string(&bank_toml_path)
            .map_err(|e| format!("Failed to read '{}': {}", bank_toml_path.display(), e))?;

        let parsed_bankfile: BankFile = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse '{}': {}", bank_toml_path.display(), e))?;

        let mut bank_map = HashMap::new();

        for bank_trigger in parsed_bankfile.triggers.unwrap_or_default() {
            // Use the configured path from the bank file as the entity reference so
            // that bank entries can point to files or nested paths. Clean common
            // local prefixes like "./" to keep the URI tidy.
            let entity_ref = bank_trigger
                .path
                .clone()
                .replace("\\", "/")
                .replace("./", "");
            let bank_trigger_path = format!("devalang://bank/{}/{}", bank_name, entity_ref);

            // Keep the trigger key as declared (bank_trigger.name) but expose its
            // value as a devalang://bank URI pointing to the configured path.
            bank_map.insert(
                bank_trigger.name.clone(),
                Value::String(bank_trigger_path.clone()),
            );

            if module.variable_table.variables.contains_key(alias_ref) {
                eprintln!(
                    "⚠️ Trigger '{}' already defined in module '{}', skipping injection.",
                    alias_ref, module.path
                );
                continue;
            }

            module.variable_table.set(
                format!("{}.{}", alias_ref, bank_trigger.name),
                Value::String(bank_trigger_path.clone()),
            );
        }

        // Inject the map under the bank name
        module
            .variable_table
            .set(alias_ref.to_string(), Value::Map(bank_map));

        Ok(module.clone())
    }

    #[cfg_attr(not(feature = "cli"), allow(dead_code))]
    fn extract_bank_decls(&self, statements: &[Statement]) -> Vec<(String, Option<String>)> {
        let mut banks = Vec::new();

        for stmt in statements {
            if let StatementKind::Bank { alias } = &stmt.kind {
                let name_opt = match &stmt.value {
                    Value::String(s) => Some(s.clone()),
                    Value::Identifier(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                };
                if let Some(name) = name_opt {
                    banks.push((name, alias.clone()));
                }
            }
        }

        banks
    }

    fn extract_plugin_uses(&self, statements: &[Statement]) -> Vec<(String, String)> {
        let mut plugins = Vec::new();

        for stmt in statements {
            if let StatementKind::Use { name, alias } = &stmt.kind {
                let alias_name = alias
                    .clone()
                    .unwrap_or_else(|| name.split('.').next_back().unwrap_or(name).to_string());
                plugins.push((name.clone(), alias_name));
            }
        }

        plugins
    }

    fn load_plugin_and_register(
        &self,
        module: &mut Module,
        plugin_name: &str,
        alias: &str,
        global_store: &mut GlobalStore,
    ) {
        // plugin_name expected format: "author.name"
        let mut parts = plugin_name.split('.');
        let author = match parts.next() {
            Some(a) if !a.is_empty() => a,
            _ => {
                eprintln!("Invalid plugin name '{}': missing author", plugin_name);
                return;
            }
        };
        let name = match parts.next() {
            Some(n) if !n.is_empty() => n,
            _ => {
                eprintln!("Invalid plugin name '{}': missing name", plugin_name);
                return;
            }
        };
        if parts.next().is_some() {
            eprintln!(
                "Invalid plugin name '{}': expected <author>.<name>",
                plugin_name
            );
            return;
        }

        // Enforce presence in .devalang config when plugin exists locally
        // Build expected URI from author/name
        let expected_uri = format!("devalang://plugin/{}.{}", author, name);

        // Detect local presence (preferred and legacy layouts)
        let root = match devalang_utils::path::get_deva_dir() {
            Ok(dir) => dir,
            Err(_) => Path::new("./.deva").to_path_buf(),
        };
        let plugin_dir_preferred = root.join("plugins").join(format!("{}.{}", author, name));
        let toml_path_preferred = plugin_dir_preferred.join("plugin.toml");
        let plugin_dir_fallback = root.join("plugins").join(author).join(name);
        let toml_path_fallback = plugin_dir_fallback.join("plugin.toml");
        let exists_locally = toml_path_preferred.exists() || toml_path_fallback.exists();

        if exists_locally {
            // Load config and verify plugin is declared
            let cfg_opt = crate::config::ops::load_config(None);
            let mut declared = false;
            if let Some(cfg) = cfg_opt {
                if let Some(list) = cfg.plugins {
                    declared = list.iter().any(|p| p.path == expected_uri);
                }
            }
            if !declared {
                // Inject a single, clear error into the module so it is reported once by the error handler
                module.statements.push(Statement {
                    kind: StatementKind::Error {
                        message: "plugin present in local files but missing in .devalang config"
                            .to_string(),
                    },
                    value: Value::Null,
                    indent: 0,
                    line: 0,
                    column: 0,
                });
                return;
            }
        }

        match load_plugin(author, name) {
            Ok((info, wasm)) => {
                let uri = format!("devalang://plugin/{}.{}", author, name);
                global_store
                    .plugins
                    .insert(format!("{}:{}", author, name), (info, wasm));
                // Set alias to URI, and inject exported variables
                module
                    .variable_table
                    .set(alias.to_string(), Value::String(uri.clone()));
                // Also expose alias at global level so runtime can resolve it
                global_store
                    .variables
                    .set(alias.to_string(), Value::String(uri.clone()));

                if let Some((plugin_info, _)) =
                    global_store.plugins.get(&format!("{}:{}", author, name))
                {
                    for exp in &plugin_info.exports {
                        match exp.kind.as_str() {
                            "number" => {
                                if let Some(toml::Value::String(s)) = &exp.default {
                                    if let Ok(n) = s.parse::<f32>() {
                                        module.variable_table.set(
                                            format!("{}.{}", alias, exp.name),
                                            Value::Number(n),
                                        );
                                    }
                                } else if let Some(toml::Value::Integer(i)) = &exp.default {
                                    module.variable_table.set(
                                        format!("{}.{}", alias, exp.name),
                                        Value::Number(*i as f32),
                                    );
                                } else if let Some(toml::Value::Float(f)) = &exp.default {
                                    module.variable_table.set(
                                        format!("{}.{}", alias, exp.name),
                                        Value::Number(*f as f32),
                                    );
                                }
                            }
                            "string" => {
                                if let Some(toml::Value::String(s)) = &exp.default {
                                    module.variable_table.set(
                                        format!("{}.{}", alias, exp.name),
                                        Value::String(s.clone()),
                                    );
                                }
                            }
                            "bool" => {
                                if let Some(toml::Value::Boolean(b)) = &exp.default {
                                    module
                                        .variable_table
                                        .set(format!("{}.{}", alias, exp.name), Value::Boolean(*b));
                                }
                            }
                            "synth" => {
                                // Provide a discoverable marker: alias.<synthName> resolves to alias.synthName waveform string
                                module.variable_table.set(
                                    format!("{}.{}", alias, exp.name),
                                    Value::String(format!("{}.{}", alias, exp.name)),
                                );
                            }
                            _ => {
                                // Fallback: if default is present, map it to a Value dynamically
                                if let Some(def) = &exp.default {
                                    let val = match def {
                                        toml::Value::String(s) => Value::String(s.clone()),
                                        toml::Value::Integer(i) => Value::Number(*i as f32),
                                        toml::Value::Float(f) => Value::Number(*f as f32),
                                        toml::Value::Boolean(b) => Value::Boolean(*b),
                                        toml::Value::Array(arr) => Value::Array(
                                            arr.iter()
                                                .map(|v| match v {
                                                    toml::Value::String(s) => {
                                                        Value::String(s.clone())
                                                    }
                                                    toml::Value::Integer(i) => {
                                                        Value::Number(*i as f32)
                                                    }
                                                    toml::Value::Float(f) => {
                                                        Value::Number(*f as f32)
                                                    }
                                                    toml::Value::Boolean(b) => Value::Boolean(*b),
                                                    _ => Value::Null,
                                                })
                                                .collect(),
                                        ),
                                        toml::Value::Table(t) => {
                                            let mut m = std::collections::HashMap::new();
                                            for (k, v) in t.iter() {
                                                let vv = match v {
                                                    toml::Value::String(s) => {
                                                        Value::String(s.clone())
                                                    }
                                                    toml::Value::Integer(i) => {
                                                        Value::Number(*i as f32)
                                                    }
                                                    toml::Value::Float(f) => {
                                                        Value::Number(*f as f32)
                                                    }
                                                    toml::Value::Boolean(b) => Value::Boolean(*b),
                                                    _ => Value::Null,
                                                };
                                                m.insert(k.clone(), vv);
                                            }
                                            Value::Map(m)
                                        }
                                        _ => Value::Null,
                                    };
                                    if val != Value::Null {
                                        module
                                            .variable_table
                                            .set(format!("{}.{}", alias, exp.name), val);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Failed to load plugin {}: {}", plugin_name, e),
        }
    }
}
