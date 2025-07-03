use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::module::Module,
        shared::{ duration::Duration, value::Value },
        store::global::GlobalStore,
        utils::validation::is_valid_entity,
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

    if !is_valid_entity(entity, module, global_store) {
        let message = format!("Invalid entity '{}', expected a valid identifier", entity);
        let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
        logger.log_error_with_stacktrace(&message, &stacktrace);

        return Statement {
            kind: stmt.kind.clone(),
            value: Value::Null,
            line: stmt.line,
            column: stmt.column,
            indent: stmt.indent,
        };
    }

    // ✅ Résolution de duration si c'est un identifiant
    if let Duration::Identifier(ident) = duration {
        if let Some(val) = module.variable_table.get(ident) {
            match val {
                Value::Number(num) => {
                    final_duration = Duration::Number(*num);
                }
                Value::String(s) => {
                    final_duration = Duration::Identifier(s.clone());
                }
                Value::Identifier(id) if id == "auto" => {
                    final_duration = Duration::Auto;
                }
                _ => {}
            }
        }
    }

    // ✅ Résolution de value (params, effets)
    final_value = match &stmt.value {
        Value::Identifier(ident) => {
            match module.variable_table.get(ident) {
                Some(val) => val.clone(),
                None => {
                    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
                    let message = format!(
                        "'{path}': value identifier '{ident}' not found in variable table"
                    );
                    logger.log_error_with_stacktrace(&message, &stacktrace);
                    Value::Null
                }
            }
        }
        Value::Map(map) => {
            let mut resolved_map = HashMap::new();
            for (k, v) in map.iter() {
                let resolved_v = match v {
                    Value::Identifier(id) => {
                        module.variable_table.get(id).cloned().unwrap_or(Value::Null)
                    }
                    other => other.clone(),
                };
                resolved_map.insert(k.clone(), resolved_v);
            }
            Value::Map(resolved_map)
        }
        other => other.clone(),
    };

    // ✅ On reconstruit le Statement avec Trigger résolu
    if let StatementKind::Trigger { entity, .. } = &stmt.kind {
        return Statement {
            kind: StatementKind::Trigger {
                entity: entity.to_string(),
                duration: final_duration,
            },
            value: final_value,
            line: stmt.line,
            column: stmt.column,
            indent: stmt.indent,
        };
    }

    return Statement {
        kind: StatementKind::Trigger {
            entity: entity.to_string(),
            duration: final_duration,
        },
        value: final_value,
        line: stmt.line,
        column: stmt.column,
        indent: stmt.indent,
    };
}
