pub mod preprocessor;
pub mod lexer;

use std::io::Write;

pub struct Debugger {}

impl Debugger {
    pub fn new() -> Self {
        Debugger {}
    }

    pub fn write_log_file(&self, path: &str, filename: &str, content: &str) {
        std::fs::create_dir_all(path).expect("Failed to create directory");
        let file_path = format!("{}/{}", path, filename);
        let mut file = std::fs::File::create(file_path).expect("Failed to create file");

        file.write_all(content.as_bytes()).expect("Failed to write to file");
    }
}
