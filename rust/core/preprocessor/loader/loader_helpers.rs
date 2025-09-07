use crate::core::preprocessor::loader::inject;
use crate::core::{
    lexer::{driver::Lexer, token::Token},
    parser::driver::parser::Parser,
    preprocessor::module::Module,
    store::global::GlobalStore,
};
use devalang_utils::logger::{LogLevel, Logger};
use devalang_utils::path::{normalize_path, resolve_relative_path};
use std::collections::HashMap;

pub fn load_module_recursively(
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
    for (bank_name, alias_opt) in inject::extract_bank_decls(&statements) {
        if let Err(e) = inject::inject_bank_triggers(&mut module, &bank_name, alias_opt) {
            eprintln!("Failed to inject bank triggers for '{}': {}", bank_name, e);
        }
    }

    for (plugin_name, alias) in inject::extract_plugin_uses(&statements) {
        inject::load_plugin_and_register(&mut module, &plugin_name, &alias, global_store);
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
    load_module_imports(&path, global_store);

    // Return tokens per module
    global_store
        .modules
        .iter()
        .map(|(p, m)| (p.clone(), m.tokens.clone()))
        .collect()
}

pub fn load_module_imports(path: &String, global_store: &mut GlobalStore) {
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
                if let crate::core::parser::statement::StatementKind::Import { source, .. } =
                    &stmt.kind
                {
                    Some(source.clone())
                } else {
                    None
                }
            })
            .collect()
    };

    for import_path in import_paths {
        let resolved = resolve_relative_path(path, &import_path);
        load_module_recursively(&resolved, global_store);
    }
}
