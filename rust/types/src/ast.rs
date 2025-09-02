use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Duration {
    Number(f32),
    Identifier(String),
    Beat(String),
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Warning,
    Critical,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StackFrame {
    pub module: Option<String>,
    pub context: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorResult {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub stack: Vec<StackFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    Pattern {
        name: String,
        target: Option<String>,
    },

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f32),
    Duration(Duration),
    Identifier(String),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Block(Vec<Statement>),
    Sample(String),
    Beat(String),
    Statement(Box<Statement>),
    StatementKind(Box<StatementKind>),
    Unknown,
    Null,
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Map(map) = self {
            map.get(key)
        } else {
            None
        }
    }
}

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

    pub fn unknown_with_pos(indent: usize, line: usize, column: usize) -> Self {
        Statement {
            kind: StatementKind::Unknown,
            value: Value::Null,
            indent,
            line,
            column,
        }
    }

    pub fn error_with_pos(indent: usize, line: usize, column: usize, message: String) -> Self {
        Statement {
            kind: StatementKind::Error { message },
            value: Value::Null,
            indent,
            line,
            column,
        }
    }
}
