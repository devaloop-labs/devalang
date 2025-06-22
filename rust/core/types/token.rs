use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenKind {
    // Keyword(String),
    At,
    Tempo,
    Bank,
    Loop,
    Identifier,
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
    Error(String),
    EOF,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenDuration {
    Number(f32),
    Identifier(String),
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
}
