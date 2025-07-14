use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::driver::resolve_statement },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{ Logger, LogLevel },
};

pub fn resolve_call(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    match &stmt.value {
        Value::Identifier(ident) => {
            match module.variable_table.get(ident) {
                Some(Value::String(group_name)) =>
                    resolve_group_by_name(group_name, stmt, module, path, global_store, &logger),
                Some(Value::Map(group_map)) =>
                    resolved_call(stmt, group_map, module, path, global_store),
                Some(other) =>
                    error_stmt(
                        &logger,
                        module,
                        stmt,
                        &format!(
                            "Identifier '{ident}' must resolve to a group name or map, found {:?}",
                            other
                        )
                    ),
                None =>
                    error_stmt(
                        &logger,
                        module,
                        stmt,
                        &format!("Identifier '{ident}' not found in variable table")
                    ),
            }
        }

        Value::String(name) =>
            resolve_group_by_name(name, stmt, module, path, global_store, &logger),

        Value::Map(group_map) => resolved_call(stmt, group_map, module, path, global_store),

        other =>
            error_stmt(
                &logger,
                module,
                stmt,
                &format!("Call expects a group name as string or identifier, found {:?}", other)
            ),
    }
}

fn resolve_group_by_name(
    name: &str,
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
    logger: &Logger
) -> Statement {
    match module.variable_table.get(name) {
        Some(Value::Map(group_map)) => resolved_call(stmt, group_map, module, path, global_store),
        Some(other) =>
            error_stmt(
                logger,
                module,
                stmt,
                &format!("Expected a group for '{}', but found {:?}", name, other)
            ),
        None =>
            error_stmt(
                logger,
                module,
                stmt,
                &format!("Group '{}' not found in module '{}'", name, module.path)
            ),
    }
}

fn resolved_call(
    stmt: &Statement,
    group_map: &std::collections::HashMap<String, Value>,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let mut cloned_map = group_map.clone();

    if let Some(Value::Block(stmts)) = group_map.get("body") {
        let resolved = stmts
            .iter()
            .map(|s| resolve_statement(s, module, path, global_store))
            .collect();
        cloned_map.insert("body".to_string(), Value::Block(resolved));
    }

    Statement {
        kind: StatementKind::Call,
        value: Value::Map(cloned_map),
        ..stmt.clone()
    }
}

fn error_stmt(logger: &Logger, module: &Module, stmt: &Statement, message: &str) -> Statement {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_message(LogLevel::Error, &format!("{message}\n  → at {stacktrace}"));

    Statement {
        kind: StatementKind::Error {
            message: message.to_string(),
        },
        value: Value::Null,
        ..stmt.clone()
    }
}
