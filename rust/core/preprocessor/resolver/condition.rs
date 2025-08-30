use crate::core::{
    parser::statement::{Statement, StatementKind},
    preprocessor::{module::Module, resolver::driver::resolve_statement},
    store::global::GlobalStore,
};
use devalang_types::Value;
use devalang_utils::logger::Logger;

pub fn resolve_condition(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Statement {
    let logger = Logger::new();

    let Value::Map(condition_map) = &stmt.value else {
        return type_error(
            &logger,
            module,
            stmt,
            "Expected a map in condition statement".to_string(),
        );
    };

    let mut resolved_map = condition_map.clone();

    // Main body resolution
    if let Some(Value::Block(body)) = condition_map.get("body") {
        let resolved_body = resolve_block_statements(body, module, path, global_store);
        resolved_map.insert("body".to_string(), Value::Block(resolved_body));
    }

    // Recursive resolution of next condition
    if let Some(Value::Map(next_map)) = condition_map.get("next") {
        let next_stmt = Statement {
            kind: StatementKind::If,
            value: Value::Map(next_map.clone()),
            ..stmt.clone()
        };

        let resolved_next = resolve_condition(&next_stmt, module, path, global_store);

        if let Value::Map(mut resolved_next_map) = resolved_next.value {
            // Body next resolution
            if let Some(Value::Block(body)) = resolved_next_map.get("body") {
                let resolved_body = resolve_block_statements(body, module, path, global_store);
                resolved_next_map.insert("body".to_string(), Value::Block(resolved_body));
            }

            resolved_map.insert("next".to_string(), Value::Map(resolved_next_map));
        }
    }

    Statement {
        kind: StatementKind::If,
        value: Value::Map(resolved_map),
        ..stmt.clone()
    }
}

fn resolve_block_statements(
    body: &[Statement],
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Vec<Statement> {
    body.iter()
        .flat_map(|stmt| {
            let resolved = resolve_statement(stmt, module, path, global_store);

            if let StatementKind::Call { .. } = resolved.kind {
                if let Value::Block(inner) = &resolved.value {
                    return inner
                        .iter()
                        .map(|s| resolve_statement(s, module, path, global_store))
                        .collect();
                }
            }

            vec![resolved]
        })
        .collect()
}

fn type_error(logger: &Logger, module: &Module, stmt: &Statement, message: String) -> Statement {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_error_with_stacktrace(&message, &stacktrace);

    Statement {
        kind: StatementKind::Error { message },
        value: Value::Null,
        ..stmt.clone()
    }
}
