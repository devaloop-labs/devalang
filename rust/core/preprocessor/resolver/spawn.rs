use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::module::Module,
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_spawn(
    stmt: &Statement,
    module: &Module,
    _path: &str,
    _global_store: &GlobalStore,
) -> Statement {
    let logger = Logger::new();

    let Value::String(name) = &stmt.value else {
        return type_error(&logger, module, stmt, "Spawn expects a group name as string.".to_string());
    };

    match module.variable_table.variables.get(name) {
        Some(Value::Map(group_stmt)) => Statement {
            kind: StatementKind::Spawn,
            value: Value::Map(group_stmt.clone()),
            ..stmt.clone()
        },
        Some(other) => type_error(
            &logger,
            module,
            stmt,
            format!("Expected a group for '{}', but found {:?}", name, other),
        ),
        None => type_error(
            &logger,
            module,
            stmt,
            format!("Group '{}' not found in module '{}'", name, module.path),
        ),
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
