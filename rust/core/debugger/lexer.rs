use crate::core::{debugger::Debugger, lexer::token::Token};
use std::{collections::HashMap, fs::create_dir_all};

pub fn write_lexer_log_file(
    output_dir: &str,
    file_name: &str,
    modules: HashMap<String, Vec<Token>>,
) {
    let debugger = Debugger::new();
    let mut content = String::new();

    let log_directory = format!("{}/logs", output_dir);

    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (path, tokens) in modules {
        content.push_str(&format!("--- Resolved Tokens for {} ---\n", path));

        for token in tokens {
            content.push_str(&format!("{:?}\n", token));
        }

        content.push_str("\n");
    }

    debugger.write_log_file(&log_directory, file_name, &content);
}
