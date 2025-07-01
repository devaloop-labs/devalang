use std::collections::HashMap;

use crate::{
    core::{
        debugger::{ lexer::write_lexer_log_file, preprocessor::write_preprocessor_log_file },
        error::ErrorHandler,
        lexer::Lexer,
        parser::{ statement::{ Statement, StatementKind }, Parser },
        preprocessor::{
            module::Module,
            processor::process_modules,
            resolver::{ resolve_all_modules, resolve_and_flatten_all_modules },
        },
        store::global::GlobalStore,
    },
    utils::logger::{ LogLevel, Logger },
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

    pub fn load_all(&self, global_store: &mut GlobalStore) -> HashMap<String, Vec<Statement>> {
        // SECTION Load the entry module and its dependencies
        self.load_module_recursively(&self.entry, global_store);

        // SECTION Process and resolve modules
        process_modules(self, global_store);
        resolve_all_modules(self, global_store);

        let resolved = resolve_and_flatten_all_modules(global_store);

        // SECTION Write resolved statements to log file
        write_preprocessor_log_file(&self.output, "resolved_statements.log", resolved.clone());

        // Return the resolved statements
        resolved
    }

    fn load_module_recursively(&self, path: &str, global_store: &mut GlobalStore) {
        if global_store.modules.contains_key(path) {
            return;
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
        module.tokens = tokens;
        module.statements = statements;

        global_store.insert_module(path.to_string(), module);

        // Then load the imports recursively
        self.load_module_imports(&path.to_string(), global_store);
    }

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
