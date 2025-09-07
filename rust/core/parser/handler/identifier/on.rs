use devalang_types::Value;

use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};

pub fn parse_on_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    // consume 'on'
    let on_tok = match parser.peek_clone() {
        Some(tok) => tok,
        None => return Statement::unknown(),
    };
    parser.advance();

    // Expect event name identifier
    let event_tok = match parser.peek_clone() {
        Some(tok) if tok.kind == TokenKind::Identifier => tok,
        Some(other) => {
            return crate::core::parser::statement::error_from_token(
                other,
                "Expected event name after 'on'".to_string(),
            );
        }
        None => {
            return crate::core::parser::statement::error_from_token(
                on_tok,
                "Expected event name after 'on'".to_string(),
            );
        }
    };
    let event_name = event_tok.lexeme.clone();
    parser.advance();

    // Optional parenthesized args on same line
    let mut args: Option<Vec<Value>> = None;
    if parser.peek_kind() == Some(TokenKind::LParen) {
        parser.advance(); // '('
        let mut collected: Vec<Value> = Vec::new();
        // Collect tokens until ')', supporting numbers and identifiers separated by comma
        while let Some(tok) = parser.peek_clone() {
            match tok.kind {
                TokenKind::RParen => {
                    parser.advance();
                    break;
                }
                TokenKind::Number => {
                    parser.advance();
                    collected.push(Value::Number(tok.lexeme.parse().unwrap_or(0.0)));
                }
                TokenKind::Identifier => {
                    parser.advance();
                    collected.push(Value::Identifier(tok.lexeme));
                }
                TokenKind::Comma => {
                    parser.advance();
                }
                TokenKind::Whitespace | TokenKind::Newline => {
                    parser.advance();
                }
                _ => {
                    break;
                }
            }
        }
        if !collected.is_empty() {
            args = Some(collected);
        }
    }

    // Expect ':' then block
    if parser.peek_kind() != Some(TokenKind::Colon) {
        return crate::core::parser::statement::error_from_token(
            event_tok,
            "Expected ':' after event name".to_string(),
        );
    }
    parser.advance(); // consume ':'

    let base_indent = on_tok.indent;
    let block_tokens = parser.collect_block_tokens(base_indent);
    // Parse body within current store context
    let body = parser.parse_block(block_tokens, _global_store);

    let stmt = Statement {
        kind: StatementKind::On {
            event: event_name,
            args,
            body,
        },
        value: Value::Null,
        indent: on_tok.indent,
        line: on_tok.line,
        column: on_tok.column,
    };

    // Register in global store for later emission
    if let StatementKind::On { event, .. } = &stmt.kind {
        _global_store.register_event_handler(event, stmt.clone());
    }

    stmt
}
