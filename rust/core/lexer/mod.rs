pub mod handler;
pub mod token;

use std::fs;
use std::path::{Path, PathBuf};
use crate::core::{
    lexer::{ handler::driver::handle_content_lexing, token::Token },
    utils::path::normalize_path,
};

pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    pub fn lex_from_source(&self, source: &str) -> Result<Vec<Token>, String> {
        handle_content_lexing(source.to_string())
    }

    pub fn lex_tokens(&self, entrypoint: &str) -> Vec<Token> {
        let path = normalize_path(entrypoint);
        let resolved_path = Self::resolve_entry_path(&path);

        let file_content =
            fs::read_to_string(&resolved_path).expect("Failed to read the entrypoint file");

        handle_content_lexing(file_content).expect("Failed to lex the content")
    }

    fn resolve_entry_path(path: &str) -> String {
        let candidate = Path::new(path);

        if candidate.is_dir() {
            let index_path = candidate.join("index.deva");
            if index_path.exists() {
                return index_path.to_string_lossy().replace("\\", "/");
            } else {
                panic!(
                    "Expected 'index.deva' in directory '{}', but it was not found",
                    path
                );
            }
        } else if candidate.is_file() {
            return path.to_string();
        } else {
            panic!("Provided entrypoint '{}' is not a valid file or directory", path);
        }
    }
}
