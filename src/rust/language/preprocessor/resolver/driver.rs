use crate::language::preprocessor::resolver::value::resolve_value;
use crate::language::syntax::ast::{Statement, StatementKind, Value};
use std::collections::HashMap;

/// Main driver for statement resolution
pub fn resolve_statement(stmt: &Statement, variables: &HashMap<String, Value>) -> Statement {
    match &stmt.kind {
        StatementKind::Trigger {
            entity,
            duration,
            effects,
        } => crate::language::preprocessor::resolver::trigger::resolve_trigger(
            stmt, entity, duration, effects, variables,
        ),
        StatementKind::Let { name, value } => {
            let resolved_value = value.as_ref().map(|v| resolve_value(v, variables, 0));
            Statement::new(
                StatementKind::Let {
                    name: name.clone(),
                    value: resolved_value,
                },
                Value::Null,
                stmt.indent,
                stmt.line,
                stmt.column,
            )
        }
        StatementKind::Bank { name, alias } => {
            // Bank names might be identifiers - resolve them
            let resolved_name = match resolve_value(&Value::String(name.clone()), variables, 0) {
                Value::String(s) => s,
                Value::Identifier(s) => s,
                _ => name.clone(),
            };
            Statement::new(
                StatementKind::Bank {
                    name: resolved_name,
                    alias: alias.clone(),
                },
                Value::Null,
                stmt.indent,
                stmt.line,
                stmt.column,
            )
        }
        _ => {
            // For other statements, resolve the value field
            let resolved_value = resolve_value(&stmt.value, variables, 0);
            Statement {
                value: resolved_value,
                ..stmt.clone()
            }
        }
    }
}
