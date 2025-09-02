use devalang_types::Value;

use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};

pub fn parse_loop_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'loop' or 'for' (aliased in lexer)
    let Some(loop_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Support two forms:
    // 1) loop <count>:
    // 2) for <ident> in [a,b,c]:

    // Peek next to decide
    let Some(next_token) = parser.peek_clone() else {
        return Statement::error_with_pos(
            loop_token.indent,
            loop_token.line,
            loop_token.column,
            "Expected iterator after loop/for".to_string(),
        );
    };

    // Try to detect 'for <ident> in [array]:' form
    let mut foreach_ident: Option<String> = None;
    if let TokenKind::Identifier = next_token.kind {
        // Could be either count identifier (old form) or foreach variable
        // Look ahead for 'in'
        let name = next_token.lexeme.clone();
        // don't consume yet; we'll branch
        if let Some(t2) = parser.peek_nth(1) {
            if t2.kind == TokenKind::Identifier && t2.lexeme == "in" {
                // foreach form
                foreach_ident = Some(name);
                // consume ident and 'in'
                parser.advance();
                parser.advance();
            }
        }
    }

    if let Some(var_name) = foreach_ident {
        // Expect [array] OR number OR string OR identifier after 'in'
        let array_val = if let Some(tok) = parser.peek_clone() {
            match tok.kind {
                TokenKind::LBracket => {
                    if let Some(v) = parser.parse_array_value() {
                        v
                    } else {
                        Value::Array(vec![])
                    }
                }
                TokenKind::Number => {
                    parser.advance();
                    let n = tok.lexeme.parse::<f32>().unwrap_or(0.0);
                    Value::Number(n)
                }
                TokenKind::String => {
                    parser.advance();
                    Value::String(tok.lexeme.clone())
                }
                TokenKind::Identifier => {
                    parser.advance();
                    Value::Identifier(tok.lexeme.clone())
                }
                _ => {
                    return Statement::error_with_pos(
                        loop_token.indent,
                        loop_token.line,
                        loop_token.column,
                        "Expected array, number, string or identifier after 'in'".to_string(),
                    );
                }
            }
        } else {
            return Statement::error_with_pos(
                loop_token.indent,
                loop_token.line,
                loop_token.column,
                "Expected array, number, string or identifier after 'in'".to_string(),
            );
        };

        // Expect ':'
        if !parser.match_token(TokenKind::Colon) {
            return Statement::error_with_pos(
                loop_token.indent,
                loop_token.line,
                loop_token.column,
                "Expected ':' after foreach header".to_string(),
            );
        }

        let tokens =
            parser.collect_until(|t| t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF);
        let loop_body = parser.parse_block(tokens.clone(), global_store);
        if let Some(token) = parser.peek() {
            if token.kind == TokenKind::Dedent {
                parser.advance();
            }
        }

        let mut value_map = std::collections::HashMap::new();
        value_map.insert("foreach".to_string(), Value::Identifier(var_name));
        value_map.insert("array".to_string(), array_val);
        value_map.insert("body".to_string(), Value::Block(loop_body.clone()));

        return Statement {
            kind: StatementKind::Loop,
            value: Value::Map(value_map),
            indent: loop_token.indent,
            line: loop_token.line,
            column: loop_token.column,
        };
    }

    // Fallback to legacy: loop <count>:
    let Some(iterator_token) = parser.peek_clone() else {
        return Statement::error_with_pos(
            loop_token.indent,
            loop_token.line,
            loop_token.column,
            "Expected number or identifier after 'loop'".to_string(),
        );
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
        TokenKind::String => {
            // strings that are numeric (e.g. "10")
            let s = iterator_token.lexeme.clone();
            parser.advance();
            Value::String(s)
        }
        _ => {
            return Statement::error_with_pos(
                iterator_token.clone().indent,
                iterator_token.clone().line,
                iterator_token.clone().column,
                "Expected a number, string or identifier as loop count".to_string(),
            );
        }
    };

    if !parser.match_token(TokenKind::Colon) {
        let message = format!(
            "Expected ':' after loop count, got {:?}",
            parser.peek_kind()
        );
        return Statement::error_with_pos(
            loop_token.clone().indent,
            loop_token.clone().line,
            loop_token.clone().column,
            message,
        );
    }

    let tokens =
        parser.collect_until(|t| t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF);
    let loop_body = parser.parse_block(tokens.clone(), global_store);
    if let Some(token) = parser.peek() {
        if token.kind == TokenKind::Dedent {
            parser.advance();
        }
    }

    let mut value_map = std::collections::HashMap::new();
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
