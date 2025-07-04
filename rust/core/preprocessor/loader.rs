use std::collections::HashMap;
use crate::{
    core::{
        error::ErrorHandler,
        lexer::{ token::Token, Lexer },
        parser::{ statement::{ Statement, StatementKind }, Parser },
        preprocessor::{
            module::Module,
            processor::process_modules,
            resolver::{ resolve_all_modules, resolve_and_flatten_all_modules },
        },
        store::global::GlobalStore,
        utils::path::normalize_path,
    },
    utils::logger::Logger,
};

pub struct ModuleLoader {
    pub entry: String,
    pub output: String,
}

impl ModuleLoader {
    pub fn new(entry: &str, output: &str) -> Self {
        Self {
            entry: entry.to_string(),
            output: output.to_string(),
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

        println!("Loading module from raw source: {}", normalized_entry_path);

        global_store.insert_module(normalized_entry_path.to_string(), module);

        Self {
            entry: normalized_entry_path.to_string(),
            output: output_path.to_string(),
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
        path: &str,
        global_store: &mut GlobalStore
    ) -> HashMap<String, Vec<Token>> {
        if global_store.modules.contains_key(path) {
            return HashMap::new();
        }

        let lexer = Lexer::new();
        let tokens = lexer.lex_tokens(path);

        let mut parser = Parser::new();
        parser.set_current_module(path.to_string());

        let statements = parser.parse_tokens(tokens.clone(), global_store);

        // SECTION Error handling
        let mut error_handler = ErrorHandler::new();
        error_handler.detect_from_statements(&mut parser, &statements);

        error_handler.has_errors().then(|| {
            let logger = Logger::new();
            let errors = error_handler.get_errors();

            for error in errors {
                let stacktrace = format!("{}:{}:{}", path, error.line, error.column);
                logger.log_error_with_stacktrace(&error.message, &stacktrace);
            }
        });

        // SECTION Module creation
        let mut module = Module::new(path);
        module.tokens = tokens.clone();
        module.statements = statements.clone();

        global_store.insert_module(path.to_string(), module);

        // Then load the imports recursively
        self.load_module_imports(&path.to_string(), global_store);

        // Return all tokens by module
        let mut tokens_by_module = HashMap::new();

        global_store.modules.iter().for_each(|(path, module)| {
            tokens_by_module.insert(path.clone(), module.tokens.clone());
        });

        tokens_by_module
    }

    #[cfg(feature = "cli")]
    fn load_module_imports(&self, path: &String, global_store: &mut GlobalStore) {
        let imports = global_store.modules
            .get(path)
            .unwrap()
            .statements.iter()
            .filter_map(|stmt| {
                if let StatementKind::Import { source, .. } = &stmt.kind {
                    Some(source.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for import_path in imports {
            self.load_module_recursively(&import_path, global_store);
        }
    }
}
