use crate::core::lexer::{handler::driver::handle_content_lexing, token::Token};
use devalang_utils::path::normalize_path;
use std::fs;
use std::path::Path;

pub struct Lexer {}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    pub fn lex_from_source(&self, source: &str) -> Result<Vec<Token>, String> {
        handle_content_lexing(source.to_string())
    }

    pub fn lex_tokens(&self, entrypoint: &str) -> Result<Vec<Token>, String> {
        let path = normalize_path(entrypoint);
        let resolved_path = Self::resolve_entry_path(&path)?;

        let file_content = fs::read_to_string(&resolved_path).map_err(|e| {
            format!(
                "Failed to read the entrypoint file '{}': {}",
                resolved_path, e
            )
        })?;

        handle_content_lexing(file_content).map_err(|e| format!("Failed to lex the content: {}", e))
    }

    fn resolve_entry_path(path: &str) -> Result<String, String> {
        let candidate = Path::new(path);

        if candidate.is_dir() {
            let index_path = candidate.join("index.deva");
            if index_path.exists() {
                Ok(index_path.to_string_lossy().replace("\\", "/"))
            } else {
                Err(format!(
                    "Expected 'index.deva' in directory '{}', but it was not found",
                    path
                ))
            }
        } else if candidate.is_file() {
            return Ok(path.to_string());
        } else {
            return Err(format!(
                "Provided entrypoint '{}' is not a valid file or directory",
                path
            ));
        }
    }
}
