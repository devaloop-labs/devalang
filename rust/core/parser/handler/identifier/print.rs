use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::parser::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
use devalang_types::Value;

pub fn parse_print_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    // consume 'print'
    parser.advance();

    let collected = parser.collect_until(|t| matches!(t.kind, TokenKind::Newline | TokenKind::EOF));
    // Accept: print <identifier|string|number|expression>
    let value = if collected.len() == 1 {
        match collected[0].kind {
            TokenKind::Identifier => Value::Identifier(collected[0].lexeme.clone()),
            TokenKind::String => Value::String(collected[0].lexeme.clone()),
            TokenKind::Number => {
                let n = collected[0].lexeme.parse::<f32>().unwrap_or(0.0);
                Value::Number(n)
            }
            _ => Value::String(collected[0].lexeme.clone()),
        }
    } else {
        // Join tokens with spaces to preserve readability for expressions/text
        let text = collected
            .iter()
            .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::EOF))
            .map(|t| t.lexeme.clone())
            .collect::<Vec<_>>()
            .join(" ");
        Value::String(text.trim().to_string())
    };

    Statement {
        kind: StatementKind::Print,
        value,
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
