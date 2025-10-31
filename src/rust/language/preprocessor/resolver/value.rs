use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Resolve a value by recursively expanding identifiers from variable tables
pub fn resolve_value(value: &Value, variables: &HashMap<String, Value>, depth: usize) -> Value {
    const MAX_DEPTH: usize = 32;

    if depth > MAX_DEPTH {
        return value.clone();
    }

    match value {
        Value::Identifier(name) => {
            // Delegate dotted resolution to central util (falls back to Null when missing)
            if name.contains('.') {
                return crate::utils::props::resolve_dotted_from_table(name, variables);
            }
            if let Some(resolved) = variables.get(name) {
                // Recursively resolve to handle chained identifiers
                resolve_value(resolved, variables, depth + 1)
            } else {
                value.clone()
            }
        }
        Value::Map(map) => {
            let mut resolved_map = HashMap::new();
            for (k, v) in map {
                resolved_map.insert(k.clone(), resolve_value(v, variables, depth + 1));
            }
            Value::Map(resolved_map)
        }
        Value::Array(arr) => {
            let resolved: Vec<Value> = arr
                .iter()
                .map(|v| resolve_value(v, variables, depth + 1))
                .collect();
            Value::Array(resolved)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
#[path = "test_value.rs"]
mod tests;
