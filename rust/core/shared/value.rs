use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{
    parser::statement::{Statement, StatementKind},
    shared::duration::Duration,
};

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
