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
    // Parse: loop <count>:  OR plain `loop:` (no count)
    // If no count is provided we store Value::Null to indicate an unbounded loop
    let count_opt = parts.next();

    let count = if let Some(count_str_ref) = count_opt {
        let count_str = count_str_ref.as_ref().trim_end_matches(':');
        // Support forms:
        // - loop pass:
        // - loop pass():
        // - loop pass(500):
        // - loop 4:
        if let Ok(num) = count_str.parse::<f32>() {
            Value::Number(num)
        } else if count_str.contains('(') && count_str.ends_with(')') {
            // Parse as a call-like value (e.g., pass(500))
            if let Some(open_idx) = count_str.find('(') {
                let name = count_str[..open_idx].to_string();
                let inside = &count_str[open_idx + 1..count_str.len() - 1];
                let args = if inside.trim().is_empty() {
                    Vec::new()
                } else {
                    crate::language::syntax::parser::driver::parse_function_args(inside)?
                };
                Value::Call { name, args }
            } else {
                Value::Identifier(count_str.to_string())
            }
        } else {
            Value::Identifier(count_str.to_string())
        }
    } else {
        // No explicit count -> mark as Null (infinite/indefinite)
        Value::Null
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

/// Parse function statement: function name(arg1, arg2, ...):
pub fn parse_function(line: &str, line_number: usize) -> Result<Statement> {
    // Expect form: function <name>(arg1, arg2, ...):
    let after_kw = line
        .trim()
        .strip_prefix("function")
        .ok_or_else(|| anyhow!("function parsing error"))?
        .trim();

    // Find name and args parentheses
    if let Some(paren_idx) = after_kw.find('(') {
        let name = after_kw[..paren_idx].trim().to_string();
        if let Some(close_idx) = after_kw.rfind(')') {
            let args_str = &after_kw[paren_idx + 1..close_idx];
            // Parse parameter names: split by ',' and trim
            let params: Vec<String> = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                args_str
                    .split(',')
                    .map(|s| s.trim().trim_end_matches(':').to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            return Ok(Statement::new(
                StatementKind::Function {
                    name: name.clone(),
                    parameters: params,
                    body: Vec::new(),
                },
                Value::Identifier(name),
                0,
                line_number,
                1,
            ));
        }
    }

    Err(anyhow!("Invalid function declaration"))
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

        // Mark this If as originating from an 'else if' so the outer parser
        // post-process can attach it as an else-branch to the previous If.
        Ok(Statement::new(
            StatementKind::If {
                condition,
                body: Vec::new(), // Will be filled during indentation parsing
                else_body: None,
            },
            Value::String("else-if".to_string()),
            0,
            line_number,
            1,
        ))
    } else {
        // Just "else:"
        // Return a lightweight marker (Comment with value "else") so the
        // outer parser can detect and attach the following indented block
        // as the else-body of the preceding If.
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
    // Determine the call name. If the call uses parentheses (call name(arg1, arg2))
    // extract the name from the text before the first '(', otherwise fall back to
    // the whitespace-split first token.
    let first_token = parts
        .next()
        .ok_or_else(|| anyhow!("call requires a target"))?
        .as_ref()
        .to_string();
    let mut call_name = first_token.clone();
    if line.find('(').is_some() {
        // Extract the substring between 'call' and the first '(' as the name
        if let Some(after_call) = line.trim().strip_prefix("call") {
            let snippet = after_call.trim();
            if let Some(pidx) = snippet.find('(') {
                call_name = snippet[..pidx].trim().to_string();
            }
        }
    }

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
    // Support call with arguments: call name(arg1, arg2)
    if let Some(paren_idx) = line.find('(') {
        if let Some(close_idx) = line.rfind(')') {
            let args_str = &line[paren_idx + 1..close_idx];
            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                // Reuse parse_function_args from parent module
                crate::language::syntax::parser::driver::parse_function_args(args_str)?
            };

            return Ok(Statement::new(
                StatementKind::Call {
                    name: call_name,
                    args,
                },
                Value::Null,
                0,
                line_number,
                1,
            ));
        }
    }

    Ok(Statement::new(
        StatementKind::Call {
            name: call_name,
            args: Vec::new(),
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse break statement
pub fn parse_break(
    _parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    Ok(Statement::new(
        StatementKind::Break,
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
    // Accept forms like: on beat: | on beat once: | on beat(4): | on beat(4) once:
    let raw = parts
        .next()
        .ok_or_else(|| anyhow!("on statement requires an event name"))?
        .as_ref()
        .to_string();

    // Parse optional interval within parentheses, e.g. beat(4)
    let mut event_name = raw.clone();
    let mut args_vec: Vec<Value> = Vec::new();
    if let Some(open_idx) = raw.find('(') {
        if raw.ends_with(')') {
            let base = raw[..open_idx].to_string();
            let inside = &raw[open_idx + 1..raw.len() - 1];
            if let Ok(n) = inside.trim().parse::<f32>() {
                args_vec.push(Value::Number(n));
            }
            event_name = base;
        }
    }

    // Check for "once" keyword as next token
    let next_word = parts.next();
    let once = next_word.map(|w| w.as_ref() == "once").unwrap_or(false);
    if once {
        args_vec.push(Value::String("once".to_string()));
    }

    let args = if !args_vec.is_empty() {
        Some(args_vec)
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
