use crate::language::syntax::ast::Value;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Parse a chain of arrow-separated effects
pub fn parse_chained_effects(effects_str: &str) -> Result<Value> {
    let effects = effects_str
        .split("->")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .try_fold(HashMap::new(), |mut map, effect| {
            parse_single_effect(effect).map(|(name, params)| {
                map.insert(name, params);
                map
            })
        })?;

    Ok(Value::Map(effects))
}

/// Parse a single effect definition into (name, parameters)
fn parse_single_effect(effect_str: &str) -> Result<(String, Value)> {
    let effect_str = effect_str.trim();
    if effect_str.is_empty() {
        return Err(anyhow!("Empty effect definition"));
    }

    // Handle function-like syntax: effect_name(param1, param2, ...)
    if let Some((name, params_str)) = effect_str.split_once('(') {
        let name = name.trim().to_string();
        let params_str = params_str.trim_end_matches(')');

        // Handle boolean single parameter
        if params_str == "true" {
            return Ok((name, Value::Boolean(true)));
        } else if params_str == "false" {
            return Ok((name, Value::Boolean(false)));
        }

        // Handle numeric single parameter
        if let Ok(num) = params_str.parse::<f32>() {
            return Ok((name, Value::Number(num)));
        }

        // Handle parameter map
        if params_str.contains('{') && params_str.contains('}') {
            let map_str = params_str.trim_matches(|c| c == '{' || c == '}');
            let mut params_map = HashMap::new();

            for pair in map_str.split(',') {
                if let Some((key, value)) = pair.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim();

                    let value = if value == "true" {
                        Value::Boolean(true)
                    } else if value == "false" {
                        Value::Boolean(false)
                    } else if let Ok(num) = value.parse::<f32>() {
                        Value::Number(num)
                    } else {
                        Value::String(value.trim_matches('"').to_string())
                    };

                    params_map.insert(key, value);
                }
            }

            Ok((name, Value::Map(params_map)))
        } else {
            // Single string parameter
            Ok((
                name,
                Value::String(params_str.trim_matches('"').to_string()),
            ))
        }
    } else {
        // Effect without parameters
        Ok((effect_str.to_string(), Value::Null))
    }
}

#[cfg(test)]
#[path = "test_effects.rs"]
mod tests;
