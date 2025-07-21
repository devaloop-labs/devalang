use crate::core::{
    lexer::token::TokenKind,
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::{ duration::Duration, value::Value },
    store::global::GlobalStore,
};

pub fn parse_dot_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume the first dot

    let Some(dot_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Parse namespaced identifier: .808.kick.snare
    let mut parts = Vec::new();

    while let Some(token) = parser.peek_clone() {
        match token.kind {
            TokenKind::Number => {
                // Stop if it's part of a duration
                if let Some(TokenKind::Slash) = parser.peek_nth_kind(1) {
                    break;
                }

                parts.push(token.lexeme.clone());
                parser.advance();
            }

            TokenKind::Identifier => {
                // Stop parsing entity name if next token is ':' or if already have one ident and current might be a param
                if parts.len() >= 1 {
                    break; // we've already got the entity
                }

                if token.lexeme == "auto" {
                    break;
                }

                parts.push(token.lexeme.clone());
                parser.advance();
            }

            TokenKind::Dot => {
                parser.advance(); // continue chaining
            }

            _ => {
                break;
            }
        }
    }

    let entity = if parts.len() == 1 { parts[0].clone() } else { parts[..=1].join(".") };

    if entity.is_empty() {
        return Statement {
            kind: StatementKind::Trigger {
                entity: String::new(),
                duration: Duration::Auto,
            },
            value: Value::Null,
            indent: dot_token.indent,
            line: dot_token.line,
            column: dot_token.column,
        };
    }

    // Check if there's a duration
    let next = parser.peek_clone();

    let (duration, value) = match next {
        None => (Duration::Auto, Value::Null),

        Some(token) =>
            match token.kind {
                TokenKind::Newline | TokenKind::EOF => (Duration::Auto, Value::Null),

                TokenKind::Number => {
                    let numerator = token.lexeme.clone();
                    parser.advance(); // consume numerator

                    if let Some(TokenKind::Slash) = parser.peek_kind() {
                        parser.advance(); // consume slash

                        if let Some(denominator_token) = parser.peek_clone() {
                            if denominator_token.kind == TokenKind::Number {
                                let denominator = denominator_token.lexeme.clone();
                                parser.advance(); // consume denominator

                                let beat_str = format!("{}/{}", numerator, denominator);
                                let beat_duration = Duration::Beat(beat_str);

                                let val = match parser.peek_clone() {
                                    Some(param_token) if
                                        param_token.kind == TokenKind::Identifier
                                    => {
                                        parser.advance();
                                        Value::Identifier(param_token.lexeme.clone())
                                    }
                                    Some(param_token) if param_token.kind == TokenKind::LBrace => {
                                        parser.parse_map_value().unwrap_or(Value::Null)
                                    }
                                    _ => Value::Null,
                                };

                                return Statement {
                                    kind: StatementKind::Trigger {
                                        entity,
                                        duration: beat_duration,
                                    },
                                    value: val,
                                    indent: dot_token.indent,
                                    line: dot_token.line,
                                    column: dot_token.column,
                                };
                            }
                        }
                    }

                    // fallback: simple numeric duration
                    let duration = parse_duration(numerator);

                    let val = match parser.peek_clone() {
                        Some(param_token) if param_token.kind == TokenKind::Identifier => {
                            parser.advance();
                            Value::Identifier(param_token.lexeme.clone())
                        }
                        Some(param_token) if param_token.kind == TokenKind::LBrace => {
                            parser.parse_map_value().unwrap_or(Value::Null)
                        }
                        _ => Value::Null,
                    };

                    (duration, val)
                }

                TokenKind::Identifier => {
                    let duration_lexeme = token.lexeme.clone();
                    parser.advance(); // consume duration

                    let val = match parser.peek_clone() {
                        Some(param_token) if param_token.kind == TokenKind::Identifier => {
                            parser.advance();
                            Value::Identifier(param_token.lexeme.clone())
                        }
                        Some(param_token) if param_token.kind == TokenKind::LBrace => {
                            parser.parse_map_value().unwrap_or(Value::Null)
                        }
                        _ => Value::Null,
                    };

                    (parse_duration(duration_lexeme), val)
                }

                _ => (Duration::Auto, Value::Null),
            }
    };

    Statement {
        kind: StatementKind::Trigger { entity, duration },
        value,
        indent: dot_token.indent,
        line: dot_token.line,
        column: dot_token.column,
    }
}

fn parse_duration(s: String) -> Duration {
    if s == "auto" {
        Duration::Auto
    } else if let Ok(num) = s.parse::<f32>() {
        Duration::Number(num)
    } else {
        Duration::Identifier(s)
    }
}
