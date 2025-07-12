use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{module::Module, resolver::driver::resolve_statement},
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_condition(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &GlobalStore
) -> Statement {
    let logger = Logger::new();

    let Value::Map(condition_map) = &stmt.value else {
        return type_error(&logger, module, stmt, "Expected a map in condition statement".to_string());
    };

    let mut resolved_map = condition_map.clone();

    // Body resolution
    if let Some(Value::Block(body)) = condition_map.get("body") {
        let resolved_body = body
            .iter()
            .map(|s| resolve_statement(s, module, path, global_store))
            .collect::<Vec<_>>();

        resolved_map.insert("body".to_string(), Value::Block(resolved_body));
    }

    // Next resolution
    if let Some(Value::Map(next)) = condition_map.get("next") {
        let next_stmt = Statement {
            kind: StatementKind::If,
            value: Value::Map(next.clone()),
            ..stmt.clone()
        };

        let resolved_next = resolve_condition(&next_stmt, module, path, global_store);

        if let Value::Map(resolved_next_map) = resolved_next.value {
            resolved_map.insert("next".to_string(), Value::Map(resolved_next_map));
        }
    }

    Statement {
        kind: StatementKind::If,
        value: Value::Map(resolved_map),
        ..stmt.clone()
    }
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
