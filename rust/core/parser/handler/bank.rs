use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_bank_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    // consume 'bank'
    parser.advance();

    let Some(bank_tok) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Parse bank name
    let bank_value: Value = match parser.peek_clone() {
        Some(tok) => match tok.kind {
            TokenKind::Identifier | TokenKind::Number => {
                // base name
                parser.advance();
                let mut base = tok.lexeme.clone();
                // optional .suffix (identifier or number)
                if let Some(dot) = parser.peek_clone() {
                    if dot.kind == TokenKind::Dot {
                        // consume '.' and the following ident/number
                        parser.advance();
                        if let Some(suffix) = parser.peek_clone() {
                            match suffix.kind {
                                TokenKind::Identifier | TokenKind::Number => {
                                    parser.advance();
                                    base.push('.');
                                    base.push_str(&suffix.lexeme);
                                    Value::String(base)
                                }
                                _ => Value::Identifier(base),
                            }
                        } else {
                            Value::Identifier(base)
                        }
                    } else {
                        match tok.kind {
                            TokenKind::Identifier => Value::String(base),
                            TokenKind::Number => Value::Number(base.parse::<f32>().unwrap_or(0.0)),
                            _ => Value::Unknown,
                        }
                    }
                } else {
                    match tok.kind {
                        TokenKind::Identifier => Value::String(base),
                        TokenKind::Number => Value::Number(base.parse::<f32>().unwrap_or(0.0)),
                        _ => Value::Unknown,
                    }
                }
            }
            TokenKind::String => {
                parser.advance();
                Value::String(tok.lexeme.clone())
            }
            _ => Value::Unknown,
        },
        None => Value::Unknown,
    };

    if matches!(bank_value, Value::Unknown | Value::Null) {
        return Statement::error(bank_tok, "Expected a bank name".to_string());
    }

    // Optional alias: as <identifier>
    let mut alias: Option<String> = None;
    if parser.peek_is("as") {
        // consume 'as'
        parser.advance();
        let Some(next) = parser.peek_clone() else {
            return Statement::error(bank_tok, "Expected identifier after 'as'".to_string());
        };
        if next.kind != TokenKind::Identifier {
            return Statement::error(next, "Expected identifier after 'as'".to_string());
        }
        parser.advance();
        alias = Some(next.lexeme.clone());
    }

    Statement {
        kind: StatementKind::Bank { alias },
        value: bank_value,
        indent: bank_tok.indent,
        line: bank_tok.line,
        column: bank_tok.column,
    }
}
