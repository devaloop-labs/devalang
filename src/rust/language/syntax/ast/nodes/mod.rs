use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum DurationValue {
    Number(f32),
    Identifier(String),
    Beat(String),
    Beats(f32),
    Milliseconds(f32),
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Boolean(bool),
    Number(f32),
    Duration(DurationValue),
    Identifier(String),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Call { name: String, args: Vec<Value> },
    Block(Vec<Statement>),
    Sample(String),
    Beat(String),
    Statement(Box<Statement>),
    StatementKind(Box<StatementKind>),
    Midi(String),
    Range { start: Box<Value>, end: Box<Value> },
    Unknown,
    Null,
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
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
#[serde(tag = "kind")]
pub enum StatementKind {
    Tempo {
        value: f32,
        body: Option<Vec<Statement>>,
    },
    Print,
    Pattern {
        name: String,
        target: Option<String>,
    },
    Trigger {
        entity: String,
        duration: DurationValue,
        effects: Option<Value>,
    },
    Sleep,
    Call {
        name: String,
        args: Vec<Value>,
    },
    Load {
        source: String,
        alias: String,
    },
    Use {
        name: String,
        alias: Option<String>,
    },
    UsePlugin {
        author: String,
        name: String,
        alias: String,
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
    Assign {
        target: String,
        property: String,
    },
    Synth,
    Bank {
        name: String,
        alias: Option<String>,
    },
    Let {
        name: String,
        value: Option<Value>,
    },
    Var {
        name: String,
        value: Option<Value>,
    },
    Const {
        name: String,
        value: Option<Value>,
    },
    Group {
        name: String,
        body: Vec<Statement>,
    },
    Spawn {
        name: String,
        args: Vec<Value>,
    },
    Loop {
        count: Value,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Value,
        body: Vec<Statement>,
    },
    Routing {
        body: Vec<Statement>,
    },
    RoutingNode {
        name: String,
        alias: Option<String>,
    },
    RoutingFx {
        target: String,
        effects: Value,
    },
    RoutingRoute {
        source: String,
        destination: String,
        effects: Option<Value>,
    },
    RoutingDuck {
        source: String,
        destination: String,
        effect: Value,
    },
    RoutingSidechain {
        source: String,
        destination: String,
        effect: Value,
    },
    Bind {
        source: String,
        target: String,
    },
    FxPipeline {
        effects: Vec<Value>,
        subject: String,
    },
    Node {
        name: String,
    },
    Sidechain {
        source: String,
        effect: Value,
        target: Option<String>,
    },
    Include(String),
    Export {
        names: Vec<String>,
        source: String,
    },
    Import {
        names: Vec<String>,
        source: String,
    },
    On {
        event: String,
        args: Option<Vec<Value>>,
        body: Vec<Statement>,
    },
    Emit {
        event: String,
        payload: Option<Value>,
    },
    If {
        condition: Value,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>, // Can contain statements or another If for else if
    },
    Return {
        value: Option<Box<Value>>,
    },
    Break,
    Comment,
    Indent,
    Dedent,
    NewLine,
    Unknown,
    Error {
        message: String,
    },
}

impl Default for StatementKind {
    fn default() -> Self {
        StatementKind::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Statement {
    pub kind: StatementKind,
    #[serde(default)]
    pub value: Value,
    pub indent: usize,
    pub line: usize,
    pub column: usize,
}

impl Statement {
    pub fn new(
        kind: StatementKind,
        value: Value,
        indent: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            kind,
            value,
            indent,
            line,
            column,
        }
    }

    pub fn tempo(value: f32, line: usize, column: usize) -> Self {
        Self::new(
            StatementKind::Tempo { value, body: None },
            Value::Null,
            0,
            line,
            column,
        )
    }

    pub fn print(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::new(
            StatementKind::Print,
            Value::String(message.into()),
            0,
            line,
            column,
        )
    }

    pub fn trigger(
        entity: impl Into<String>,
        duration: DurationValue,
        effects: Option<Value>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::new(
            StatementKind::Trigger {
                entity: entity.into(),
                duration,
                effects,
            },
            Value::Null,
            0,
            line,
            column,
        )
    }

    pub fn unknown() -> Self {
        Self::default()
    }

    pub fn unknown_with_pos(indent: usize, line: usize, column: usize) -> Self {
        Self::new(StatementKind::Unknown, Value::Null, indent, line, column)
    }

    pub fn error_with_pos(indent: usize, line: usize, column: usize, message: String) -> Self {
        Self::new(
            StatementKind::Error { message },
            Value::Null,
            indent,
            line,
            column,
        )
    }
}
