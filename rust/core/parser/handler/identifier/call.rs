use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_call_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume "call"

    // Expect function name
    let name_token = match parser.peek_clone() {
        Some(t) => t,
        None => {
            return Statement::error(
                current_token,
                "Expected function name after 'call'".to_string(),
            );
        }
    };

    if name_token.kind != TokenKind::Identifier {
        return Statement::error(
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
                        return Statement::error(
                            token,
                            "Unexpected token in call arguments".to_string(),
                        );
                    }
                }
            }
        }
    }

    Statement {
        kind: StatementKind::Call {
            name: func_name,
            args,
        },
        value: Value::Null,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
