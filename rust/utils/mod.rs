pub mod version;
pub mod signature;
pub mod spinner;
pub mod watcher;
pub mod file;
pub mod logger;

use crate::core::parser::statement::{Statement, StatementKind};
use crate::core::error::ErrorResult;
use crate::core::shared::value::Value;

pub fn collect_errors_recursively(statements: &[Statement]) -> Vec<ErrorResult> {
    let mut errors: Vec<ErrorResult> = Vec::new();

    for stmt in statements {
        match &stmt.kind {
            StatementKind::Unknown => {
                errors.push(ErrorResult {
                    message: format!("Unknown statement at line {}:{}", stmt.line, stmt.column),
                    line: stmt.line,
                    column: stmt.column,
                });
            }
            StatementKind::Error { message } => {
                errors.push(ErrorResult {
                    message: message.clone(),
                    line: stmt.line,
                    column: stmt.column,
                });
            }
            StatementKind::Loop => {
                if let Some(body_statements) = extract_loop_body_statements(&stmt.value) {
                    errors.extend(collect_errors_recursively(body_statements));
                }
            }
            _ => {}
        }
    }

    errors
}

pub fn extract_loop_body_statements(value: &Value) -> Option<&[Statement]> {
    if let Value::Map(map) = value {
        if let Some(Value::Block(statements)) = map.get("body") {
            return Some(statements);
        }
    }
    None
}