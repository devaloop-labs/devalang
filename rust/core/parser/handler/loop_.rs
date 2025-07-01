use std::collections::HashMap;

use crate::core::{
    lexer::{ token::TokenKind },
    parser::{ statement::{ Statement, StatementKind }, Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_loop_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'loop'

    let Some(loop_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Expect an identifier (iterator)
    let Some(iterator_token) = parser.peek_clone() else {
        return Statement::error(loop_token, "Expected identifier after 'loop'".to_string());
    };

    let iterator_name = iterator_token.lexeme.clone();
    parser.advance(); // consume iterator

    // Expect colon
    let Some(colon_token) = parser.peek_clone() else {
        return Statement::error(iterator_token.clone(), "Expected ':' after iterator".to_string());
    };

    if colon_token.kind != TokenKind::Colon {
        let message = format!("Expected ':' after iterator, got {:?}", colon_token.kind);
        return Statement::error(colon_token.clone(), message);
    }

    parser.advance(); // consume ':'

    // Collect all indented statements
    let tokens = parser.collect_until(
        |t| (t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF)
    );
    let loop_body = parser.parse_block(tokens.clone(), global_store);

    let mut value_map = HashMap::new();

    value_map.insert("iterator".to_string(), Value::Identifier(iterator_name));
    value_map.insert("body".to_string(), Value::Block(loop_body.clone()));

    Statement {
        kind: StatementKind::Loop,
        value: Value::Map(value_map),
        indent: loop_token.indent,
        line: loop_token.line,
        column: loop_token.column,
    }
}
