use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

pub fn parse_tempo_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'bpm'

    let Some(tempo_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Expect a number or identifier
    let Some(value_token) = parser.peek_clone() else {
        return Statement::error_with_pos(
            tempo_token.indent,
            tempo_token.line,
            tempo_token.column,
            "Expected a number or identifier after 'bpm'".to_string(),
        );
    };

    let value = match value_token.kind {
        TokenKind::Number => {
            parser.advance();
            Value::Number(value_token.lexeme.parse().unwrap_or(0.0))
        }
        TokenKind::Identifier => {
            parser.advance();
            Value::Identifier(value_token.lexeme.clone())
        }
        _ => {
            return Statement::error_with_pos(
                value_token.indent,
                value_token.line,
                value_token.column,
                format!(
                    "Expected a number or identifier after 'bpm', got {:?}",
                    value_token.kind
                ),
            );
        }
    };

    Statement {
        kind: StatementKind::Tempo,
        value,
        indent: tempo_token.indent,
        line: tempo_token.line,
        column: tempo_token.column,
    }
}
