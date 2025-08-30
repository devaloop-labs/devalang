use serde::{Deserialize, Serialize};

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
        matches!(&self.kind, TokenKind::Error(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // ───── Keywords ─────
    At,
    Tempo,
    Bank,
    Loop,
    Function,
    As,
    On,
    Emit,

    // ───── Instruments ─────
    Synth,

    // ───── Literals ─────
    Identifier,
    Number,
    String,
    Boolean,
    Arrow,

    // ───── Structures ─────
    Map,
    Array,

    // ───── Symbols ─────
    Colon,
    Comma,
    Equals,
    Dot,
    Slash,
    Plus,
    Asterisk,
    Minus,

    // ───── Operators ─────
    DoubleEquals,
    NotEquals,
    GreaterEqual,
    LessEqual,
    Greater,
    Less,

    // ───── Brackets ─────
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    LParen,   // (
    RParen,   // )

    // ───── Quotes ─────
    Quote,   // '
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
    Whitespace,
    Unknown,
    Error(String),
    EOF,
}
