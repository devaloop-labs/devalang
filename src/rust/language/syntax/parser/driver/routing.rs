use crate::language::syntax::ast::{Statement, StatementKind, Value};
use anyhow::{Result, anyhow};

pub fn parse_routing_command<'a>(
    mut parts: impl Iterator<Item = &'a str>,
    line_number: usize,
) -> Result<Statement> {
    let source = parts
        .next()
        .ok_or_else(|| anyhow!("routing statement requires a source alias"))?
        .trim_end_matches(',')
        .to_string();
    let mut parent = None;

    while let Some(token) = parts.next() {
        match token {
            "->" => {
                let target = parts
                    .next()
                    .ok_or_else(|| anyhow!("routing statement missing target after '->'"))?;
                parent = Some(target.trim_matches(',').to_string());
                break;
            }
            other => {
                // Support simple "routing alias parent" form.
                parent = Some(other.trim_matches(',').to_string());
                break;
            }
        }
    }

    let mut map = std::collections::HashMap::new();
    map.insert("source".to_string(), Value::String(source));
    if let Some(target) = parent {
        map.insert("target".to_string(), Value::String(target));
    }

    Ok(Statement::new(
        StatementKind::Routing,
        Value::Map(map),
        0,
        line_number,
        1,
    ))
}
