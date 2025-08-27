use crate::core::{
    lexer::{ token::TokenKind },
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
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
        return Statement::error(loop_token, "Expected iterator after loop/for".to_string());
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
        // Expect array literal
        let array_val = if let Some(v) = parser.parse_array_value() { v } else {
            return Statement::error(loop_token, "Expected array literal after 'in'".to_string());
        };

        // Expect ':'
        if !parser.match_token(TokenKind::Colon) {
            return Statement::error(loop_token, "Expected ':' after foreach header".to_string());
        }

        let tokens = parser.collect_until(|t| t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF);
        let loop_body = parser.parse_block(tokens.clone(), global_store);
        if let Some(token) = parser.peek() { if token.kind == TokenKind::Dedent { parser.advance(); } }

        let mut value_map = std::collections::HashMap::new();
        value_map.insert("foreach".to_string(), Value::Identifier(var_name));
        value_map.insert("array".to_string(), array_val);
        value_map.insert("body".to_string(), Value::Block(loop_body.clone()));

        return Statement { kind: StatementKind::Loop, value: Value::Map(value_map), indent: loop_token.indent, line: loop_token.line, column: loop_token.column };
    }

    // Fallback to legacy: loop <count>:
    let Some(iterator_token) = parser.peek_clone() else {
        return Statement::error(loop_token, "Expected number or identifier after 'loop'".to_string());
    };

    let iterator_value = match iterator_token.kind {
        TokenKind::Number => { let val = iterator_token.lexeme.parse::<f32>().unwrap_or(1.0); parser.advance(); Value::Number(val) }
        TokenKind::Identifier => { let val = iterator_token.lexeme.clone(); parser.advance(); Value::Identifier(val) }
        _ => { return Statement::error(iterator_token.clone(), "Expected a number or identifier as loop count".to_string()); }
    };

    if !parser.match_token(TokenKind::Colon) {
        let message = format!("Expected ':' after loop count, got {:?}", parser.peek_kind());
        return Statement::error(loop_token.clone(), message);
    }

    let tokens = parser.collect_until(|t| t.kind == TokenKind::Dedent || t.kind == TokenKind::EOF);
    let loop_body = parser.parse_block(tokens.clone(), global_store);
    if let Some(token) = parser.peek() { if token.kind == TokenKind::Dedent { parser.advance(); } }

    let mut value_map = std::collections::HashMap::new();
    value_map.insert("iterator".to_string(), iterator_value);
    value_map.insert("body".to_string(), Value::Block(loop_body.clone()));

    Statement { kind: StatementKind::Loop, value: Value::Map(value_map), indent: loop_token.indent, line: loop_token.line, column: loop_token.column }
}
