use crate::language::preprocessor::resolver::value::resolve_value;
use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use std::collections::HashMap;

/// Resolve trigger statement: expand duration and effect identifiers
pub fn resolve_trigger(
    stmt: &Statement,
    entity: &str,
    duration: &DurationValue,
    effects: &Option<Value>,
    variables: &HashMap<String, Value>,
) -> Statement {
    let final_duration = resolve_duration(duration, variables);
    let final_effects = effects.as_ref().map(|e| resolve_value(e, variables, 0));

    Statement::new(
        StatementKind::Trigger {
            entity: entity.to_string(),
            duration: final_duration,
            effects: final_effects,
        },
        Value::Null,
        stmt.indent,
        stmt.line,
        stmt.column,
    )
}

fn resolve_duration(duration: &DurationValue, variables: &HashMap<String, Value>) -> DurationValue {
    match duration {
        DurationValue::Identifier(name) => {
            if let Some(val) = variables.get(name) {
                match val {
                    Value::Number(n) => DurationValue::Milliseconds(*n),
                    Value::Identifier(s) if s == "auto" => DurationValue::Auto,
                    Value::String(s) if s.ends_with("ms") => {
                        if let Ok(ms) = s.trim_end_matches("ms").parse::<f32>() {
                            return DurationValue::Milliseconds(ms);
                        }
                        DurationValue::Identifier(name.clone())
                    }
                    _ => DurationValue::Identifier(name.clone()),
                }
            } else {
                DurationValue::Identifier(name.clone())
            }
        }
        other => other.clone(),
    }
}

#[cfg(test)]
#[path = "test_trigger.rs"]
mod tests;
