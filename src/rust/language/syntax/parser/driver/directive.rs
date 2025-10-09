use crate::language::syntax::ast::{Statement, StatementKind, Value};
use anyhow::{Result, anyhow};
use std::path::Path;

pub fn parse_directive(
    line: &str,
    line_number: usize,
    file_path: &std::path::Path,
) -> Result<Statement> {
    // Handle @use directive for plugins: @use author.plugin as alias
    if line.starts_with("@use") {
        return parse_use_directive(line, line_number);
    }

    if !line.starts_with("@load") {
        // handle @import and @export directives
        if line.starts_with("@import") {
            // syntax: @import { a, b } from "path"
            let rest = line["@import".len()..].trim();
            if let Some(open) = rest.find('{') {
                if let Some(close) = rest.find('}') {
                    let names = rest[open + 1..close]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>();
                    let after = rest[close + 1..].trim();
                    let from_prefix = "from";
                    if after.starts_with(from_prefix) {
                        let path_part = after[from_prefix.len()..].trim();
                        let raw = path_part.trim().trim_matches('"');
                        // Resolve relative to file_path
                        let base = file_path
                            .parent()
                            .unwrap_or_else(|| std::path::Path::new("."));
                        let joined = base.join(raw);
                        let path = joined.to_string_lossy().to_string();
                        return Ok(Statement::new(
                            StatementKind::Import {
                                names,
                                source: path,
                            },
                            Value::Null,
                            0,
                            line_number,
                            1,
                        ));
                    }
                }
            }
        }

        if line.starts_with("@export") {
            // syntax: @export { a, b }
            let rest = line["@export".len()..].trim();
            if let Some(open) = rest.find('{') {
                if let Some(close) = rest.find('}') {
                    let names = rest[open + 1..close]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>();
                    return Ok(Statement::new(
                        StatementKind::Export {
                            names,
                            source: String::new(),
                        },
                        Value::Null,
                        0,
                        line_number,
                        1,
                    ));
                }
            }
        }

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
            let raw = rest[1..1 + end].to_string();
            // Resolve relative paths relative to the file location
            let base = file_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let joined = base.join(&raw);
            path = joined.to_string_lossy().to_string();
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

/// Parse @use directive for plugins: @use author.plugin as alias
/// Example: @use devaloop.acid as acid
fn parse_use_directive(line: &str, line_number: usize) -> Result<Statement> {
    let rest = line["@use".len()..].trim();
    if rest.is_empty() {
        return Err(anyhow!(
            "@use directive requires a plugin reference (author.plugin)"
        ));
    }

    // Parse plugin reference (author.plugin)
    let plugin_ref;
    let mut parts = rest.splitn(2, " as ");
    plugin_ref = parts.next().unwrap_or_default().trim().to_string();

    if !plugin_ref.contains('.') {
        return Err(anyhow!("@use directive requires format: author.plugin"));
    }

    let plugin_parts: Vec<&str> = plugin_ref.split('.').collect();
    if plugin_parts.len() != 2 {
        return Err(anyhow!(
            "@use directive requires format: author.plugin (got: {})",
            plugin_ref
        ));
    }

    let author = plugin_parts[0].to_string();
    let plugin_name = plugin_parts[1].to_string();

    // Parse alias if provided
    let alias = if let Some(alias_part) = parts.next() {
        alias_part.trim().to_string()
    } else {
        plugin_name.clone()
    };

    Ok(Statement::new(
        StatementKind::UsePlugin {
            author,
            name: plugin_name,
            alias,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
