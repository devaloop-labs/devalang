use crate::core::parser::{ statement::{ Statement, StatementKind }, driver::Parser };
use serde::{Serialize, Deserialize};

pub struct ErrorHandler {
    errors: Vec<Error>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ErrorResult {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub struct Error {
    pub message: String,
    pub line: usize,
    pub column: usize,
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
                    self.add_error(
                        "Unknown statement".to_string(),
                        stmt.line,
                        stmt.column
                    );
                }
                StatementKind::Error { message } => {
                    self.add_error(
                        message.clone(),
                        stmt.line,
                        stmt.column
                    );
                }
                _ => {}
            }
        }
    }
}
