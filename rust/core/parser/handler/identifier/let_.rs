use std::collections::HashMap;

use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::parser::Parser,
        handler::{dot::parse_dot_token, identifier::synth::parse_synth_token},
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

pub fn parse_let_token(
    parser: &mut Parser,
    current_token: Token,
    global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume "let"

    let identifier = if let Some(token) = parser.peek_clone() {
        if token.kind == TokenKind::Identifier {
            parser.advance();
            token.lexeme.clone()
        } else {
            return crate::core::parser::statement::error_from_token(
                token,
                "Expected identifier after 'let'".to_string(),
            );
        }
    } else {
        return crate::core::parser::statement::error_from_token(
            current_token,
            "Expected identifier after 'let'".to_string(),
        );
    };

    if !parser.match_token(TokenKind::Equals) {
        return crate::core::parser::statement::error_from_token(
            current_token,
            "Expected '=' after identifier".to_string(),
        );
    }

    // If RHS begins with '$' or contains expression tokens ('+', '-', '*', '/', '(', '['),
    // collect the rest of the line as a raw expression string.
    if let Some(tok) = parser.peek_clone() {
        let line = tok.line;
        if tok.lexeme.starts_with('$')
            || matches!(
                tok.kind,
                TokenKind::Identifier | TokenKind::Number | TokenKind::LParen | TokenKind::LBracket
            )
        {
            // Collect tokens until end of the current line
            let collected = parser.collect_until(|t| {
                t.line != line || matches!(t.kind, TokenKind::Newline | TokenKind::EOF)
            });
            let mut text = String::new();
            for t in collected.iter() {
                if matches!(t.kind, TokenKind::Newline | TokenKind::EOF) {
                    break;
                }
                text.push_str(&t.lexeme);
            }
            return Statement {
                kind: StatementKind::Let { name: identifier },
                value: Value::String(text.trim().to_string()),
                indent: current_token.indent,
                line: current_token.line,
                column: current_token.column,
            };
        }
    }

    let value = match parser.peek_clone() {
        Some(token) if token.kind == TokenKind::Dot => {
            let dot_stmt = parse_dot_token(parser, global_store);
            Value::Statement(Box::new(dot_stmt))
        }
        Some(token) if token.kind == TokenKind::Synth => {
            let synth_stmt = parse_synth_token(parser, token.clone(), global_store);
            Value::Statement(Box::new(synth_stmt))
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
                    return crate::core::parser::statement::error_from_token(
                        token,
                        "Expected key identifier in map".to_string(),
                    );
                }
                parser.advance();
                let key = key_token.lexeme.clone();

                if !parser.match_token(TokenKind::Colon) {
                    let message = format!("Expected ':' after key '{}'", key);
                    return crate::core::parser::statement::error_from_token(token, message);
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
                    return crate::core::parser::statement::error_from_token(token, message);
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
            return crate::core::parser::statement::error_from_token(
                current_token,
                "Unhandled value type after '='".to_string(),
            );
        }
    };

    Statement {
        kind: StatementKind::Let { name: identifier },
        value,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
