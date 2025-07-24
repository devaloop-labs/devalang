use std::collections::HashMap;

use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{
        driver::Parser,
        handler::{ dot::parse_dot_token, identifier::synth::parse_synth_token },
        statement::{ Statement, StatementKind },
    },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_let_token(
    parser: &mut Parser,
    current_token: Token,
    global_store: &mut GlobalStore
) -> Statement {
    parser.advance(); // consume "let"

    let identifier = if let Some(token) = parser.peek_clone() {
        if token.kind == TokenKind::Identifier {
            parser.advance();
            token.lexeme.clone()
        } else {
            return Statement::error(token, "Expected identifier after 'let'".to_string());
        }
    } else {
        return Statement::error(current_token, "Expected identifier after 'let'".to_string());
    };

    if !parser.match_token(TokenKind::Equals) {
        return Statement::error(current_token, "Expected '=' after identifier".to_string());
    }

    let value = match parser.peek_clone() {
        Some(token) if token.kind == TokenKind::Dot => {
            let dot_stmt = parse_dot_token(parser, global_store);
            Value::StatementKind(Box::new(dot_stmt.kind))
        }
        Some(token) if token.kind == TokenKind::Synth => {
            let synth_stmt = parse_synth_token(parser, token.clone(), global_store);
            Value::StatementKind(Box::new(synth_stmt.kind))
        }
        Some(token) if token.kind == TokenKind::Identifier => {
            parser.advance();
            Value::Identifier(token.lexeme.clone())
        }
        Some(token) if token.kind == TokenKind::String => {
            parser.advance();
            Value::String(token.lexeme.clone())
        }
        Some(token) if token.kind == TokenKind::Number => {
            parser.advance();
            Value::Number(token.lexeme.parse().unwrap_or(0.0))
        }
        Some(token) if token.kind == TokenKind::Boolean => {
            parser.advance();
            Value::Boolean(token.lexeme.parse().unwrap_or(false))
        }
        Some(token) if token.kind == TokenKind::LBrace => {
            parser.advance();

            let mut map = HashMap::new();

            while let Some(key_token) = parser.peek_clone() {
                if key_token.kind == TokenKind::RBrace {
                    parser.advance(); // consume '}'
                    break;
                }

                if key_token.kind != TokenKind::Identifier {
                    return Statement::error(token, "Expected key identifier in map".to_string());
                }
                parser.advance();
                let key = key_token.lexeme.clone();

                if !parser.match_token(TokenKind::Colon) {
                    let message = format!("Expected ':' after key '{}'", key);
                    return Statement::error(token, message);
                }

                let val = match parser.peek_clone() {
                    Some(t) if t.kind == TokenKind::Number => {
                        parser.advance();
                        Value::Number(t.lexeme.parse().unwrap_or(0.0))
                    }
                    Some(t) if t.kind == TokenKind::String => {
                        parser.advance();
                        Value::String(t.lexeme.clone())
                    }
                    Some(t) if t.kind == TokenKind::Identifier => {
                        parser.advance();
                        Value::Identifier(t.lexeme.clone())
                    }
                    _ => Value::Null,
                };

                if val == Value::Null {
                    let message = format!("Invalid value for key '{}'", key);
                    return Statement::error(token, message);
                }

                map.insert(key, val);

                if let Some(t) = parser.peek() {
                    if t.kind == TokenKind::Comma {
                        parser.advance(); // skip comma
                    }
                }
            }

            Value::Map(map)
        }
        _ => {
            return Statement::error(current_token, "Unhandled value type after '='".to_string());
        }
    };

    Statement {
        kind: StatementKind::Let { name: identifier },
        value: value,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
