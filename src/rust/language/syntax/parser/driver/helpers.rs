use crate::language::syntax::ast::Value;
/// Helper functions for parsing values, arguments, and complex structures
use anyhow::Result;
use std::collections::HashMap;

/// Parse function arguments from string
/// Supports: numbers, strings, identifiers, arrays, maps
pub fn parse_function_args(args_str: &str) -> Result<Vec<Value>> {
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
pub fn parse_single_arg(arg: &str) -> Result<Value> {
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

/// Parse synth definition: synth waveform { params } OR synth plugin.<name> { params }
/// Returns a Map with type="synth", waveform/plugin info, and parameters
/// 
/// Supported syntaxes:
/// - synth "sine" { attack: 0.1 }
/// - synth plugin.acid.synth { waveform: "sine" }
/// - synth { waveform: "sine", attack: 0.1 }  // waveform in params
pub fn parse_synth_definition(input: &str) -> Result<Value> {
    // Remove "synth " prefix
    let input = input.trim_start_matches("synth ").trim();

    // Check if we have braces
    let (waveform_or_plugin, params_str) = if let Some(brace_idx) = input.find('{') {
        let before_brace = input[..brace_idx].trim();
        let params = &input[brace_idx..];
        (before_brace, params)
    } else {
        // No parameters, just waveform or plugin reference
        return Ok(Value::Map({
            let mut map = HashMap::new();
            map.insert("type".to_string(), Value::String("synth".to_string()));
            
            // Check if it's a plugin reference (plugin.author.name)
            if input.starts_with("plugin.") {
                let parts: Vec<&str> = input.split('.').collect();
                if parts.len() >= 3 {
                    map.insert("plugin_author".to_string(), Value::String(parts[1].to_string()));
                    map.insert("plugin_name".to_string(), Value::String(parts[2].to_string()));
                    if parts.len() >= 4 {
                        map.insert("plugin_export".to_string(), Value::String(parts[3].to_string()));
                    }
                }
            } else {
                map.insert("waveform".to_string(), Value::String(input.to_string()));
            }
            
            map
        }));
    };

    // Parse parameters from { key: value, ... }
    let params_str = params_str.trim_matches(|c| c == '{' || c == '}').trim();
    let mut params_map = HashMap::new();

    // Add type
    params_map.insert("type".to_string(), Value::String("synth".to_string()));

    // Handle waveform_or_plugin (what comes before the braces)
    if !waveform_or_plugin.is_empty() {
        // Check if it's a plugin reference (contains '.' → variable.property or plugin.author.name)
        if waveform_or_plugin.contains('.') {
            // Store plugin reference in internal field (will be resolved in interpreter)
            params_map.insert("_plugin_ref".to_string(), Value::String(waveform_or_plugin.to_string()));
            
            // Also check for explicit plugin.author.name format
            if waveform_or_plugin.starts_with("plugin.") {
                let parts: Vec<&str> = waveform_or_plugin.split('.').collect();
                if parts.len() >= 3 {
                    params_map.insert("plugin_author".to_string(), Value::String(parts[1].to_string()));
                    params_map.insert("plugin_name".to_string(), Value::String(parts[2].to_string()));
                    if parts.len() >= 4 {
                        params_map.insert("plugin_export".to_string(), Value::String(parts[3].to_string()));
                    }
                }
            }
        } else {
            // It's a waveform string
            params_map.insert("waveform".to_string(), Value::String(waveform_or_plugin.to_string()));
        }
    }

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
                let value_str = value_part.trim().trim_matches(',').trim_matches('"');

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
                    // Store as string (override waveform if specified in params)
                    params_map.insert(key, Value::String(value_str.to_string()));
                }
            }
        }
    }

    Ok(Value::Map(params_map))
}

/// Parse array value like [{ key: val }, ...]
pub fn parse_array_value(input: &str) -> Result<Value> {
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
pub fn parse_map_value(input: &str) -> Result<Value> {
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

/// Parse a condition string into a Value (for if statements)
/// Supports: var > value, var < value, var == value, var != value, var >= value, var <= value
pub fn parse_condition(condition_str: &str) -> Result<Value> {
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
