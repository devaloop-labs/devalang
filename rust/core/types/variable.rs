use std::collections::HashMap;

use serde::Serialize;

use crate::core::types::token::Token;

#[derive(Debug, Clone, Serialize)]
pub enum VariableValue {
    Number(f32),
    Array(Vec<Token>),
    Map(HashMap<String, VariableValue>),
    Text(String),
    Boolean(bool),
    Unknown
}