use devalang_types::Value;

use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};

pub fn parse_sleep_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume "sleep"

    // Accept number, decimal, fraction like 1/4 (-> Duration::Beat), string, or identifier
    let duration_value = if let Some(token) = parser.peek_clone() {
        match token.kind {
            TokenKind::Number => {
                let mut num = token.lexeme.clone();
                parser.advance();
                // decimal part
                if let Some(dot) = parser.peek_clone() {
                    if dot.kind == TokenKind::Dot {
                        if let Some(next) = parser.peek_nth(1).cloned() {
                            if next.kind == TokenKind::Number {
                                parser.advance(); // consume dot
                                parser.advance(); // consume next number
                                num.push('.');
                                num.push_str(&next.lexeme);
                            }
                        }
                    }
                }

                // fraction form 1/4 -> Duration::Beat("1/4")
                if let Some(slash) = parser.peek_clone() {
                    if slash.kind == TokenKind::Slash {
                        parser.advance();
                        if let Some(den) = parser.peek_clone() {
                            if den.kind == TokenKind::Number || den.kind == TokenKind::Identifier {
                                let frac = format!("{}/{}", num, den.lexeme);
                                parser.advance();
                                Value::Duration(devalang_types::Duration::Beat(frac))
                            } else {
                                return crate::core::parser::statement::error_from_token(
                                    slash,
                                    "Expected denominator after '/' in sleep".to_string(),
                                );
                            }
                        } else {
                            return crate::core::parser::statement::error_from_token(
                                slash,
                                "Expected denominator after '/' in sleep".to_string(),
                            );
                        }
                    } else {
                        Value::Number(num.parse().unwrap_or(0.0))
                    }
                } else {
                    Value::Number(num.parse().unwrap_or(0.0))
                }
            }
            TokenKind::String => {
                parser.advance();
                Value::String(token.lexeme.clone())
            }
            TokenKind::Identifier => {
                parser.advance();
                Value::Identifier(token.lexeme.clone())
            }
            _ => {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected duration after 'sleep'".to_string(),
                );
            }
        }
    } else {
        return crate::core::parser::statement::error_from_token(
            current_token,
            "Expected duration after 'sleep'".to_string(),
        );
    };

    Statement {
        kind: StatementKind::Sleep,
        value: duration_value,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
