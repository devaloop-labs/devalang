use crate::core::{
    parser::statement::{Statement, StatementKind},
    preprocessor::{module::Module},
    store::global::GlobalStore,
};
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};

pub fn resolve_pattern(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Statement {
    let logger = Logger::new();

    // Expecting pattern name stored on the Statement.kind; value may contain the string
    if let StatementKind::Pattern { name, target } = &stmt.kind {
        // Ensure name doesn't already exist
        if global_store.variables.variables.contains_key(name) {
            logger.log_error_with_stacktrace(&format!("Pattern identifier '{}' already exists", name), path);
            return Statement {
                kind: StatementKind::Error { message: format!("Pattern '{}' already exists", name) },
                ..stmt.clone()
            };
        }

        // Resolve potential target and pattern string value
        let resolved_value = resolve_value(&stmt.value, module, global_store);

        // Build a map to store the pattern definition
        let mut map = std::collections::HashMap::new();
        map.insert("identifier".to_string(), Value::String(name.clone()));
        if let Some(t) = target {
            map.insert("target".to_string(), Value::String(t.clone()));
        }
        // Keep raw pattern in 'pattern' key
        map.insert("pattern".to_string(), resolved_value.clone());

        let resolved_stmt = Statement {
            kind: StatementKind::Pattern {
                name: name.clone(),
                target: target.clone(),
            },
            value: resolved_value,
            ..stmt.clone()
        };

        // Store into global variables as a Statement
        global_store.variables.variables.insert(
            name.clone(),
            Value::Statement(Box::new(resolved_stmt.clone())),
        );

        return resolved_stmt;
    }

    logger.log_message(LogLevel::Warning, "resolve_pattern called on non-pattern statement");
    stmt.clone()
}

fn resolve_value(value: &Value, module: &Module, global_store: &mut GlobalStore) -> Value {
    // reuse driver::resolve_value logic; simple local resolution for pattern value
    match value {
        Value::String(s) => Value::String(s.clone()),
        Value::Map(m) => {
            let mut resolved = std::collections::HashMap::new();
            for (k, v) in m {
                resolved.insert(k.clone(), resolve_value(v, module, global_store));
            }
            Value::Map(resolved)
        }
        other => other.clone(),
    }
}
