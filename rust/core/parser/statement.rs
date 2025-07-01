use serde::{ Deserialize, Serialize };

use crate::core::{ lexer::token::Token, shared::{ duration::Duration, value::Value } };

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
/// Represents the kind of a statement
pub enum StatementKind {
    // Trigger statements
    Trigger {
        entity: String,
        duration: Duration,
        // params: Vec<TokenParam>,
    },

    // Variable statements
    Let {
        name: String,
    },

    // Loop statements
    Loop,

    // Conditional statements
    // If {
    //     // condition: ConditionParts,
    //     condition_state: bool,
    //     body: Vec<Statement>,
    // },

    // Keyword statements
    Tempo,
    Bank,

    // At (@) statements
    Include(String),
    Export {
        names: Vec<String>,
        source: String,
    },
    Import {
        names: Vec<String>,
        source: String,
    },
    Load {
        source: String,
        alias: String,
    },

    // Error & Unknown statements
    Unknown,
    Error {
        message: String,
    },

    // Empty or ignored statements
    Comment,
    Indent,
    Dedent,
    NewLine,
}
