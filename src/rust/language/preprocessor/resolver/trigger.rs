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
mod tests {
    use super::*;

    #[test]
    fn test_resolve_trigger_with_duration_identifier() {
        let mut vars = HashMap::new();
        vars.insert("short".to_string(), Value::Number(100.0));

        let stmt = Statement::new(
            StatementKind::Trigger {
                entity: "kick".to_string(),
                duration: DurationValue::Identifier("short".to_string()),
                effects: None,
            },
            Value::Null,
            0,
            1,
            1,
        );

        let resolved = resolve_trigger(
            &stmt,
            "kick",
            &DurationValue::Identifier("short".to_string()),
            &None,
            &vars,
        );

        if let StatementKind::Trigger { duration, .. } = &resolved.kind {
            assert!(
                matches!(duration, DurationValue::Milliseconds(ms) if (ms - 100.0).abs() < f32::EPSILON)
            );
        } else {
            panic!("Expected trigger statement");
        }
    }
}
