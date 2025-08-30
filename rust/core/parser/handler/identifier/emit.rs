use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

pub fn parse_emit_token(
    parser: &mut Parser,
    current: crate::core::lexer::token::Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume 'emit'

    let Some(ev) = parser.peek_clone() else {
        return crate::core::parser::statement::error_from_token(
            current,
            "Expected event name after 'emit'".into(),
        );
    };
    if ev.kind != TokenKind::Identifier {
        return crate::core::parser::statement::error_from_token(
            ev.clone(),
            "Expected identifier as event name".into(),
        );
    }
    let event_name = ev.lexeme.clone();
    parser.advance(); // consume event name

    // Optional payload on same line: number|string|identifier|map|array
    let mut payload: Option<Value> = None;
    if let Some(tok) = parser.peek_clone() {
        if tok.line == ev.line {
            let val = match tok.kind {
                TokenKind::String => {
                    parser.advance();
                    Value::String(tok.lexeme.clone())
                }
                TokenKind::Number => {
                    parser.advance();
                    Value::Number(tok.lexeme.parse().unwrap_or(0.0))
                }
                TokenKind::Identifier => {
                    parser.advance();
                    Value::Identifier(tok.lexeme.clone())
                }
                TokenKind::LBrace => parser.parse_map_value().unwrap_or(Value::Null),
                TokenKind::LBracket => parser.parse_array_value().unwrap_or(Value::Null),
                _ => Value::Null,
            };
            if val != Value::Null {
                payload = Some(val);
            }
        }
    }

    Statement {
        kind: StatementKind::Emit {
            event: event_name,
            payload: payload.clone(),
        },
        value: payload.unwrap_or(Value::Null),
        indent: current.indent,
        line: current.line,
        column: current.column,
    }
}
