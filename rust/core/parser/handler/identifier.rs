use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{ statement::{ Statement, StatementKind }, Parser },
    shared::value::Value,
    store::global::GlobalStore,
};
use std::collections::HashMap;

pub fn parse_identifier_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    let Some(current_token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    if current_token.lexeme == "let" {
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
            Some(token) if token.kind == TokenKind::LBrace => {
                parser.advance(); // consume '{'
                let mut map = HashMap::new();

                while let Some(key_token) = parser.peek_clone() {
                    if key_token.kind == TokenKind::RBrace {
                        parser.advance(); // consume '}'
                        break;
                    }

                    if key_token.kind != TokenKind::Identifier {
                        return Statement::error(
                            token,
                            "Expected key identifier in map".to_string()
                        );
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
                        _ => { Value::Null }
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
            other => {
                let message = format!("Unexpected value token in let: {:?}", other);
                return Statement::error(current_token, message);
            }
        };

        return Statement {
            kind: StatementKind::Let { name: identifier },
            value,
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    } else if current_token.lexeme == "group" {
        parser.advance(); // consume "group"

        let Some(identifier_token) = parser.peek_clone() else {
            return Statement::error(current_token, "Expected identifier after 'group'".to_string());
        };

        if
            identifier_token.kind != TokenKind::Identifier &&
            identifier_token.kind != TokenKind::String
        {
            return Statement::error(identifier_token, "Expected valid identifier".to_string());
        }

        parser.advance(); // consume identifier

        let Some(colon_token) = parser.peek_clone() else {
            return Statement::error(
                identifier_token,
                "Expected ':' after group identifier".to_string()
            );
        };

        if colon_token.kind != TokenKind::Colon {
            return Statement::error(
                colon_token.clone(),
                "Expected ':' after group identifier".to_string()
            );
        }

        parser.advance(); // consume ':'

        let base_indent = current_token.indent;

        // Clone without consuming tokens
        let mut index = parser.token_index;
        let mut tokens_inside_group = Vec::new();

        while index < parser.tokens.len() {
            let token = parser.tokens[index].clone();

            if token.indent <= base_indent && token.kind != TokenKind::Newline {
                break;
            }

            tokens_inside_group.push(token);
            index += 1;
        }

        // Advance index once to skip the processed tokens
        parser.token_index = index;

        let body = parser.parse_block(tokens_inside_group, global_store);

        let mut value_map = HashMap::new();
        value_map.insert("identifier".to_string(), Value::String(identifier_token.lexeme.clone()));
        value_map.insert("body".to_string(), Value::Block(body));

        return Statement {
            kind: StatementKind::Group,
            value: Value::Map(value_map),
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    } else if current_token.lexeme == "call" {
        parser.advance(); // consume "call"

        let identifier = if let Some(token) = parser.peek_clone() {
            if token.kind == TokenKind::Identifier {
                parser.advance();
                token.lexeme.clone()
            } else {
                return Statement::error(token, "Expected identifier after 'call'".to_string());
            }
        } else {
            return Statement::error(current_token, "Expected identifier after 'call'".to_string());
        };

        return Statement {
            kind: StatementKind::Call,
            value: Value::String(identifier),
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    } else if current_token.lexeme == "spawn" {
        parser.advance(); // consume "spawn"

        let identifier = if let Some(token) = parser.peek_clone() {
            if token.kind == TokenKind::Identifier {
                parser.advance();
                token.lexeme.clone()
            } else {
                return Statement::error(token, "Expected identifier after 'spawn'".to_string());
            }
        } else {
            return Statement::error(current_token, "Expected identifier after 'spawn'".to_string());
        };

        return Statement {
            kind: StatementKind::Spawn,
            value: Value::String(identifier),
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    } else if current_token.lexeme == "sleep" {
        parser.advance(); // consume "sleep"

        let duration = if let Some(token) = parser.peek_clone() {
            if token.kind == TokenKind::Number {
                parser.advance();
                token.lexeme.parse().unwrap_or(0.0)
            } else {
                return Statement::error(token, "Expected number after 'sleep'".to_string());
            }
        } else {
            return Statement::error(current_token, "Expected number after 'sleep'".to_string());
        };

        return Statement {
            kind: StatementKind::Sleep,
            value: Value::Number(duration),
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    } else {
        // Unknown identifier handling
        Statement {
            kind: StatementKind::Unknown,
            value: Value::String(current_token.lexeme.clone()),
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
    }

    parser.advance(); // unknown identifier fallback

    Statement {
        kind: StatementKind::Unknown,
        value: Value::String(current_token.lexeme.clone()),
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
