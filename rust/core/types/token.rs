use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn is_error(&self) -> bool {
        match &self.kind {
            TokenKind::Error(_) => {
                return true;
            },
            _ => {
                return false;
            },
        };
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenKind {
    At,
    Tempo,
    Bank,
    Loop,
    Identifier,
    Map,
    Array,
    Number,
    String,
    Boolean,
    Colon,
    Comma,
    Equals,
    DoubleEquals,
    Dot,
    LBrace,
    RBrace,
    DbQuote,
    Quote,
    LBracket,
    RBracket,
    Newline,
    Indent,
    Dedent,
    Comment(String),
    Unknown,
    Error(String),
    EOF,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenDuration {
    Number(f32),
    Identifier(String),
    Infinite,
    Auto,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokenParam {
    pub name: String,
    pub value: TokenParamValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenParamValue {
    Number(f32),
    String(String),
    Boolean(bool),
    Identifier(String),
    Map(HashMap<String, TokenParamValue>),
    Array(Vec<TokenParamValue>),
    Unknown,
}
