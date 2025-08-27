use crate::core::{
    error::ErrorResult,
    parser::statement::{Statement, StatementKind},
    shared::value::Value,
};

/// Recursively collects errors from a list of statements.
///
/// This function traverses the provided statements and aggregates any
/// `Unknown` or explicit `Error` statements into a flat vector.
/// It also descends into loop bodies to ensure nested errors are
/// surfaced.
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

fn extract_loop_body_statements(value: &Value) -> Option<&[Statement]> {
    if let Value::Map(map) = value {
        if let Some(Value::Block(statements)) = map.get("body") {
            return Some(statements);
        }
    }
    None
}