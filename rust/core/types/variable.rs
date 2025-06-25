use std::collections::HashMap;

use serde::Serialize;

use crate::core::types::token::{Token, TokenParamValue};

#[derive(Debug, Clone, Serialize)]
pub enum VariableValue {
    Number(f32),
    Array(Vec<Token>),
    Map(HashMap<String, TokenParamValue>),
    Text(String),
    Boolean(bool),
    Sample(String),
    Unknown,
    Null,
}
pub struct Variable {
    pub value: VariableValue,
}

impl Variable {
    pub fn from_number(value: f32) -> Self {
        Variable {
            value: VariableValue::Number(value),
        }
    }

    pub fn from_token(value: Token) -> Self {
        Variable {
            value: VariableValue::Text(value.lexeme),
        }
    }
}
