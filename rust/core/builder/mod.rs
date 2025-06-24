use crate::core::types::statement::{ StatementResolved };
use std::fs::File;
use std::io::Write;

pub fn build_ast(statements: &Vec<StatementResolved>) -> String {
    let mut ast_string = String::new();

    serde_json
        ::to_string_pretty(statements)
        .map(|json| {
            ast_string.push_str(&json);
        })
        .unwrap_or_else(|err| {
            eprintln!("Error serializing AST: {}", err);
            std::process::exit(1);
        });

    ast_string
}

pub fn write_ast_to_file(ast: &str, file_path: &str) {
    // Ensure the json directory exists and is cleared
    clear_json_directory(&file_path);
    create_json_directory(&file_path);

    let file_path = format!("{}/ast.json", file_path);

    let mut file = File::create(&file_path).expect("Unable to create AST file");
    file.write_all(ast.as_bytes()).expect("Unable to write AST to file");
}

fn clear_json_directory(path: &str) {
    std::fs::remove_dir_all(path);
}

fn create_json_directory(path: &str) {
    std::fs::create_dir_all(path);
}
