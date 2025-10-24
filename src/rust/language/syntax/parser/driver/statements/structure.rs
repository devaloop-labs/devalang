use super::super::helpers::{parse_array_value, parse_condition};
use crate::language::syntax::ast::{Statement, StatementKind, Value};
/// Structure statement parsing: group, pattern, loop, for, if, on, emit, call, spawn
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::iter::Iterator;

/// Parse pattern statement
/// Supports:
/// - pattern name with target = "pattern"
/// - pattern name with target { options } = "pattern"
pub fn parse_pattern(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("pattern requires a name"))?
        .as_ref()
        .to_string();

    let mut target = None;
    let mut pattern_str = None;
    let mut options: HashMap<String, Value> = HashMap::new();

    // Check for "with" keyword
    if let Some(word) = parts.next() {
        if word.as_ref() == "with" {
            // Next is the target
            target = parts.next().map(|v| v.as_ref().to_string());

            // Collect remaining parts to check for options block or pattern
            let rest: Vec<String> = parts.map(|s| s.as_ref().to_string()).collect();
            let joined = rest.join(" ");

            // Check if there's an options block: { key: value, key: value }
            if let Some(brace_start) = joined.find('{') {
                if let Some(brace_end) = joined.rfind('}') {
                    // Extract options block
                    let options_str = &joined[brace_start + 1..brace_end];

                    // Parse options (key: value pairs)
                    for pair in options_str.split(',') {
                        let parts: Vec<&str> = pair.split(':').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim().to_string();
                            let value_str = parts[1].trim();

                            // Parse value
                            let value = if let Ok(num) = value_str.parse::<f32>() {
                                Value::Number(num)
                            } else if value_str == "true" {
                                Value::Boolean(true)
                            } else if value_str == "false" {
                                Value::Boolean(false)
                            } else if value_str.starts_with('"') && value_str.ends_with('"') {
                                Value::String(value_str.trim_matches('"').to_string())
                            } else {
                                Value::Identifier(value_str.to_string())
                            };

                            options.insert(key, value);
                        }
                    }

                    // Check for "=" and pattern after the brace
                    let after_brace = joined[brace_end + 1..].trim();
                    if let Some(eq_pos) = after_brace.find('=') {
                        let pattern_part = after_brace[eq_pos + 1..].trim();
                        pattern_str = Some(pattern_part.trim_matches('"').to_string());
                    }
                } else {
                    return Err(anyhow!("Unclosed brace in pattern options"));
                }
            } else {
                // No options block, check for "=" and pattern directly
                if let Some(eq_pos) = joined.find('=') {
                    let pattern_part = joined[eq_pos + 1..].trim();
                    pattern_str = Some(pattern_part.trim_matches('"').to_string());
                }
            }
        }
    }

    // Store pattern string and options in value
    let value = if !options.is_empty() {
        let mut map = options;
        if let Some(pat) = pattern_str {
            map.insert("pattern".to_string(), Value::String(pat));
        }
        Value::Map(map)
    } else {
        pattern_str.map(Value::String).unwrap_or(Value::Null)
    };

    Ok(Statement::new(
        StatementKind::Pattern { name, target },
        value,
        0,
        line_number,
        1,
    ))
}

/// Parse group statement
pub fn parse_group(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("group requires a name"))?
        .as_ref()
        .trim_end_matches(':')
        .to_string();

    Ok(Statement::new(
        StatementKind::Group {
            name: name.clone(),
            body: Vec::new(),
        },
        Value::Identifier(name),
        0,
        line_number,
        1,
    ))
}

/// Parse automate statement: automate <target> [mode <note|global>]:
pub fn parse_automate(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // First part is the target
    let target = parts
        .next()
        .ok_or_else(|| anyhow!("automate requires a target"))?
        .as_ref()
        .trim_end_matches(':')
        .to_string();

    // Optional mode: 'mode note' or 'mode global'
    let mut mode: Option<String> = None;
    if let Some(word) = parts.next() {
        if word.as_ref() == "mode" {
            if let Some(m) = parts.next() {
                mode = Some(m.as_ref().trim_end_matches(':').to_string());
            }
        }
    }

    let mut map = HashMap::new();
    if let Some(m) = mode {
        map.insert("mode".to_string(), Value::String(m));
    }

    Ok(Statement::new(
        StatementKind::Automate { target },
        Value::Map(map),
        0,
        line_number,
        1,
    ))
}

/// Parse loop statement
pub fn parse_loop(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // Parse: loop <count>:
    let count_str_ref = parts
        .next()
        .ok_or_else(|| anyhow!("loop requires a count"))?;
    let count_str = count_str_ref.as_ref().trim_end_matches(':');

    let count = if let Ok(num) = count_str.parse::<f32>() {
        Value::Number(num)
    } else {
        Value::Identifier(count_str.to_string())
    };

    Ok(Statement::new(
        StatementKind::Loop {
            count,
            body: Vec::new(), // Will be filled during indentation parsing
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse for statement
pub fn parse_for(
    parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // Parse: for <var> in <iterable>:
    let parts_vec: Vec<String> = parts.map(|s| s.as_ref().to_string()).collect();

    if parts_vec.is_empty() {
        return Err(anyhow!("for loop requires a variable name"));
    }

    let variable = parts_vec[0].clone();

    // Expect "in"
    if parts_vec.len() < 2 || parts_vec[1] != "in" {
        return Err(anyhow!("Expected 'in' after variable in for loop"));
    }

    // Parse iterable (array or identifier)
    let iterable_str = parts_vec[2..].join(" ");
    let iterable_str = iterable_str.trim_end_matches(':').trim();

    let iterable = if iterable_str.starts_with('[') && iterable_str.ends_with(']') {
        // Parse as array: [1, 2, 3] or range: [1..10]
        parse_array_value(iterable_str)?
    } else {
        // Parse as identifier or number
        if let Ok(num) = iterable_str.parse::<f32>() {
            Value::Number(num)
        } else {
            Value::Identifier(iterable_str.to_string())
        }
    };

    Ok(Statement::new(
        StatementKind::For {
            variable,
            iterable,
            body: Vec::new(), // Will be filled during indentation parsing
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse if statement
pub fn parse_if(
    parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // Parse: if <condition>:
    let condition_str = parts
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let condition_str = condition_str.trim_end_matches(':').trim();

    // Parse condition (for now, simple comparison)
    let condition = parse_condition(condition_str)?;

    Ok(Statement::new(
        StatementKind::If {
            condition,
            body: Vec::new(), // Will be filled during indentation parsing
            else_body: None,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse else statement (can be "else if" or just "else")
pub fn parse_else(line: &str, line_number: usize) -> Result<Statement> {
    // Check if this is "else if" or just "else"
    if line.trim().starts_with("else if") {
        // Parse as "else if <condition>:"
        let condition_str = line.trim().strip_prefix("else if").unwrap().trim();
        let condition_str = condition_str.trim_end_matches(':').trim();
        let condition = parse_condition(condition_str)?;

        Ok(Statement::new(
            StatementKind::If {
                condition,
                body: Vec::new(), // Will be filled during indentation parsing
                else_body: None,
            },
            Value::Null,
            0,
            line_number,
            1,
        ))
    } else {
        // Just "else:"
        Ok(Statement::new(
            StatementKind::Comment, // We'll handle this specially during body parsing
            Value::String("else".to_string()),
            0,
            line_number,
            1,
        ))
    }
}

/// Parse call statement
/// Supports:
/// - call groupName
/// - call target = "pattern" (inline pattern assignment)
pub fn parse_call(
    line: &str,
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let first = parts
        .next()
        .ok_or_else(|| anyhow!("call requires a target"))?
        .as_ref()
        .to_string();

    // Check if this is an inline pattern assignment: call target = "pattern"
    if line.contains('=') {
        // Split by = to get target and pattern
        let eq_parts: Vec<&str> = line.splitn(2, '=').collect();
        if eq_parts.len() == 2 {
            // Extract target (remove "call " prefix)
            let target = eq_parts[0]
                .trim()
                .strip_prefix("call")
                .unwrap_or(eq_parts[0])
                .trim()
                .to_string();
            let pattern = eq_parts[1].trim().trim_matches('"').to_string();

            // Create an inline pattern statement
            // Store pattern data in the Call's value field
            let mut map = HashMap::new();
            map.insert("inline_pattern".to_string(), Value::Boolean(true));
            map.insert("target".to_string(), Value::String(target.clone()));
            map.insert("pattern".to_string(), Value::String(pattern));

            return Ok(Statement::new(
                StatementKind::Call {
                    name: target,
                    args: Vec::new(),
                },
                Value::Map(map),
                0,
                line_number,
                1,
            ));
        }
    }

    // Regular call statement (call groupName or call patternName)
    Ok(Statement::new(
        StatementKind::Call {
            name: first,
            args: Vec::new(),
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse spawn statement
pub fn parse_spawn(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("spawn requires a target"))?
        .as_ref()
        .to_string();

    Ok(Statement::new(
        StatementKind::Spawn {
            name,
            args: Vec::new(),
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse on statement (event handler)
pub fn parse_on(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // Parse: on eventName: or on eventName once:
    let event_name = parts
        .next()
        .ok_or_else(|| anyhow!("on statement requires an event name"))?
        .as_ref()
        .to_string();

    // Check for "once" keyword
    let next_word = parts.next();
    let once = next_word.map(|w| w.as_ref() == "once").unwrap_or(false);

    // Store once flag in args (temporary workaround until we have proper AST)
    let args = if once {
        Some(vec![Value::String("once".to_string())])
    } else {
        None
    };

    Ok(Statement::new(
        StatementKind::On {
            event: event_name,
            args,
            body: Vec::new(), // Will be filled during indentation parsing
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse emit statement (event emission)
pub fn parse_emit(
    line: &str,
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    // Parse: emit eventName { key: value, key: value }
    let event_name = parts
        .next()
        .ok_or_else(|| anyhow!("emit statement requires an event name"))?
        .as_ref()
        .to_string();

    // Parse payload if present
    let remainder = line.splitn(2, &event_name).nth(1).unwrap_or("").trim();

    let payload = if remainder.starts_with('{') && remainder.ends_with('}') {
        // Parse as map: { key: value, key: value }
        let inner = &remainder[1..remainder.len() - 1];
        let mut map = HashMap::new();

        // Split by comma and parse key-value pairs
        for pair in inner.split(',') {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value_str = parts[1].trim();

                // Parse value
                let value = if value_str.starts_with('"') && value_str.ends_with('"') {
                    Value::String(value_str.trim_matches('"').to_string())
                } else if let Ok(num) = value_str.parse::<f32>() {
                    Value::Number(num)
                } else {
                    Value::Identifier(value_str.to_string())
                };

                map.insert(key, value);
            }
        }

        Some(Value::Map(map))
    } else {
        None
    };

    Ok(Statement::new(
        StatementKind::Emit {
            event: event_name,
            payload,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
