use std::collections::HashMap;

use serde::Serialize;

use crate::core::types::{token::{Token, TokenDuration, TokenParam}, variable::VariableValue};

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub value: VariableValue,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatementValue {
    Boolean(bool),
    Number(f32),
    String(String),
    Array(Vec<Token>),
    Map(HashMap<String, VariableValue>),
    Unknown,
}

#[derive(Debug, Serialize, Clone)]
/// Represents the kind of a statement
pub enum StatementKind {
    // Trigger statements
    Trigger {
        entity: String,
        duration: TokenDuration,
        // params: Vec<TokenParam>,
    },

    // Variable statements
    Let {
        name: String,
    },

    // Loop statements
    // Loop {
    //     iterator: Vec<Token>,
    //     body: Vec<Statement>,
    // },

    // Conditional statements
    // If {
    //     // condition: ConditionParts,
    //     condition_state: bool,
    //     body: Vec<Statement>,
    // },

    // Keyword statements
    Tempo(f32),
    Bank,

    // At (@) statements
    Include(String),
    Export,
    Import {
        names: Vec<String>,
        source: String,
    },
    Define(String),

    // Error & Unknown statements
    Unknown(String),
    Error {
        message: String,
        line: usize,
        column: usize,
    },

    // Empty or ignored statements
    Comment(String),
    Indent,
    Dedent,
    NewLine,
}
