use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Resolve a dotted identifier like "a.b.c" from a variable table.
/// Returns Value::Null if resolution fails at any step.
pub fn resolve_dotted_from_table(name: &str, variables: &HashMap<String, Value>) -> Value {
    resolve_dotted_with_lookup(name, |k| variables.get(k).cloned())
}

/// Resolve dotted identifier using a lookup function for the root name.
pub fn resolve_dotted_with_lookup<F>(name: &str, mut lookup: F) -> Value
where
    F: FnMut(&str) -> Option<Value>,
{
    if name.is_empty() {
        return Value::Null;
    }

    let parts: Vec<&str> = name.split('.').collect();
    if parts.is_empty() {
        return Value::Null;
    }

    // Resolve first part using provided lookup
    let mut current: Option<Value> = lookup(parts[0]);
    if current.is_none() {
        return Value::Null;
    }

    for prop in parts.iter().skip(1) {
        match current {
            Some(Value::Map(ref map)) => {
                if let Some(next) = map.get(*prop) {
                    current = Some(next.clone());
                } else {
                    return Value::Null;
                }
            }
            _ => return Value::Null,
        }
    }

    current.unwrap_or(Value::Null)
}

/// Ensure a set of default properties exist on an object map.
/// `kind_hint` can be used to provide specialized defaults for different object kinds.
pub fn ensure_default_properties(map: &mut HashMap<String, Value>, kind_hint: Option<&str>) {
    // Common defaults for audio objects
    map.entry("volume".to_string())
        .or_insert(Value::Number(1.0));
    map.entry("gain".to_string()).or_insert(Value::Number(1.0));
    map.entry("pan".to_string()).or_insert(Value::Number(0.0));
    map.entry("detune".to_string())
        .or_insert(Value::Number(0.0));
    // Provide a visible type for printing
    map.entry("type".to_string())
        .or_insert(Value::String("object".to_string()));

    if let Some(kind) = kind_hint {
        match kind {
            "synth" => {
                map.entry("type".to_string())
                    .or_insert(Value::String("synth".to_string()));
                map.entry("waveform".to_string())
                    .or_insert(Value::String("sine".to_string()));
            }
            "mapping" => {
                map.entry("_type".to_string())
                    .or_insert(Value::String("midi_mapping".to_string()));
            }
            "trigger" => {
                // triggers commonly reference sample URIs; nothing to add by default
            }
            _ => {}
        }
    }
}
