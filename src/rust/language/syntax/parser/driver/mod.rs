pub mod directive;
pub mod duration;
pub mod helpers;
pub mod preprocessing;
pub mod routing;
pub mod statements;
pub mod trigger;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::language::syntax::ast::{Statement, StatementKind, Value};
use directive::parse_directive;
use preprocessing::{preprocess_multiline_arrow_calls, preprocess_multiline_braces};
use routing::parse_routing_command;
use statements::*;
use trigger::parse_trigger_line;

pub struct SimpleParser;

impl SimpleParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(path: impl AsRef<Path>) -> Result<Vec<Statement>> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read source file: {}", path.display()))?;
        Self::parse(&content, path.to_path_buf())
    }

    pub fn parse(source: &str, path: PathBuf) -> Result<Vec<Statement>> {
        // Pre-process: merge ALL multiline statements with braces
        let preprocessed = preprocess_multiline_braces(source);

        // Then merge multiline arrow calls (without braces)
        let preprocessed = preprocess_multiline_arrow_calls(&preprocessed);

        let lines: Vec<_> = preprocessed.lines().collect();
        Self::parse_lines(&lines, 0, lines.len(), 0, &path)
    }

    fn parse_lines(
        lines: &[&str],
        start: usize,
        end: usize,
        base_indent: usize,
        path: &Path,
    ) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        let mut i = start;

        while i < end {
            let raw_line = lines[i];
            let line_number = i + 1;
            let trimmed = raw_line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Calculate indentation
            let indent = raw_line.len() - raw_line.trim_start().len();

            // If this line is less indented than base, stop parsing this block
            if indent < base_indent {
                break;
            }

            // Parse the statement
            let mut statement = match Self::parse_line(trimmed, line_number, path) {
                Ok(stmt) => {
                    let mut s = stmt;
                    s.indent = indent;
                    s
                }
                Err(error) => {
                    let error_msg = error.to_string();

                    // Push structured error to WASM registry if available
                    #[cfg(target_arch = "wasm32")]
                    {
                        use crate::web::registry::debug;
                        if debug::is_debug_errors_enabled() {
                            debug::push_parse_error_from_parts(
                                error_msg.clone(),
                                line_number,
                                1,
                                "ParseError".to_string(),
                            );
                        }
                    }

                    Statement::new(
                        StatementKind::Error { message: error_msg },
                        Value::String(trimmed.to_string()),
                        indent,
                        line_number,
                        1,
                    )
                }
            };

            // If this is a statement needing body parsing (group, for, loop, if, on)
            let needs_body_parsing = matches!(
                &statement.kind,
                StatementKind::Group { .. }
                    | StatementKind::For { .. }
                    | StatementKind::Loop { .. }
                    | StatementKind::If { .. }
                    | StatementKind::On { .. }
            );

            if needs_body_parsing {
                i += 1; // Move to next line

                // Find the end of the body (next line with same or less indentation)
                let block_indent = indent;
                let body_start = i;
                let mut body_end = i;

                while body_end < end {
                    let body_line = lines[body_end];
                    let body_trimmed = body_line.trim();

                    if body_trimmed.is_empty() || body_trimmed.starts_with('#') {
                        body_end += 1;
                        continue;
                    }

                    let body_indent = body_line.len() - body_line.trim_start().len();
                    if body_indent <= block_indent {
                        break;
                    }

                    body_end += 1;
                }

                // Parse the body recursively
                let body = Self::parse_lines(lines, body_start, body_end, block_indent + 1, path)?;

                // Update the statement with the parsed body
                match &statement.kind {
                    StatementKind::Group { name, .. } => {
                        let group_name = name.clone();
                        statement.kind = StatementKind::Group {
                            name: group_name.clone(),
                            body,
                        };
                        statement.value = Value::Identifier(group_name);
                    }
                    StatementKind::For {
                        variable, iterable, ..
                    } => {
                        statement.kind = StatementKind::For {
                            variable: variable.clone(),
                            iterable: iterable.clone(),
                            body,
                        };
                    }
                    StatementKind::Loop { count, .. } => {
                        statement.kind = StatementKind::Loop {
                            count: count.clone(),
                            body,
                        };
                    }
                    StatementKind::On { event, args, .. } => {
                        statement.kind = StatementKind::On {
                            event: event.clone(),
                            args: args.clone(),
                            body,
                        };
                    }
                    StatementKind::If { condition, .. } => {
                        // Check if there's an else clause after the body
                        let mut else_body = None;
                        if body_end < end {
                            let next_line = lines[body_end].trim();
                            if next_line.starts_with("else") {
                                // Found else, parse its body
                                let else_start = body_end + 1;
                                let mut else_end = else_start;

                                while else_end < end {
                                    let else_line = lines[else_end];
                                    let else_trimmed = else_line.trim();

                                    if else_trimmed.is_empty() || else_trimmed.starts_with('#') {
                                        else_end += 1;
                                        continue;
                                    }

                                    let else_indent =
                                        else_line.len() - else_line.trim_start().len();
                                    if else_indent <= block_indent {
                                        break;
                                    }

                                    else_end += 1;
                                }

                                else_body = Some(Self::parse_lines(
                                    lines,
                                    else_start,
                                    else_end,
                                    block_indent + 1,
                                    path,
                                )?);
                                body_end = else_end; // Update i to skip else body
                            }
                        }

                        statement.kind = StatementKind::If {
                            condition: condition.clone(),
                            body,
                            else_body,
                        };
                    }
                    _ => {}
                }

                i = body_end;
                statements.push(statement);
                continue;
            }

            statements.push(statement);
            i += 1;
        }

        Ok(statements)
    }

    fn parse_line(line: &str, line_number: usize, path: &Path) -> Result<Statement> {
        if line.starts_with('@') {
            return parse_directive(line, line_number);
        }

        if line.starts_with('.') {
            return parse_trigger_line(line, line_number);
        }

        let mut parts = line.split_whitespace();
        let keyword = parts
            .next()
            .ok_or_else(|| anyhow!("empty line"))?
            .to_lowercase();

        // Check if this is a property assignment first: target.property = value
        if line.contains('=') && keyword.contains('.') {
            return parse_assign(line, line_number);
        }

        // Check if this looks like a trigger (contains a dot like "drums.kick")
        if keyword.contains('.') && !keyword.contains('(') {
            // Reconstruct the line and parse as trigger
            return parse_trigger_line(line, line_number);
        }

        // Check if this is a bind statement first (before arrow call)
        if keyword == "bind" && line.contains("->") {
            return parse_bind(line, line_number);
        }

        // Check if this is an arrow call: target -> method(args)
        if line.contains("->") {
            return parse_arrow_call(line, line_number);
        }

        match keyword.as_str() {
            "bpm" | "tempo" => parse_tempo(parts, line_number),
            "print" => parse_print(line, line_number),
            "sleep" => parse_sleep(parts, line_number),
            "trigger" => Err(anyhow!(
                "keyword 'trigger' is deprecated; use dot notation like '.alias' instead"
            )),
            "pattern" => parse_pattern(parts, line_number),
            "bank" => parse_bank(parts, line_number),
            "let" => parse_let(line, parts, line_number),
            "var" => parse_var(line, parts, line_number),
            "const" => parse_const(line, parts, line_number),
            "for" => parse_for(parts, line_number),
            "loop" => parse_loop(parts, line_number),
            "if" => parse_if(parts, line_number),
            "else" => parse_else(line, line_number),
            "group" => parse_group(parts, line_number),
            "call" => parse_call(parts, line_number),
            "spawn" => parse_spawn(parts, line_number),
            "on" => parse_on(parts, line_number),
            "emit" => parse_emit(line, parts, line_number),
            "routing" => parse_routing_command(parts, line_number),
            _ => {
                // Check if this looks like a potential trigger identifier (single word, no special chars except dots)
                // This handles cases like variable references: let kick = drums.kick; kick
                if keyword
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                {
                    // Parse as trigger
                    return parse_trigger_line(line, line_number);
                }

                let error_msg = format!(
                    "Unknown statement '{}' at {}:{}",
                    keyword,
                    path.display(),
                    line_number
                );

                // Push structured error to WASM registry if available
                #[cfg(target_arch = "wasm32")]
                {
                    use crate::web::registry::debug;
                    if debug::is_debug_errors_enabled() {
                        debug::push_parse_error_from_parts(
                            format!("Unknown statement '{}'", keyword),
                            line_number,
                            1,
                            "UnknownStatement".to_string(),
                        );
                    }
                }

                Ok(Statement::new(
                    StatementKind::Unknown,
                    Value::String(error_msg),
                    0,
                    line_number,
                    1,
                ))
            }
        }
    }

    /// Parse a condition string into a Value (for if statements)
    /// Supports: var > value, var < value, var == value, var != value, var >= value, var <= value
    fn parse_condition(condition_str: &str) -> Result<Value> {
        use std::collections::HashMap;

        // Find the operator
        let operators = vec![">=", "<=", "==", "!=", ">", "<"];
        for op in operators {
            if let Some(idx) = condition_str.find(op) {
                let left = condition_str[..idx].trim();
                let right = condition_str[idx + op.len()..].trim();

                // Create a map representing the condition
                let mut map = HashMap::new();
                map.insert("operator".to_string(), Value::String(op.to_string()));
                map.insert(
                    "left".to_string(),
                    if let Ok(num) = left.parse::<f32>() {
                        Value::Number(num)
                    } else {
                        Value::Identifier(left.to_string())
                    },
                );
                map.insert(
                    "right".to_string(),
                    if let Ok(num) = right.parse::<f32>() {
                        Value::Number(num)
                    } else {
                        Value::Identifier(right.to_string())
                    },
                );

                return Ok(Value::Map(map));
            }
        }

        // No operator found, treat as boolean identifier
        Ok(Value::Identifier(condition_str.to_string()))
    }
}

/// Parse an arrow call: target -> method(args) -> method2(args2)
/// Supports chaining multiple calls
fn parse_arrow_call(line: &str, line_number: usize) -> Result<Statement> {
    use std::collections::HashMap;

    // Split by "->" to get chain of calls
    let parts: Vec<&str> = line.split("->").map(|s| s.trim()).collect();

    if parts.len() < 2 {
        return Err(anyhow!("Arrow call requires at least one '->' operator"));
    }

    // First part is the target
    let target = parts[0].to_string();

    // Parse method calls
    let mut calls = Vec::new();

    for method_call in &parts[1..] {
        // Parse method(args) or just method
        if let Some(paren_idx) = method_call.find('(') {
            let method_name = method_call[..paren_idx].trim();
            let args_str = &method_call[paren_idx + 1..];

            // Find matching closing paren
            let close_paren = args_str
                .rfind(')')
                .ok_or_else(|| anyhow!("Missing closing parenthesis in arrow call"))?;

            let args_str = &args_str[..close_paren];

            // Parse arguments
            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                parse_function_args(args_str)?
            };

            calls.push((method_name.to_string(), args));
        } else {
            // Method without args
            calls.push((method_call.trim().to_string(), Vec::new()));
        }
    }

    // For now, we'll store all calls as separate ArrowCall statements
    // or we can store them in a chain structure
    // Let's store the first call and chain the rest

    if calls.is_empty() {
        return Err(anyhow!("No method calls found in arrow call"));
    }

    let (method, args) = calls[0].clone();

    // Store chain in value for later processing
    let mut chain_value = HashMap::new();
    chain_value.insert("target".to_string(), Value::String(target.clone()));
    chain_value.insert("method".to_string(), Value::String(method.clone()));
    chain_value.insert("args".to_string(), Value::Array(args.clone()));

    // Add remaining calls to chain
    if calls.len() > 1 {
        let chain_calls: Vec<Value> = calls[1..]
            .iter()
            .map(|(m, a)| {
                let mut call_map = HashMap::new();
                call_map.insert("method".to_string(), Value::String(m.clone()));
                call_map.insert("args".to_string(), Value::Array(a.clone()));
                Value::Map(call_map)
            })
            .collect();

        chain_value.insert("chain".to_string(), Value::Array(chain_calls));
    }

    Ok(Statement::new(
        StatementKind::ArrowCall {
            target,
            method,
            args,
        },
        Value::Map(chain_value),
        0,
        line_number,
        1,
    ))
}

/// Parse function arguments from string
/// Supports: numbers, strings, identifiers, arrays, maps
fn parse_function_args(args_str: &str) -> Result<Vec<Value>> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut depth = 0; // Track nested structures
    let mut in_string = false;

    for ch in args_str.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current_arg.push(ch);
            }
            '[' | '{' if !in_string => {
                depth += 1;
                current_arg.push(ch);
            }
            ']' | '}' if !in_string => {
                depth -= 1;
                current_arg.push(ch);
            }
            ',' if depth == 0 && !in_string => {
                // End of argument
                if !current_arg.trim().is_empty() {
                    args.push(parse_single_arg(current_arg.trim())?);
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    // Last argument
    if !current_arg.trim().is_empty() {
        args.push(parse_single_arg(current_arg.trim())?);
    }

    Ok(args)
}

/// Parse a single argument value
fn parse_single_arg(arg: &str) -> Result<Value> {
    use std::collections::HashMap;

    let arg = arg.trim();

    // String literal
    if arg.starts_with('"') && arg.ends_with('"') {
        return Ok(Value::String(arg[1..arg.len() - 1].to_string()));
    }

    // Array
    if arg.starts_with('[') && arg.ends_with(']') {
        let inner = &arg[1..arg.len() - 1];
        let items = parse_function_args(inner)?;
        return Ok(Value::Array(items));
    }

    // Map/Object
    if arg.starts_with('{') && arg.ends_with('}') {
        let inner = &arg[1..arg.len() - 1];
        let mut map = HashMap::new();

        // Parse key: value pairs
        for pair in inner.split(',') {
            if let Some(colon_idx) = pair.find(':') {
                let key = pair[..colon_idx].trim().trim_matches('"');
                let value = parse_single_arg(pair[colon_idx + 1..].trim())?;
                map.insert(key.to_string(), value);
            }
        }

        return Ok(Value::Map(map));
    }

    // Number
    if let Ok(num) = arg.parse::<f32>() {
        return Ok(Value::Number(num));
    }

    // Boolean
    match arg.to_lowercase().as_str() {
        "true" => return Ok(Value::Boolean(true)),
        "false" => return Ok(Value::Boolean(false)),
        _ => {}
    }

    // Default to identifier
    Ok(Value::Identifier(arg.to_string()))
}

/// Parse synth definition: synth waveform { params }
/// Returns a Map with type="synth", waveform, and ADSR parameters
fn parse_synth_definition(input: &str) -> Result<Value> {
    use std::collections::HashMap;

    // Remove "synth " prefix
    let input = input.trim_start_matches("synth ").trim();

    // Extract waveform (everything before '{')
    let (waveform, params_str) = if let Some(brace_idx) = input.find('{') {
        let waveform = input[..brace_idx].trim();
        let params = &input[brace_idx..];
        (waveform, params)
    } else {
        // No parameters, just waveform
        return Ok(Value::Map({
            let mut map = HashMap::new();
            map.insert("type".to_string(), Value::String("synth".to_string()));
            map.insert("waveform".to_string(), Value::String(input.to_string()));
            map
        }));
    };

    // Parse parameters from { key: value, ... }
    let params_str = params_str.trim_matches(|c| c == '{' || c == '}').trim();
    let mut params_map = HashMap::new();

    // Add type and waveform
    params_map.insert("type".to_string(), Value::String("synth".to_string()));
    params_map.insert("waveform".to_string(), Value::String(waveform.to_string()));

    // Parse key: value pairs (support newlines by replacing them with commas)
    if !params_str.is_empty() {
        // First, remove inline comments (everything after //)
        let mut cleaned_lines = Vec::new();
        for line in params_str.lines() {
            if let Some(comment_pos) = line.find("//") {
                let clean_line = &line[..comment_pos];
                if !clean_line.trim().is_empty() {
                    cleaned_lines.push(clean_line);
                }
            } else if !line.trim().is_empty() {
                cleaned_lines.push(line);
            }
        }

        // Now join lines and split by comma and newline
        let cleaned = cleaned_lines.join("\n");
        let normalized = cleaned.replace('\n', ",").replace('\r', "");

        for pair in normalized.split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim().to_string();
                // Join back in case value contains ':'
                let value_part = parts[1..].join(":");
                let value_str = value_part.trim().trim_matches(',');

                // Parse arrays (for filters)
                if value_str.starts_with('[') {
                    if let Ok(array_val) = parse_array_value(value_str) {
                        params_map.insert(key, array_val);
                        continue;
                    }
                }

                // Try to parse as number
                if let Ok(num) = value_str.parse::<f32>() {
                    params_map.insert(key, Value::Number(num));
                } else {
                    // Store as string
                    params_map.insert(key, Value::String(value_str.to_string()));
                }
            }
        }
    }

    Ok(Value::Map(params_map))
}

/// Parse array value like [{ key: val }, ...]
fn parse_array_value(input: &str) -> Result<Value> {
    let input = input.trim().trim_matches(|c| c == '[' || c == ']').trim();
    if input.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    // Check for range pattern: "start..end"
    if input.contains("..") {
        let parts: Vec<&str> = input.split("..").collect();
        if parts.len() == 2 {
            let start_str = parts[0].trim();
            let end_str = parts[1].trim();

            // Try to parse as numbers
            if let (Ok(start), Ok(end)) = (start_str.parse::<f32>(), end_str.parse::<f32>()) {
                return Ok(Value::Range {
                    start: Box::new(Value::Number(start)),
                    end: Box::new(Value::Number(end)),
                });
            }
        }
    }

    let mut items = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth -= 1;
                current.push(ch);

                if depth == 0 && !current.trim().is_empty() {
                    // Parse this object
                    if let Ok(obj) = parse_map_value(&current) {
                        items.push(obj);
                    }
                    current.clear();
                }
            }
            ',' if depth == 0 => {
                // Skip commas at array level
                continue;
            }
            _ => {
                current.push(ch);
            }
        }
    }

    Ok(Value::Array(items))
}

/// Parse map value like { key: val, key2: val2 }
fn parse_map_value(input: &str) -> Result<Value> {
    use std::collections::HashMap;

    let input = input.trim().trim_matches(|c| c == '{' || c == '}').trim();
    let mut map = HashMap::new();

    for pair in input.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() >= 2 {
            let key = parts[0].trim().to_string();
            // Join back in case value contains ':' (shouldn't happen but just in case)
            let value_part = parts[1..].join(":");

            // Remove inline comments (everything after //)
            let value_clean = if let Some(comment_pos) = value_part.find("//") {
                &value_part[..comment_pos]
            } else {
                &value_part
            };

            let value_str = value_clean.trim().trim_matches('"').trim_matches('\'');

            // Try to parse as number
            if let Ok(num) = value_str.parse::<f32>() {
                map.insert(key, Value::Number(num));
            } else {
                map.insert(key, Value::String(value_str.to_string()));
            }
        }
    }

    Ok(Value::Map(map))
}
