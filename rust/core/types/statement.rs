use std::collections::HashMap;

use serde::Serialize;

use crate::core::types::{ token::{ Token, TokenDuration, TokenParam }, variable::VariableValue };

#[derive(Debug, Clone, Serialize)]
pub struct Statement {
    pub kind: StatementKind,
    pub value: VariableValue,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementResolved {
    pub kind: StatementKind,
    pub value: StatementResolvedValue,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatementValue {
    Boolean(bool),
    Number(f32),
    String(String),
    Array(Vec<Statement>),
    Map(HashMap<String, VariableValue>),
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatementResolvedValue {
    Boolean(bool),
    Number(f32),
    String(String),
    Array(Vec<StatementResolved>),
    Map(HashMap<String, StatementResolvedValue>),
    Unknown,
    Null,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatementIterator {
    Identifier(String),
    Number(f32),
    Array(Vec<Statement>),
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
    Loop {
        iterator: StatementIterator,
    },

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
    Export,
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
    Error,

    // Empty or ignored statements
    Comment(String),
    Indent,
    Dedent,
    NewLine,
}
