use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::module::Module,
        shared::{ duration::Duration, value::Value },
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_trigger(
    stmt: &Statement,
    entity: &str,
    duration: &mut Duration,
    module: &Module,
    path: &str,
    global_store: &GlobalStore
) -> Statement {
    let logger = Logger::new();

    let mut final_duration = duration.clone();
    let mut final_value = stmt.value.clone();

    // Duration resolution
    if let Duration::Identifier(ident) = duration {
        if let Some(val) = resolve_identifier(ident, module, global_store) {
            match val {
                Value::Number(n) => {
                    final_duration = Duration::Number(n);
                }
                Value::String(s) => {
                    final_duration = Duration::Identifier(s);
                }
                Value::Identifier(s) if s == "auto" => {
                    final_duration = Duration::Auto;
                }
                _ => {}
            }
        }
    }

    // Params value resolution
    final_value = match &stmt.value {
        Value::Identifier(ident) => {
            resolve_identifier(ident, module, global_store).unwrap_or_else(|| {
                logger.log_error_with_stacktrace(
                    &format!("'{path}': value identifier '{ident}' not found"),
                    &format!("{}:{}:{}", module.path, stmt.line, stmt.column)
                );
                Value::Null
            })
        }
        Value::Map(map) => {
            let mut resolved_map = HashMap::new();
            for (k, v) in map {
                let resolved = match v {
                    Value::Identifier(id) => {
                        resolve_identifier(id, module, global_store).unwrap_or(Value::Null)
                    }
                    other => other.clone(),
                };
                resolved_map.insert(k.clone(), resolved);
            }
            Value::Map(resolved_map)
        }
        other => other.clone(),
    };

    Statement {
        kind: StatementKind::Trigger {
            entity: entity.to_string(),
            duration: final_duration,
        },
        value: final_value,
        line: stmt.line,
        column: stmt.column,
        indent: stmt.indent,
    }
}

fn resolve_identifier(ident: &str, module: &Module, global_store: &GlobalStore) -> Option<Value> {
    if let Some(val) = module.variable_table.get(ident) {
        return Some(resolve_value(val, module, global_store));
    }

    for (_, other_mod) in &global_store.modules {
        if let Some(val) = other_mod.export_table.get_export(ident) {
            return Some(resolve_value(val, other_mod, global_store));
        }
    }

    None
}

fn resolve_value(val: &Value, module: &Module, global_store: &GlobalStore) -> Value {
    match val {
        Value::Identifier(inner) =>
            resolve_identifier(inner, module, global_store).unwrap_or(
                Value::Identifier(inner.clone())
            ),
        Value::Map(map) => {
            let mut resolved = HashMap::new();
            for (k, v) in map {
                resolved.insert(k.clone(), resolve_value(v, module, global_store));
            }
            Value::Map(resolved)
        }
        other => other.clone(),
    }
}
