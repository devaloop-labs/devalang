use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keyword {
    At,
    Tempo,
    Bank,
    Pattern,
    Loop,
    Function,
    As,
    On,
    Emit,
    Synth,
    Use,
    Let,
    Automate,
    Trigger,
    Sleep,
    Call,
    Spawn,
    Group,
    Include,
    Export,
    Import,
    Routing,
    Bind,
    Fx,
    Node,
    Sidechain,
    If,
    Else,
    ElseIf,
    Print,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier,
    Number,
    String,
    Boolean,
    Duration,
    Arrow,
    Colon,
    Comma,
    Equals,
    Dot,
    Slash,
    Plus,
    Asterisk,
    Minus,
    DoubleEquals,
    NotEquals,
    GreaterEqual,
    LessEqual,
    Greater,
    Less,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Newline,
    Indent,
    Dedent,
    Comment,
    Eof,
    Unknown,
    Error(String),
}

impl TokenKind {
    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        matches!(self, TokenKind::Keyword(k) if *k == keyword)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: impl Into<String>,
        indent: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            indent,
            line,
            column,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind, TokenKind::Error(_))
    }
}

pub fn keyword_from_ident(ident: &str) -> Option<Keyword> {
    match ident.to_lowercase().as_str() {
        "at" => Some(Keyword::At),
        "bpm" | "tempo" => Some(Keyword::Tempo),
        "bank" => Some(Keyword::Bank),
        "pattern" => Some(Keyword::Pattern),
        "loop" => Some(Keyword::Loop),
        "fn" | "function" => Some(Keyword::Function),
        "as" => Some(Keyword::As),
        "on" => Some(Keyword::On),
        "emit" => Some(Keyword::Emit),
        "synth" => Some(Keyword::Synth),
        "use" => Some(Keyword::Use),
        "let" => Some(Keyword::Let),
        "automate" => Some(Keyword::Automate),
        "trigger" => Some(Keyword::Trigger),
        "sleep" | "rest" | "wait" => Some(Keyword::Sleep),
        "call" => Some(Keyword::Call),
        "spawn" => Some(Keyword::Spawn),
        "group" => Some(Keyword::Group),
        "include" => Some(Keyword::Include),
        "export" => Some(Keyword::Export),
        "import" => Some(Keyword::Import),
        "routing" => Some(Keyword::Routing),
        "bind" => Some(Keyword::Bind),
        "fx" | "pipeline" => Some(Keyword::Fx),
        "node" => Some(Keyword::Node),
        "sidechain" => Some(Keyword::Sidechain),
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "elseif" | "else_if" | "elif" => Some(Keyword::ElseIf),
        "print" => Some(Keyword::Print),
        _ => None,
    }
}
