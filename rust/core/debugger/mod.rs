use std::collections::HashMap;
use crate::core::types::{ module::Module, statement::{ Statement, StatementResolved } };

pub struct Debugger {
    pub module: Module,
}

impl Debugger {
    pub fn new(module: &Module) -> Self {
        Debugger {
            module: module.clone(),
        }
    }

    pub fn run(&self) {
        println!(" ");

        println!("🔍 Running Debugger for module: {}", self.module.path);

        // Here you can add more functionality to run the debugger,
        // such as stepping through statements, inspecting variables, etc.
        println!("  - Total statements: {}", self.module.statements.len());
        println!("  - Variables: {:?}", self.module.variable_table.variables.len());
        println!("  - Exports: {:?}", self.module.export_table.exports.len());
        println!("  - Imports: {:?}", self.module.import_table.imports.len());

        println!(" ");
    }

    pub fn log_statements(&self) {
        println!("📜 Logging statements for module: {}", self.module.path);

        for stmt in &self.module.statements {
            println!("  - Statement: {:?}", stmt);
        }
    }

    pub fn log_resolved_statements(&self, statements: &Vec<StatementResolved>) {
        println!("📜 Logging resolved statements for module: {}", self.module.path);

        for stmt in statements {
            println!("  - Resolved Statement: {:?}", stmt);
        }
    }

    pub fn write_files(&self, path: &str, resolved_statements: Vec<StatementResolved>) {
        const LEXER_FILENAME: &str = "debug_lexer.log";
        const STATEMENTS_FILENAME: &str = "debug_statements.log";
        const AST_FILENAME: &str = "debug_ast.log";

        let lexer_path = format!("{}{}", path, LEXER_FILENAME);
        let statements_path = format!("{}{}", path, STATEMENTS_FILENAME);
        let ast_path = format!("{}{}", path, AST_FILENAME);

        println!("📝 Writing debug statements to: {}", path);

        // Collect debug information
        let tokens = self.module.tokens
            .iter()
            .map(|token| format!("{:?}", token))
            .collect::<Vec<String>>();
        let statements = resolved_statements
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect::<Vec<String>>();

        // Ensure the debug directory exists and is cleared
        clear_debug_directory(path);
        create_debug_directory(path);

        // Writing files
        write_tokens_to_file(&tokens, &lexer_path);
        write_statements_to_file(&statements, &statements_path);

        println!("✅ Debug files written successfully.");
    }
}

fn clear_debug_directory(path: &str) {
    if std::fs::remove_dir_all(path).is_err() {
        println!("⚠️ Could not clear debug directory: {}", path);
    } else {
        println!("✅ Debug directory cleared: {}", path);
    }
}

fn create_debug_directory(path: &str) {
    if std::fs::create_dir_all(path).is_err() {
        println!("⚠️ Could not create debug directory: {}", path);
    } else {
        println!("✅ Debug directory created: {}", path);
    }
}

fn write_statements_to_file(statements: &Vec<String>, path: &str) {
    let content = statements.join("\n");

    std::fs::write(path, content).expect("Unable to write statements to file");

    println!("✅ Statements written to file: {}", path);
}

fn write_tokens_to_file(tokens: &Vec<String>, path: &str) {
    let content = tokens.join("\n");

    std::fs::write(path, content).expect("Unable to write tokens to file");

    println!("✅ Tokens written to file: {}", path);
}
