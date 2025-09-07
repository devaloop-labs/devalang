use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;
use std::collections::HashMap;

pub fn parse_condition_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'if'
    let Some(if_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    let Some(condition) = parser.parse_condition_until_colon() else {
        return crate::core::parser::statement::error_from_token(
            if_token,
            "Expected condition after 'if'".to_string(),
        );
    };

    parser.advance_if(TokenKind::Colon);
    let base_indent = if_token.indent;

    let if_body = parser.parse_block_until_else_or_dedent(base_indent, global_store);

    let mut root_map = HashMap::new();
    root_map.insert("condition".to_string(), condition);
    root_map.insert("body".to_string(), Value::Block(if_body));

    let mut current = &mut root_map;

    // Loop for else / else if
    while let Some(tok) = parser.peek_clone() {
        // Only continue if we see `else` at same indent level
        if tok.lexeme != "else" || tok.indent != base_indent {
            break;
        }

        parser.advance(); // consume 'else'

        // Check if it's an 'else if'
        let next_condition = if parser.peek_is("if") {
            parser.advance(); // consume 'if'
            let Some(cond) = parser.parse_condition_until_colon() else {
                return crate::core::parser::statement::error_from_token(
                    tok.clone(),
                    "Expected condition after 'else if'".to_string(),
                );
            };
            parser.advance_if(TokenKind::Colon);
            Some(cond)
        } else {
            parser.advance_if(TokenKind::Colon);
            None
        };

        let body = parser.parse_block_until_else_or_dedent(base_indent, global_store);

        let mut next_map = HashMap::new();
        if let Some(cond) = next_condition {
            next_map.insert("condition".to_string(), cond);
        }
        next_map.insert("body".to_string(), Value::Block(body));

        current.insert("next".to_string(), Value::Map(next_map));
        current = match current.get_mut("next") {
            Some(Value::Map(map)) => map,
            _ => break,
        };
    }

    Statement {
        kind: StatementKind::If,
        value: Value::Map(root_map),
        indent: if_token.indent,
        line: if_token.line,
        column: if_token.column,
    }
}
