use crate::core::{
    lexer::token::TokenKind,
    parser::{ driver::Parser, statement::{ Statement, StatementKind } },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_arrow_call(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    let Some(target_token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    if target_token.kind != TokenKind::Identifier {
        parser.advance(); // consume target token
        return Statement::unknown();
    }

    let Some(arrow_token) = parser.peek_nth(1).cloned() else {
        parser.advance(); // consume arrow token
        return Statement {
            kind: StatementKind::Unknown,
            value: Value::String(target_token.lexeme.clone()),
            indent: target_token.indent,
            line: target_token.line,
            column: target_token.column,
        };
    };

    if arrow_token.kind != TokenKind::Arrow {
        parser.advance(); // consume method token
        return Statement {
            kind: StatementKind::Unknown,
            value: Value::String(target_token.lexeme.clone()),
            indent: target_token.indent,
            line: target_token.line,
            column: target_token.column,
        };
    }

    // We have a valid arrow call, so we consume the arrow token
    let Some(method_token) = parser.peek_nth(2).cloned() else {
        parser.advance();
        return Statement::unknown();
    };

    if method_token.kind != TokenKind::Identifier {
        parser.advance();
        return Statement::unknown();
    }

    // Consume the tokens for target, arrow, and method
    parser.advance(); // target
    parser.advance(); // ->
    parser.advance(); // method

    let mut args = Vec::new();

    while let Some(token) = parser.peek_clone() {
        if token.kind == TokenKind::Newline || token.kind == TokenKind::EOF {
            break;
        }

        parser.advance();

        let value = match token.kind {
            TokenKind::Identifier => Value::Identifier(token.lexeme.clone()),
            TokenKind::String => Value::String(token.lexeme.clone()),
            TokenKind::Number => Value::Number(token.lexeme.parse::<f32>().unwrap_or(0.0)),
            TokenKind::LBrace => {
                // Handle map literal
                let mut map = std::collections::HashMap::new();
                while let Some(inner_token) = parser.peek_clone() {
                    if inner_token.kind == TokenKind::RBrace {
                        parser.advance(); // consume RBrace
                        break;
                    }
                    if inner_token.kind == TokenKind::Newline || inner_token.kind == TokenKind::EOF {
                        break;
                    }
                    parser.advance(); // consume key token
                    let key = inner_token.lexeme.clone();

                    if let Some(colon_token) = parser.peek_clone() {
                        if colon_token.kind == TokenKind::Colon {
                            parser.advance(); // consume colon
                            if let Some(value_token) = parser.peek_clone() {
                                parser.advance(); // consume value token
                                let value = match value_token.kind {
                                    TokenKind::Identifier =>
                                        Value::Identifier(value_token.lexeme.clone()),
                                    TokenKind::String => Value::String(value_token.lexeme.clone()),
                                    TokenKind::Number =>
                                        Value::Number(
                                            value_token.lexeme.parse::<f32>().unwrap_or(0.0)
                                        ),
                                    TokenKind::Boolean =>
                                        Value::Boolean(
                                            value_token.lexeme.parse::<bool>().unwrap_or(false)
                                        ),
                                    _ => Value::Unknown,
                                };
                                map.insert(key, value);
                            }
                        }
                    }
                }
                Value::Map(map)
            }
            _ => Value::Unknown,
        };

        args.push(value);
    }

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
