use std::collections::HashMap;
use serde::{ Deserialize, Serialize };

use crate::core::parser::statement::Statement;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f32),
    Identifier(String),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Block(Vec<Statement>),
    Sample(String),
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