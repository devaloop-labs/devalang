use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use anyhow::{Result, anyhow};

pub fn parse_trigger_line(line: &str, line_number: usize) -> Result<Statement> {
    let mut parts = line.trim_start_matches('.').trim().split_whitespace();
    let entity = parts
        .next()
        .ok_or_else(|| anyhow!("trigger requires a target"))?
        .to_string();

    let mut duration = DurationValue::Auto;
    if let Some(token) = parts.next() {
        if token.eq_ignore_ascii_case("for") {
            if let Some(value) = parts.next() {
                duration =
                    crate::language::syntax::parser::driver::duration::parse_duration_token(value)?;
            }
        } else {
            duration =
                crate::language::syntax::parser::driver::duration::parse_duration_token(token)?;
        }
    }

    Ok(Statement::new(
        StatementKind::Trigger {
            entity,
            duration,
            effects: None,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
