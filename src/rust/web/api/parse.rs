//! Parsing API for WASM
//!
//! Exposes Devalang parser to JavaScript.

use serde::Serialize;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

use crate::language::syntax::ast::{Statement, StatementKind, Value};
use crate::language::syntax::parser::driver::SimpleParser;

/// Parse result returned to JavaScript
#[derive(Serialize)]
pub struct ParseResult {
    pub success: bool,
    pub statements: Vec<StatementInfo>,
    pub errors: Vec<String>,
}

/// Simplified statement info for JS
#[derive(Serialize)]
pub struct StatementInfo {
    pub kind: String,
    pub line: usize,
}

/// Parse Devalang source code
///
/// # Arguments
/// * `entry_path` - Path to the source file (for error messages)
/// * `source` - Source code to parse
///
/// # Returns
/// JSON object with parse results
#[wasm_bindgen]
pub fn parse(entry_path: &str, source: &str) -> Result<JsValue, JsValue> {
    match SimpleParser::parse(source, PathBuf::from(entry_path)) {
        Ok(statements) => {
            let errors = collect_statement_errors(&statements);
            let statement_infos: Vec<StatementInfo> = statements
                .iter()
                .map(|stmt| StatementInfo {
                    kind: format!("{:?}", stmt.kind),
                    line: stmt.line,
                })
                .collect();

            let result = ParseResult {
                success: errors.is_empty(),
                statements: statement_infos,
                errors,
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => {
            let result = ParseResult {
                success: false,
                statements: vec![],
                errors: vec![format!("{}", e)],
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
    }
}

/// Quick parse check - returns true if code parses without errors
#[wasm_bindgen]
pub fn check_syntax(source: &str) -> bool {
    SimpleParser::parse(source, PathBuf::from("check.deva"))
        .map(|statements| collect_statement_errors(&statements).is_empty())
        .unwrap_or(false)
}

fn collect_statement_errors(statements: &[Statement]) -> Vec<String> {
    let mut errors = Vec::new();

    for statement in statements {
        collect_statement_errors_into(statement, &mut errors);
    }

    errors
}

fn collect_statement_errors_into(statement: &Statement, errors: &mut Vec<String>) {
    match &statement.kind {
        StatementKind::Unknown => match &statement.value {
            Value::String(message) => errors.push(message.clone()),
            _ => errors.push(format!("Unknown statement at line {}", statement.line)),
        },
        StatementKind::Error { message } => errors.push(message.clone()),
        StatementKind::Tempo {
            body: Some(body), ..
        }
        | StatementKind::Function { body, .. }
        | StatementKind::Group { body, .. }
        | StatementKind::Loop { body, .. }
        | StatementKind::For { body, .. }
        | StatementKind::Routing { body }
        | StatementKind::On { body, .. }
        | StatementKind::Section { body, .. }
        | StatementKind::While { body, .. } => {
            for child in body {
                collect_statement_errors_into(child, errors);
            }
        }
        StatementKind::If {
            body, else_body, ..
        } => {
            for child in body {
                collect_statement_errors_into(child, errors);
            }
            if let Some(else_body) = else_body {
                for child in else_body {
                    collect_statement_errors_into(child, errors);
                }
            }
        }
        StatementKind::Schedule { events } => {
            for (_, _, body) in events {
                for child in body {
                    collect_statement_errors_into(child, errors);
                }
            }
        }
        _ => {}
    }
}
