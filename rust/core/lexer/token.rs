use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            }
            _ => {
                return false;
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Comment,
    Unknown,
    Error(String),
    EOF,
}
