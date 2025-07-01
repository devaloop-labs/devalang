use crate::core::parser::statement::Statement;
use std::{ collections::HashMap, fs::create_dir_all };
use std::io::Write;

pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Builder {}
    }

    pub fn build_ast(&self, modules: &HashMap<String, Vec<Statement>>) {
        let output_path = "./output";

        for (name, statements) in modules {
            let formatted_name = name.split("/").last().unwrap_or(name);
            let formatted_name = formatted_name.replace(".deva", "");

            create_dir_all(format!("{}/ast", output_path)).expect("Failed to create AST directory");

            let file_path = format!("{}/ast/{}.json", output_path, formatted_name);
            let mut file = std::fs::File::create(file_path).expect("Failed to create AST file");

            let content = serde_json
                ::to_string_pretty(&statements)
                .expect("Failed to serialize AST");

            file.write_all(content.as_bytes()).expect("Failed to write AST to file");
        }
    }
}
