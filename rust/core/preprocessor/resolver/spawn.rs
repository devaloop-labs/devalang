use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::{module::Module, resolver::group::resolve_group},
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_spawn(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &GlobalStore,
) -> Statement {
    let logger = Logger::new();

    match &stmt.value {
        Value::Identifier(ident) => {
            match module.variable_table.get(ident) {
                Some(Value::String(group_name)) => resolve_group_by_name(group_name, stmt, module, &logger),
                Some(Value::Map(group_map)) => resolved_spawn(stmt, group_map),
                Some(other) => type_error(
                    &logger,
                    module,
                    stmt,
                    format!("Identifier '{ident}' must resolve to a group name or map, found {:?}", other),
                ),
                None => type_error(
                    &logger,
                    module,
                    stmt,
                    format!("Identifier '{ident}' not found in variable table"),
                ),
            }
        }
        Value::String(name) => resolve_group_by_name(name, stmt, module, &logger),
        Value::Map(group_map) => resolved_spawn(stmt, group_map),
        other => type_error(
            &logger,
            module,
            stmt,
            format!("Spawn expects a group name as string or identifier, found {:?}", other),
        ),
    }
}

fn resolve_group_by_name<'a>(
    name: &str,
    stmt: &Statement,
    module: &'a Module,
    logger: &Logger,
) -> Statement {
    match module.variable_table.get(name) {
        Some(Value::Map(group_map)) => resolved_spawn(stmt, group_map),
        Some(other) => type_error(
            logger,
            module,
            stmt,
            format!("Expected a group for '{}', but found {:?}", name, other),
        ),
        None => type_error(
            logger,
            module,
            stmt,
            format!("Group '{}' not found in module '{}'", name, module.path),
        ),
    }
}

fn resolved_spawn(stmt: &Statement, group_map: &std::collections::HashMap<String, Value>) -> Statement {
    Statement {
        kind: StatementKind::Spawn,
        value: Value::Map(group_map.clone()),
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
