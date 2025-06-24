use crate::core::types::{ module::Module, statement::{ StatementResolved } };

pub struct Debugger {
    pub module: Module,
}

impl Debugger {
    pub fn new(module: &Module) -> Self {
        Debugger {
            module: module.clone(),
        }
    }

    pub fn write_files(&self, output_dir: &str, resolved_statements: Vec<StatementResolved>) {
        const LEXER_FILENAME: &str = "debug_lexer.log";
        const STATEMENTS_FILENAME: &str = "debug_statements.log";

        let lexer_path = format!("{}{}", output_dir, LEXER_FILENAME);
        let statements_path = format!("{}{}", output_dir, STATEMENTS_FILENAME);

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
        clear_debug_directory(output_dir);
        create_debug_directory(output_dir);

        // Writing files
        write_tokens_debug_to_file(&tokens, &lexer_path);
        write_statements_debug_to_file(&statements, &statements_path);
    }
}

fn clear_debug_directory(path: &str) {
    std::fs::remove_dir_all(path);
}

fn create_debug_directory(path: &str) {
    std::fs::create_dir_all(path);
}

fn write_statements_debug_to_file(statements: &Vec<String>, path: &str) {
    let content = statements.join("\n");
    std::fs::write(path, content).expect("Unable to write statements to file");
}

fn write_tokens_debug_to_file(tokens: &Vec<String>, path: &str) {
    let content = tokens.join("\n");
    std::fs::write(path, content).expect("Unable to write tokens to file");
}
