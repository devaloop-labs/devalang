use crate::core::lexer::token::Token;

pub use devalang_types::{Duration, Statement, StatementKind, Value};

pub fn unknown_from_token(token: &Token) -> Statement {
    Statement::unknown_with_pos(token.indent, token.line, token.column)
}

pub fn error_from_token(token: Token, message: String) -> Statement {
    Statement::error_with_pos(token.indent, token.line, token.column, message)
}
