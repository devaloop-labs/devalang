pub mod handler;
pub mod token;

use std::fs;
use crate::core::{ lexer::{ handler::handle_content_lexing, token::Token }, utils::path::normalize_path };

pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    pub fn lex_tokens(&self, entrypoint: &str) -> Vec<Token> {
        let path = normalize_path(entrypoint);

        let file_content = fs
            ::read_to_string(&path)
            .expect("Failed to read the entrypoint file");

        let tokens = handle_content_lexing(file_content);

        tokens
    }
}
