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
            TokenKind::Identifier => {
                parser.advance();
                Value::Identifier(token.lexeme.clone())
            }
            TokenKind::Number => {
                parser.advance();
                Value::Number(token.lexeme.parse::<f32>().unwrap_or(0.0))
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
