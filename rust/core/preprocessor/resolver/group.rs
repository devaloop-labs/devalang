use crate::core::{
    parser::statement::{Statement, StatementKind},
    preprocessor::{module::Module, resolver::driver::resolve_statement},
    store::global::GlobalStore,
};
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};

pub fn resolve_group(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Statement {
    let logger = Logger::new();

    // Extract identifier from several allowed shapes (map.identifier, bare string, number -> to_string)
    let identifier = match extract_group_identifier(stmt, &logger, module) {
        Ok(id) => id,
        Err(err_stmt) => return err_stmt,
    };

    // group_map: if the value is a map we clone it, otherwise create an empty map to hold body
    let group_map = match &stmt.value {
        Value::Map(m) => m.clone(),
        _ => std::collections::HashMap::new(),
    };

    // Ensure the identifier does not already exist
    if global_store.variables.variables.contains_key(&identifier) {
        return type_error(
            &logger,
            module,
            stmt,
            format!("Group identifier '{}' already exists", identifier),
        );
    }

    // Resolve statements in the body
    let mut resolved_map = group_map.clone();
    if let Some(Value::Block(body)) = group_map.get("body") {
        let resolved_body = resolve_block_statements(body, module, path, global_store);
        resolved_map.insert("body".to_string(), Value::Block(resolved_body));
    } else {
        logger.log_message(LogLevel::Warning, "Group without a body");
    }

    // Build a complete Statement for the group
    let resolved_group_stmt = Statement {
        kind: StatementKind::Group,
        value: Value::Map(resolved_map.clone()),
        ..stmt.clone()
    };

    // Store the Statement directly in the global variable_table
    global_store.variables.variables.insert(
        identifier.clone(),
        Value::Statement(Box::new(resolved_group_stmt.clone())),
    );

    resolved_group_stmt
}

fn resolve_block_statements(
    body: &[Statement],
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Vec<Statement> {
    body.iter()
        .map(|stmt| resolve_statement(stmt, module, path, global_store))
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

// Helper to extract a group identifier from multiple Value forms
fn extract_group_identifier(
    stmt: &Statement,
    logger: &Logger,
    module: &Module,
) -> Result<String, Statement> {
    match &stmt.value {
        Value::Map(map) => match map.get("identifier") {
            Some(Value::String(s)) => Ok(s.clone()),
            Some(Value::Identifier(s)) => Ok(s.clone()),
            Some(Value::Number(n)) => Ok(n.to_string()),
            Some(other) => Err(type_error(
                logger,
                module,
                stmt,
                format!("Unsupported type for 'identifier': {:?}", other),
            )),
            None => Err(type_error(
                logger,
                module,
                stmt,
                "Group statement must have an 'identifier' field".to_string(),
            )),
        },
        Value::String(s) => Ok(s.clone()),
        Value::Identifier(s) => Ok(s.clone()),
        other => Err(type_error(
            logger,
            module,
            stmt,
            format!(
                "Expected a map or string for group statement, found {:?}",
                other
            ),
        )),
    }
}
