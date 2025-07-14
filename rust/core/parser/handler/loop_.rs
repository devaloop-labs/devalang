use std::collections::HashMap;

use crate::core::{
    lexer::{ token::TokenKind },
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_loop_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'loop'
    let Some(loop_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    let Some(iterator_token) = parser.peek_clone() else {
        return Statement::error(loop_token, "Expected number or identifier after 'loop'".to_string());
    };

    let iterator_value = match iterator_token.kind {
        TokenKind::Number => {
            let val = iterator_token.lexeme.parse::<f32>().unwrap_or(1.0);
            parser.advance();
            Value::Number(val)
        }
        TokenKind::Identifier => {
            let val = iterator_token.lexeme.clone();
            parser.advance();
            Value::Identifier(val)
        }
        _ => {
            return Statement::error(
                iterator_token.clone(),
                "Expected a number or identifier as loop count".to_string()
            );
        }
    };

    // Expect colon
    let Some(colon_token) = parser.peek_clone() else {
        return Statement::error(iterator_token.clone(), "Expected ':' after loop count".to_string());
    };

    if colon_token.kind != TokenKind::Colon {
        let message = format!("Expected ':' after loop count, got {:?}", colon_token.kind);
        return Statement::error(colon_token.clone(), message);
    }

    parser.advance(); // consume ':'

    // Collect body
    let tokens = parser.collect_until(|t| t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF);
    let loop_body = parser.parse_block(tokens.clone(), global_store);

    if let Some(token) = parser.peek() {
        if token.kind == TokenKind::Dedent {
            parser.advance();
        }
    }

    let mut value_map = HashMap::new();
    value_map.insert("iterator".to_string(), iterator_value);
    value_map.insert("body".to_string(), Value::Block(loop_body.clone()));

    Statement {
        kind: StatementKind::Loop,
        value: Value::Map(value_map),
        indent: loop_token.indent,
        line: loop_token.line,
        column: loop_token.column,
    }
}
