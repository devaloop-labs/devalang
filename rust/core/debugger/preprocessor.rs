use std::collections::HashMap;
use crate::core::{ debugger::Debugger, parser::statement::Statement };

pub fn write_preprocessor_log_file(
    output_dir: &str,
    file_name: &str,
    statements: HashMap<String, Vec<Statement>>
) {
    let debugger = Debugger::new();
    let mut content = String::new();

    for (path, stmts) in statements {
        content.push_str(&format!("--- Resolved Statements for {} ---\n", path));
        
        for stmt in stmts {
            content.push_str(&format!("{:?}\n", stmt));
        }

        content.push_str("\n");
    }

    debugger.write_log_file(output_dir, file_name, &content);
}
