use crate::core::{
    lexer::token::TokenKind,
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_bank_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'bank'

    let Some(bank_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    let bank_value = if let Some(token) = parser.peek_clone() {
        match token.kind {
            TokenKind::Identifier | TokenKind::Number => {
                parser.advance(); // consume identifier or number

                let mut value = token.lexeme.clone();

                // Support namespaced banks: <author>.<bank_name>
                if let Some(next) = parser.peek_clone() {
                    if next.kind == TokenKind::Dot {
                        parser.advance(); // consume '.'
                        if let Some(last) = parser.peek_clone() {
                            match last.kind {
                                TokenKind::Identifier | TokenKind::Number => {
                                    parser.advance();
                                    value = format!("{}.{}", value, last.lexeme);
                                    Value::String(value)
                                }
                                _ => Value::Unknown,
                            }
                        } else {
                            Value::Unknown
                        }
                    } else {
                        match token.kind {
                            TokenKind::Identifier => Value::Identifier(value),
                            TokenKind::Number => Value::Number(value.parse::<f32>().unwrap_or(0.0)),
                            _ => Value::Unknown,
                        }
                    }
                } else {
                    match token.kind {
                        TokenKind::Identifier => Value::Identifier(value),
                        TokenKind::Number => Value::Number(value.parse::<f32>().unwrap_or(0.0)),
                        _ => Value::Unknown,
                    }
                }
            }
            _ => Value::Unknown,
        }
    } else {
        return Statement::error(
            bank_token,
            "Expected identifier or number after 'bank'".to_string()
        );
    };

    Statement {
        kind: StatementKind::Bank,
        value: bank_value,
        indent: bank_token.indent,
        line: bank_token.line,
        column: bank_token.column,
    }
}
