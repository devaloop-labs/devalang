use devalang_types::Value;

use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use std::collections::HashMap;

pub fn parse_group_token(
    parser: &mut Parser,
    current_token: Token,
    global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume "group"

    let Some(identifier_token) = parser.peek_clone() else {
        return crate::core::parser::statement::error_from_token(
            current_token,
            "Expected identifier after 'group'".to_string(),
        );
    };

    if identifier_token.kind != TokenKind::Identifier && identifier_token.kind != TokenKind::String
    {
        return crate::core::parser::statement::error_from_token(
            identifier_token,
            "Expected valid identifier".to_string(),
        );
    }

    parser.advance(); // consume identifier

    let Some(colon_token) = parser.peek_clone() else {
        return crate::core::parser::statement::error_from_token(
            identifier_token,
            "Expected ':' after group identifier".to_string(),
        );
    };

    if colon_token.kind != TokenKind::Colon {
        return crate::core::parser::statement::error_from_token(
            colon_token.clone(),
            "Expected ':' after group identifier".to_string(),
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
    value_map.insert(
        "identifier".to_string(),
        Value::String(identifier_token.lexeme.clone()),
    );
    value_map.insert("body".to_string(), Value::Block(body));

    Statement {
        kind: StatementKind::Group,
        value: Value::Map(value_map),
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
