pub mod handler;
pub mod token;

use std::fs;
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

        let file_content = fs::read_to_string(&path).expect("Failed to read the entrypoint file");

        let tokens = handle_content_lexing(file_content).expect("Failed to lex the content");

        tokens
    }
}
