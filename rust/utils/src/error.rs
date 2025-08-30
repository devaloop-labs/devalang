use super::logger::{LogLevel, Logger};
use devalang_types::{ErrorResult, Severity, StackFrame, Statement, StatementKind, Value};
use std::collections::HashMap;

pub fn collect_errors_recursively(statements: &[Statement]) -> Vec<ErrorResult> {
    let mut errors: Vec<ErrorResult> = Vec::new();

    for stmt in statements {
        match &stmt.kind {
            StatementKind::Unknown => {
                errors.push(ErrorResult {
                    message: format!("Unknown statement at line {}:{}", stmt.line, stmt.column),
                    line: stmt.line,
                    column: stmt.column,
                    severity: Severity::Warning,
                    stack: vec![StackFrame {
                        module: None,
                        context: Some("Unknown".to_string()),
                        line: stmt.line,
                        column: stmt.column,
                    }],
                });
            }
            StatementKind::Error { message } => {
                errors.push(ErrorResult {
                    message: message.clone(),
                    line: stmt.line,
                    column: stmt.column,
                    severity: Severity::Critical,
                    stack: vec![StackFrame {
                        module: None,
                        context: Some("Error".to_string()),
                        line: stmt.line,
                        column: stmt.column,
                    }],
                });
            }
            StatementKind::Loop => {
                if let Some(body_statements) = extract_loop_body_statements(&stmt.value) {
                    let nested = collect_errors_recursively(body_statements);
                    errors.extend(nested.into_iter().map(|mut e| {
                        e.stack.insert(
                            0,
                            StackFrame {
                                module: None,
                                context: Some("loop".to_string()),
                                line: stmt.line,
                                column: stmt.column,
                            },
                        );
                        e
                    }));
                }
            }
            _ => {}
        }
    }

    errors
}

fn extract_loop_body_statements(value: &Value) -> Option<&[Statement]> {
    if let Value::Map(map) = value {
        if let Some(Value::Block(statements)) = map.get("body") {
            return Some(statements);
        }
    }
    None
}

pub fn partition_errors(errors: Vec<ErrorResult>) -> (Vec<ErrorResult>, Vec<ErrorResult>) {
    let mut warnings = Vec::new();
    let mut criticals = Vec::new();
    for e in errors {
        match e.severity {
            Severity::Warning => warnings.push(e),
            Severity::Critical => criticals.push(e),
        }
    }
    (warnings, criticals)
}

pub fn log_errors_with_stack(prefix: &str, warnings: &[ErrorResult], criticals: &[ErrorResult]) {
    let logger = Logger::new();
    if !warnings.is_empty() {
        logger.log_message(
            LogLevel::Warning,
            &format!("{}: {} warning(s)", prefix, warnings.len()),
        );
        for w in warnings {
            logger.log_message(LogLevel::Warning, &format!("- {}", w.message));
            if let Some(frame) = w.stack.first() {
                let module = frame.module.clone().unwrap_or_default();
                logger.log_message(
                    LogLevel::Debug,
                    &format!(
                        "     ↳ {}:{}:{} {}",
                        module,
                        frame.line,
                        frame.column,
                        frame.context.clone().unwrap_or_default()
                    ),
                );
            }
            if w.stack.len() > 1 {
                for (i, f) in w.stack.iter().enumerate().skip(1) {
                    let module = f.module.clone().unwrap_or_default();
                    logger.log_message(
                        LogLevel::Debug,
                        &format!(
                            "       #{} {}:{}:{} {}",
                            i,
                            module,
                            f.line,
                            f.column,
                            f.context.clone().unwrap_or_default()
                        ),
                    );
                }
            }
        }
    }
    if !criticals.is_empty() {
        logger.log_message(
            LogLevel::Error,
            &format!("{}: {} critical error(s)", prefix, criticals.len()),
        );
        for c in criticals {
            logger.log_message(LogLevel::Error, &format!("- {}", c.message));
            if let Some(frame) = c.stack.first() {
                let module = frame.module.clone().unwrap_or_default();
                logger.log_message(
                    LogLevel::Error,
                    &format!(
                        "     ↳ {}:{}:{} {}",
                        module,
                        frame.line,
                        frame.column,
                        frame.context.clone().unwrap_or_default()
                    ),
                );
            }
            if c.stack.len() > 1 {
                for (i, f) in c.stack.iter().enumerate().skip(1) {
                    let module = f.module.clone().unwrap_or_default();
                    logger.log_message(
                        LogLevel::Error,
                        &format!(
                            "       #{} {}:{}:{} {}",
                            i,
                            module,
                            f.line,
                            f.column,
                            f.context.clone().unwrap_or_default()
                        ),
                    );
                }
            }
        }
    }
}

pub fn collect_all_errors_with_modules(
    modules: &HashMap<String, Vec<Statement>>,
) -> Vec<ErrorResult> {
    let mut all = Vec::new();
    for (module_path, stmts) in modules {
        let mut errs = collect_errors_recursively(stmts);
        for e in errs.iter_mut() {
            if e.stack.is_empty() {
                e.stack.push(StackFrame {
                    module: Some(module_path.clone()),
                    context: None,
                    line: e.line,
                    column: e.column,
                });
            } else {
                if e.stack[0].module.is_none() {
                    e.stack[0].module = Some(module_path.clone());
                }
            }
        }
        all.extend(errs);
    }
    all
}
