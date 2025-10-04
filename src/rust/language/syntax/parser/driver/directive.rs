use crate::language::syntax::ast::{Statement, StatementKind, Value};
use anyhow::{Result, anyhow};
use std::path::Path;

pub fn parse_directive(line: &str, line_number: usize) -> Result<Statement> {
    if !line.starts_with("@load") {
        return Ok(Statement::new(
            StatementKind::Unknown,
            Value::String(line.to_string()),
            0,
            line_number,
            1,
        ));
    }

    let mut rest = line["@load".len()..].trim();
    if rest.is_empty() {
        return Err(anyhow!("@load directive requires a path"));
    }

    let path;
    if rest.starts_with('"') {
        if let Some(end) = rest[1..].find('"') {
            path = rest[1..1 + end].to_string();
            rest = rest[1 + end + 1..].trim();
        } else {
            return Err(anyhow!("unterminated string literal in @load"));
        }
    } else {
        let mut split = rest.splitn(2, char::is_whitespace);
        path = split.next().unwrap_or_default().trim().to_string();
        rest = split.next().unwrap_or("").trim();
    }

    let alias = if rest.starts_with("as ") {
        rest[3..].trim().to_string()
    } else {
        Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone())
    };

    Ok(Statement::new(
        StatementKind::Load {
            source: path,
            alias,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
