use crate::core::{
    lexer::token::Token,
    shared::{duration::Duration, value::Value},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub value: Value,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

impl Statement {
    pub fn unknown() -> Self {
        Statement {
            kind: StatementKind::Unknown,
            value: Value::Null,
            indent: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn unknown_from_token(token: &Token) -> Self {
        Statement {
            kind: StatementKind::Unknown,
            value: Value::Null,
            indent: token.indent,
            line: token.line,
            column: token.column,
        }
    }

    pub fn error(token: Token, message: String) -> Self {
        Statement {
            kind: StatementKind::Error { message },
            value: Value::Null,
            indent: token.indent,
            line: token.line,
            column: token.column,
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum StatementKind {
    // ───── Core Instructions ─────
    Tempo,
    Bank {
        alias: Option<String>,
    },
    Print,
    Load {
        source: String,
        alias: String,
    },
    Use {
        name: String,
        alias: Option<String>,
    },
    Let {
        name: String,
    },
    Automate {
        target: String,
    },
    ArrowCall {
        target: String,
        method: String,
        args: Vec<Value>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },

    // ───── Instruments ─────
    Synth,

    // ───── Playback / Scheduling ─────
    Trigger {
        entity: String,
        duration: Duration,
        effects: Option<Value>,
    },
    Sleep,
    Call {
        name: String,
        args: Vec<Value>,
    },
    Spawn {
        name: String,
        args: Vec<Value>,
    },
    Loop,

    // ───── Structure & Logic ─────
    Group,

    // ───── Module System ─────
    Include(String),
    Export {
        names: Vec<String>,
        source: String,
    },
    Import {
        names: Vec<String>,
        source: String,
    },

    // ───── Conditions ─────
    If,
    Else,
    ElseIf,

    // ───── Internal / Utility ─────
    Comment,
    Indent,
    Dedent,
    NewLine,

    // ───── Events / Live coding ─────
    On {
        event: String,
        args: Option<Vec<Value>>,
        body: Vec<Statement>,
    },
    Emit {
        event: String,
        payload: Option<Value>,
    },

    // ───── Error Handling ─────
    Unknown,
    Error {
        message: String,
    },
}
