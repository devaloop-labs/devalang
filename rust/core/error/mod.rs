use crate::core::parser::{
    driver::parser::Parser,
    statement::{Statement, StatementKind},
};
use devalang_types::Value;
use serde::{Deserialize, Serialize};

pub struct ErrorHandler {
    errors: Vec<Error>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Severity {
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StackFrame {
    pub module: Option<String>,
    pub context: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorResult {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub stack: Vec<StackFrame>,
}

#[derive(Clone)]
pub struct Error {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, message: String, line: usize, column: usize) {
        let error_statement = Error {
            message,
            line,
            column,
        };
        self.errors.push(error_statement);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn detect_from_statements(&mut self, _parser: &mut Parser, statements: &[Statement]) {
        for stmt in statements {
            match &stmt.kind {
                StatementKind::Unknown => {
                    self.add_error("Unknown statement".to_string(), stmt.line, stmt.column);
                }
                StatementKind::Error { message } => {
                    self.add_error(message.clone(), stmt.line, stmt.column);
                }
                _ => {}
            }
        }
    }
}

/// Collects errors recursively from statements (mirrors old utils implementation).
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
    use devalang_utils::logger::LogLevel;
    use devalang_utils::logger::Logger;

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

/// Collects errors from all modules and annotates stack frames with module names.
pub fn collect_all_errors_with_modules(
    statements_by_module: &std::collections::HashMap<String, Vec<Statement>>,
) -> Vec<ErrorResult> {
    let mut all: Vec<ErrorResult> = Vec::new();
    for (module, stmts) in statements_by_module.iter() {
        let mut errs = collect_errors_recursively(stmts);
        for e in errs.iter_mut() {
            // annotate first stack frame module if missing
            if let Some(first) = e.stack.first_mut() {
                if first.module.is_none() {
                    first.module = Some(module.clone());
                }
            }
        }
        all.extend(errs.into_iter());
    }
    all
}
