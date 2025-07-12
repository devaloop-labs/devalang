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
    // ───── Keywords ─────
    At,
    Tempo,
    Bank,
    Loop,

    // ───── Literals ─────
    Identifier,
    Number,
    String,
    Boolean,

    // ───── Structures ─────
    Map,
    Array,

    // ───── Symbols ─────
    Colon,
    Comma,
    Equals,
    Dot,

    // ───── Operators ─────
    DoubleEquals,
    NotEquals,
    GreaterEqual,
    LessEqual,
    Greater,
    Less,

    // ───── Brackets ─────
    LBrace, // {
    RBrace, // }
    LBracket, // [
    RBracket, // ]

    // ───── Quotes ─────
    Quote, // '
    DbQuote, // "

    // ───── Formatting ─────
    Newline,
    Indent,
    Dedent,
    Comment,

    // ───── Conditions ─────
    If,
    Else,
    ElseIf,

    // ───── Special / Internal ─────
    Unknown,
    Error(String),
    EOF,
}
