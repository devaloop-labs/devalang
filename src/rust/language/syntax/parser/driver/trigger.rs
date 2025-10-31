use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use crate::language::syntax::parser::driver::effects::parse_chained_effects;
use anyhow::{Result, anyhow};

pub fn parse_trigger_line(line: &str, line_number: usize) -> Result<Statement> {
    // Split by arrow operator to separate trigger definition from effects chain
    let parts: Vec<&str> = line.split("->").collect();
    let trigger_def = parts[0].trim();

    // Parse base trigger definition
    let mut base_parts = trigger_def
        .trim_start_matches('.')
        .trim()
        .split_whitespace();
    let entity = base_parts
        .next()
        .ok_or_else(|| anyhow!("trigger requires a target"))?
        .to_string();

    let mut duration = DurationValue::Auto;
    if let Some(token) = base_parts.next() {
        if token.eq_ignore_ascii_case("for") {
            if let Some(value) = base_parts.next() {
                duration =
                    crate::language::syntax::parser::driver::duration::parse_duration_token(value)?;
            }
        } else {
            duration =
                crate::language::syntax::parser::driver::duration::parse_duration_token(token)?;
        }
    }

    // Parse effects chain if present
    let effects = if parts.len() > 1 {
        Some(parse_chained_effects(&parts[1..].join("->"))?)
    } else {
        None
    };

    Ok(Statement::new(
        StatementKind::Trigger {
            entity,
            duration,
            effects,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
