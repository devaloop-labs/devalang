use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

fn parse_map_literal(parser: &mut Parser) -> Value {
    // Assumes '{' has already been consumed by caller
    let mut map = std::collections::HashMap::new();
    loop {
        let Some(inner_token) = parser.peek_clone() else {
            break;
        };

        match inner_token.kind {
            TokenKind::RBrace => {
                parser.advance(); // consume '}'
                break;
            }
            TokenKind::Newline | TokenKind::Comma => {
                parser.advance();
                continue;
            }
            _ => {}
        }

        // Key
        parser.advance();
        let key = inner_token.lexeme.clone();

        // Expect ':'
        if let Some(colon_token) = parser.peek_clone() {
            if colon_token.kind == TokenKind::Colon {
                parser.advance(); // consume ':'

                // Value
                if let Some(value_token) = parser.peek_clone() {
                    match value_token.kind {
                        TokenKind::LBrace => {
                            parser.advance(); // consume '{'
                            let nested = parse_map_literal(parser);
                            map.insert(key, nested);
                        }
                        TokenKind::Identifier => {
                            parser.advance();
                            let v = if value_token.lexeme == "true" {
                                Value::Boolean(true)
                            } else if value_token.lexeme == "false" {
                                Value::Boolean(false)
                            } else {
                                Value::Identifier(value_token.lexeme.clone())
                            };
                            map.insert(key, v);
                        }
                        TokenKind::String => {
                            parser.advance();
                            map.insert(key, Value::String(value_token.lexeme.clone()));
                        }
                        TokenKind::Number => {
                            parser.advance();
                            // Beat fraction support: NUMBER '/' NUMBER
                            if let Some(TokenKind::Slash) = parser.peek_kind() {
                                parser.advance(); // '/'
                                if let Some(den) = parser.peek_clone() {
                                    if den.kind == TokenKind::Number
                                        || den.kind == TokenKind::Identifier
                                    {
                                        parser.advance();
                                        let beat = format!("{}/{}", value_token.lexeme, den.lexeme);
                                        map.insert(
                                            key,
                                            Value::Duration(devalang_types::Duration::Beat(beat)),
                                        );
                                        continue;
                                    }
                                }
                            }
                            // Decimal support NUMBER '.' NUMBER
                            if let Some(next) = parser.peek_clone() {
                                if next.kind == TokenKind::Dot {
                                    parser.advance(); // '.'
                                    if let Some(after) = parser.peek_clone() {
                                        if after.kind == TokenKind::Number {
                                            parser.advance();
                                            let combined =
                                                format!("{}.{}", value_token.lexeme, after.lexeme);
                                            map.insert(
                                                key,
                                                Value::Number(
                                                    combined.parse::<f32>().unwrap_or(0.0),
                                                ),
                                            );
                                            continue;
                                        }
                                    }
                                }
                            }
                            map.insert(
                                key,
                                Value::Number(value_token.lexeme.parse::<f32>().unwrap_or(0.0)),
                            );
                        }
                        TokenKind::Boolean => {
                            parser.advance();
                            map.insert(
                                key,
                                Value::Boolean(value_token.lexeme.parse::<bool>().unwrap_or(false)),
                            );
                        }
                        _ => {
                            // Unknown value type, consume and store Unknown
                            parser.advance();
                            map.insert(key, Value::Unknown);
                        }
                    }
                }
            }
        }
    }
    Value::Map(map)
}

pub fn parse_arrow_call(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    let Some(target_token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    if target_token.kind != TokenKind::Identifier {
        parser.advance(); // consume target token
        return Statement::unknown_with_pos(
            target_token.indent,
            target_token.line,
            target_token.column,
        );
    }

    let Some(arrow_token) = parser.peek_nth(1).cloned() else {
        parser.advance(); // consume arrow token
        return Statement::unknown_with_pos(
            target_token.indent,
            target_token.line,
            target_token.column,
        );
    };

    if arrow_token.kind != TokenKind::Arrow {
        parser.advance(); // consume method token
        return Statement::unknown_with_pos(
            target_token.indent,
            target_token.line,
            target_token.column,
        );
    }

    // We have a valid arrow call, so we consume the arrow token
    let Some(method_token) = parser.peek_nth(2).cloned() else {
        parser.advance();
        return Statement::unknown_with_pos(
            target_token.indent,
            target_token.line,
            target_token.column,
        );
    };

    if method_token.kind != TokenKind::Identifier {
        parser.advance();
        return Statement::unknown_with_pos(
            method_token.indent,
            method_token.line,
            method_token.column,
        );
    }

    // Consume the tokens for target, arrow, and method
    parser.advance(); // target
    parser.advance(); // ->
    parser.advance(); // method

    let args = parse_arrow_args(parser);
    Statement {
        kind: StatementKind::ArrowCall {
            target: target_token.lexeme.clone(),
            method: method_token.lexeme.clone(),
            args,
        },
        value: Value::Null,
        indent: target_token.indent,
        line: target_token.line,
        column: target_token.column,
    }
}

// Parse args after the method token. Reused by continuation parser.
fn parse_arrow_args(parser: &mut Parser) -> Vec<Value> {
    let mut args = Vec::new();
    let mut paren_depth = 0;
    let mut map_depth = 0;

    while let Some(token) = parser.peek_clone() {
        if token.kind == TokenKind::Newline || token.kind == TokenKind::EOF {
            break;
        }
        if token.kind == TokenKind::LParen {
            paren_depth += 1;
        }
        if token.kind == TokenKind::RParen {
            if paren_depth > 0 {
                paren_depth -= 1;
                parser.advance();
                if paren_depth == 0 {
                    break;
                }
                continue;
            } else {
                break;
            }
        }
        if token.kind == TokenKind::LBrace {
            map_depth += 1;
        }
        if token.kind == TokenKind::RBrace {
            if map_depth > 0 {
                map_depth -= 1;
                parser.advance();
                if map_depth == 0 {
                    continue;
                }
                continue;
            } else {
                break;
            }
        }

        parser.advance();

        let value = match token.kind {
            TokenKind::Identifier => Value::Identifier(token.lexeme.clone()),
            TokenKind::String => Value::String(token.lexeme.clone()),
            TokenKind::Number => {
                if let Some(TokenKind::Slash) = parser.peek_kind() {
                    parser.advance(); // consume '/'
                    if let Some(den) = parser.peek_clone() {
                        if den.kind == TokenKind::Number || den.kind == TokenKind::Identifier {
                            parser.advance();
                            let beat = format!("{}/{}", token.lexeme, den.lexeme);
                            Value::Duration(devalang_types::Duration::Beat(beat))
                        } else {
                            Value::Number(token.lexeme.parse::<f32>().unwrap_or(0.0))
                        }
                    } else {
                        Value::Number(token.lexeme.parse::<f32>().unwrap_or(0.0))
                    }
                } else {
                    Value::Number(token.lexeme.parse::<f32>().unwrap_or(0.0))
                }
            }
            TokenKind::LBrace => parse_map_literal(parser),
            _ => Value::Unknown,
        };

        args.push(value);

        if paren_depth == 0 && (token.kind == TokenKind::RParen || token.kind == TokenKind::RBrace)
        {
            break;
        }
    }

    args
}

// Parse an arrow continuation that begins with an Arrow token. If prev_target is Some,
// use it as the call target; otherwise produce an Unknown statement.
pub fn parse_arrow_continuation(
    parser: &mut Parser,
    _global_store: &mut GlobalStore,
    prev_target: Option<String>,
) -> Statement {
    // We expect current token to be Arrow
    let arrow_tok = parser.peek_clone();
    if arrow_tok.is_none() {
        return Statement::unknown();
    }

    // If there is no previous target, consume arrow and return unknown
    let Some(target) = prev_target else {
        parser.advance(); // consume Arrow
        return Statement::unknown();
    };

    // consume '->'
    parser.advance();

    // next token should be method identifier
    let Some(method_token) = parser.peek_nth(0).cloned() else {
        return Statement::unknown();
    };

    if method_token.kind != TokenKind::Identifier {
        parser.advance();
        return Statement::unknown_with_pos(
            method_token.indent,
            method_token.line,
            method_token.column,
        );
    }

    // consume method
    parser.advance();

    let args = parse_arrow_args(parser);

    Statement {
        kind: StatementKind::ArrowCall {
            target,
            method: method_token.lexeme.clone(),
            args,
        },
        value: Value::Null,
        indent: method_token.indent,
        line: method_token.line,
        column: method_token.column,
    }
}
