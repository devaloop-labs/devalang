use crate::core::{
    lexer::token::TokenKind,
    parser::{driver::Parser, statement::{Statement, StatementKind}},
    shared::{duration::Duration, value::Value},
};

pub fn parse_dot_token(
    parser: &mut Parser,
    _global_store: &mut crate::core::store::global::GlobalStore
) -> Statement {
    parser.advance(); // consume '.'

    let Some(dot_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Parse a single entity (namespace-friendly, stops at newline)
    let mut parts = Vec::new();

    while let Some(token) = parser.peek_clone() {
        match token.kind {
            TokenKind::Identifier | TokenKind::Number => {
                parts.push(token.lexeme.clone());
                parser.advance();
                if parser.peek_kind() != Some(TokenKind::Dot) {
                    break;
                }
            }
            TokenKind::Dot => {
                parser.advance();
            }
            TokenKind::Newline | TokenKind::EOF | TokenKind::Indent | TokenKind::Dedent => {
                break; // Stop at newline or dedent
            }
            _ => {
                break;
            }
        }
    }

    // Build entity name properly
    let entity = if !parts.is_empty() {
        parts.join(".") // only join within the same line
    } else {
        eprintln!("⚠️ Empty entity after '.' at line {}", dot_token.line);
        String::new()
    };

    // Optional duration and effects map
    let mut duration = Duration::Auto;
    let mut value = Value::Null;

    if let Some(token) = parser.peek_clone() {
        match token.kind {
            TokenKind::Number => {
                let numerator = token.lexeme.clone();
                parser.advance();
                if let Some(TokenKind::Slash) = parser.peek_kind() {
                    parser.advance();
                    if let Some(denominator_token) = parser.peek_clone() {
                        if denominator_token.kind == TokenKind::Number {
                            let denominator = denominator_token.lexeme.clone();
                            parser.advance();
                            duration = Duration::Beat(format!("{}/{}", numerator, denominator));
                        }
                    }
                } else {
                    duration = parse_duration(numerator);
                }
                if let Some(next) = parser.peek_clone() {
                    if next.kind == TokenKind::LBrace {
                        value = parser.parse_map_value().unwrap_or(Value::Null);
                    }
                }
            }
            TokenKind::Identifier => {
                let id = token.lexeme.clone();
                parser.advance();
                duration = parse_duration(id);
                if let Some(next) = parser.peek_clone() {
                    if next.kind == TokenKind::LBrace {
                        value = parser.parse_map_value().unwrap_or(Value::Null);
                    }
                }
            }
            TokenKind::LBrace => {
                value = parser.parse_map_value().unwrap_or(Value::Null);
            }
            _ => {}
        }
    }

    Statement {
        kind: StatementKind::Trigger {
            entity,
            duration,
            effects: Some(value.clone()),
        },
        value: Value::Null,
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
