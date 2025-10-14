use super::super::helpers::{parse_function_args, parse_map_value};
use crate::language::syntax::ast::{Statement, StatementKind, Value};
/// Advanced statement parsing: ArrowCall, Assign, Automate, Bind
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Parse an arrow call: target -> method(args) -> method2(args2)
/// Supports chaining multiple calls
pub fn parse_arrow_call(line: &str, line_number: usize) -> Result<Statement> {
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
            let mut method_name = method_call[..paren_idx].trim().to_string();
            // Accept both vel(...) and velocity(...)
            if method_name == "vel" {
                method_name = "velocity".to_string();
            }
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

/// Parse property assignment: target.property = value
pub fn parse_assign(line: &str, line_number: usize) -> Result<Statement> {
    let assign_parts: Vec<&str> = line.splitn(2, '=').collect();
    if assign_parts.len() != 2 {
        return Err(anyhow!("Invalid assignment syntax"));
    }

    let left = assign_parts[0].trim();
    let right = assign_parts[1].trim();

    // Split left into target.property
    let prop_parts: Vec<&str> = left.splitn(2, '.').collect();
    if prop_parts.len() != 2 {
        return Err(anyhow!("Assignment requires target.property syntax"));
    }

    let target = prop_parts[0].trim().to_string();
    let property = prop_parts[1].trim().to_string();

    // Parse value
    let value = if let Ok(num) = right.parse::<f32>() {
        Value::Number(num)
    } else if right.starts_with('"') && right.ends_with('"') {
        Value::String(right.trim_matches('"').to_string())
    } else {
        Value::Identifier(right.to_string())
    };

    Ok(Statement::new(
        StatementKind::Assign { target, property },
        value,
        0,
        line_number,
        1,
    ))
}

/// Parse bind statement: bind source -> target { options }
pub fn parse_bind(line: &str, line_number: usize) -> Result<Statement> {
    // Supported forms:
    //   bind <left> -> <right>
    //   bind <left> -> <right> with { ... }
    //   bind <left> with { ... } -> <right>
    // We'll normalize to source/target and put the options (if any) into the statement value.

    // Remove leading 'bind' and split on '->'
    let after_bind = line
        .trim()
        .strip_prefix("bind")
        .ok_or_else(|| anyhow!("bind parsing error"))?
        .trim();

    let arrow_parts: Vec<&str> = after_bind.splitn(2, "->").collect();
    if arrow_parts.len() != 2 {
        return Err(anyhow!("bind requires source -> target syntax"));
    }

    // Left side may contain 'with { ... }'
    let left_raw = arrow_parts[0].trim();
    let right_raw = arrow_parts[1].trim();

    // Helper to extract optional 'with { ... }' from a side
    fn extract_with(side: &str) -> Result<(String, Option<Value>)> {
        let s = side.trim();
        // Look for 'with {' pattern
        if let Some(with_pos) = s.find("with") {
            let before = s[..with_pos].trim().to_string();
            let after = s[with_pos + "with".len()..].trim();
            if after.starts_with('{') {
                if let Some(close_brace) = after.rfind('}') {
                    let options_str = &after[..close_brace + 1];
                    let options = parse_map_value(options_str)?;
                    return Ok((before, Some(options)));
                } else {
                    return Err(anyhow!("unclosed brace in bind options"));
                }
            }
        }
        // No 'with' found
        Ok((s.to_string(), None))
    }

    let (source_candidate, left_opts) = extract_with(left_raw)?;
    let (target_candidate, right_opts) = extract_with(right_raw)?;

    // Merge options, favouring explicit right-side options (target) over left-side
    let merged_options = match (left_opts, right_opts) {
        (Some(mut l), Some(r)) => {
            // merge maps: r overrides l
            if let (Value::Map(lm), Value::Map(rm)) = (&mut l, &r) {
                for (k, v) in rm.iter() {
                    lm.insert(k.clone(), v.clone());
                }
                Some(l)
            } else {
                Some(r)
            }
        }
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (None, None) => None,
    };

    let source = source_candidate;
    let target = target_candidate;

    Ok(Statement::new(
        StatementKind::Bind { source, target },
        merged_options.unwrap_or(Value::Null),
        0,
        line_number,
        1,
    ))
}
