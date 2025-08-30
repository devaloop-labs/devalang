use crate::core::{debugger::Debugger, parser::statement::Statement};
use std::{collections::HashMap, fs::create_dir_all};

pub fn write_preprocessor_log_file(
    output_dir: &str,
    file_name: &str,
    modules: HashMap<String, Vec<Statement>>,
) {
    let debugger = Debugger::new();
    let mut content = String::new();

    let log_directory = format!("{}/logs", output_dir);

    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (path, stmts) in modules {
        content.push_str(&format!("--- Resolved Statements for {} ---\n", path));

        for stmt in stmts {
            content.push_str(&format!("{:?}\n", stmt));
        }

        content.push('\n');
    }

    debugger.write_log_file(&log_directory, file_name, &content);
}
