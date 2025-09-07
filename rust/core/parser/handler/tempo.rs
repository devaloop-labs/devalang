use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::parser::Parser,
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
            // support decimals and fraction forms
            let mut num = value_token.lexeme.clone();
            parser.advance();
            if let Some(dot) = parser.peek_clone() {
                if dot.kind == TokenKind::Dot {
                    if let Some(next) = parser.peek_nth(1).cloned() {
                        if next.kind == TokenKind::Number {
                            parser.advance();
                            parser.advance();
                            num.push('.');
                            num.push_str(&next.lexeme);
                        }
                    }
                }
            }

            if let Some(slash) = parser.peek_clone() {
                if slash.kind == TokenKind::Slash {
                    parser.advance();
                    if let Some(den) = parser.peek_clone() {
                        if den.kind == TokenKind::Number || den.kind == TokenKind::Identifier {
                            let frac = format!("{}/{}", num, den.lexeme);
                            parser.advance();
                            Value::Duration(devalang_types::Duration::Beat(frac))
                        } else {
                            return Statement::error_with_pos(
                                slash.indent,
                                slash.line,
                                slash.column,
                                "Expected denominator after '/' in bpm".to_string(),
                            );
                        }
                    } else {
                        return Statement::error_with_pos(
                            slash.indent,
                            slash.line,
                            slash.column,
                            "Expected denominator after '/' in bpm".to_string(),
                        );
                    }
                } else {
                    Value::Number(num.parse().unwrap_or(0.0))
                }
            } else {
                Value::Number(num.parse().unwrap_or(0.0))
            }
        }
        TokenKind::Identifier => {
            parser.advance();
            Value::Identifier(value_token.lexeme.clone())
        }
        TokenKind::String => {
            parser.advance();
            Value::String(value_token.lexeme.clone())
        }
        _ => {
            return Statement::error_with_pos(
                value_token.indent,
                value_token.line,
                value_token.column,
                format!(
                    "Expected a number, string or identifier after 'bpm', got {:?}",
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
