use crate::core::{
    lexer::token::TokenKind,
    parser::{ statement::{ Statement, StatementKind }, Parser },
    shared::{ duration::Duration, value::Value },
    store::global::GlobalStore,
};

pub fn parse_dot_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume the dot token

    let Some(dot_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // .kick
    let Some(entity_token) = parser.peek_clone() else {
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
    };

    parser.advance(); // consume entity
    let entity = entity_token.lexeme.clone();

    // Check if there's a duration
    let next = parser.peek_clone();

    let (duration, value) = match next {
        // If no more tokens, it's just `.kick`
        None => (Duration::Auto, Value::Null),

        Some(token) =>
            match token.kind {
                TokenKind::Newline | TokenKind::EOF => { (Duration::Auto, Value::Null) }

                TokenKind::Number => {
                    let duration_lexeme = token.lexeme.clone();
                    parser.advance(); // consume duration

                    // Try to parse optional value (ex: .kick 250 params)
                    match parser.peek_clone() {
                        Some(param_token) if param_token.kind == TokenKind::Identifier => {
                            parser.advance();
                            (
                                parse_duration(duration_lexeme),
                                Value::Identifier(param_token.lexeme.clone()),
                            )
                        }

                        Some(param_token) if param_token.kind == TokenKind::LBrace => {
                            // Handle value as Map
                            let map = parser.parse_map_value(); // Assumes you have a helper for map
                            (parse_duration(duration_lexeme), map.unwrap_or(Value::Null))
                        }

                        _ => (parse_duration(duration_lexeme), Value::Null),
                    }
                }

                TokenKind::Identifier => {
                    let duration_lexeme = token.lexeme.clone();
                    parser.advance(); // consume duration

                    // Try to parse optional value (ex: .kick auto params)
                    match parser.peek_clone() {
                        Some(param_token) if param_token.kind == TokenKind::Identifier => {
                            parser.advance();
                            (
                                parse_duration(duration_lexeme),
                                Value::Identifier(param_token.lexeme.clone()),
                            )
                        }

                        Some(param_token) if param_token.kind == TokenKind::LBrace => {
                            // Handle value as Map
                            let map = parser.parse_map_value(); // Assumes you have a helper for map
                            (parse_duration(duration_lexeme), map.unwrap_or(Value::Null))
                        }

                        _ => (parse_duration(duration_lexeme), Value::Null),
                    }
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
    } else if s.parse::<f32>().is_ok() {
        Duration::Number(s.parse().unwrap())
    } else {
        Duration::Identifier(s)
    }
}
