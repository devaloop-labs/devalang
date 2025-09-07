use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

pub fn parse_spawn_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume "spawn"

    // Expect function name
    let name_token = match parser.peek_clone() {
        Some(t) => t,
        None => {
            return crate::core::parser::statement::error_from_token(
                current_token,
                "Expected function name after 'spawn'".to_string(),
            );
        }
    };

    if name_token.kind != TokenKind::Identifier {
        return crate::core::parser::statement::error_from_token(
            name_token,
            "Expected function name to be an identifier".to_string(),
        );
    }

    let func_name = name_token.lexeme.clone();
    parser.advance(); // consume function name

    // Expect '('
    let mut args: Vec<Value> = Vec::new();
    if let Some(open_paren) = parser.peek_clone() {
        if open_paren.kind == TokenKind::LParen {
            parser.advance(); // consume '('

            // Collect args until ')'
            while let Some(token) = parser.peek_clone() {
                if token.kind == TokenKind::RParen {
                    parser.advance(); // consume ')'
                    break;
                }

                match token.kind {
                    TokenKind::Number => {
                        if let Ok(num) = token.lexeme.parse::<f32>() {
                            args.push(Value::Number(num));
                        }
                        parser.advance();
                    }
                    TokenKind::String => {
                        args.push(Value::String(token.lexeme.clone()));
                        parser.advance();
                    }
                    TokenKind::Identifier => {
                        args.push(Value::Identifier(token.lexeme.clone()));
                        parser.advance();
                    }
                    TokenKind::Comma => {
                        parser.advance(); // skip comma
                    }
                    _ => {
                        return crate::core::parser::statement::error_from_token(
                            token,
                            "Unexpected token in spawn arguments".to_string(),
                        );
                    }
                }
            }
        }
    }

    Statement {
        kind: StatementKind::Spawn {
            name: func_name,
            args,
        },
        value: Value::Null,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
