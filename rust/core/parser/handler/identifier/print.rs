use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{ driver::Parser, statement::{ Statement, StatementKind } },
    store::global::GlobalStore,
};

pub fn parse_print_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore
) -> Statement {
    // consume 'print'
    parser.advance();

    let collected = parser.collect_until(|t| matches!(t.kind, TokenKind::Newline | TokenKind::EOF));
    // If single identifier, store as Identifier; else store as String of concatenated lexemes
    let value = if collected.len() == 1 && collected[0].kind == TokenKind::Identifier {
        crate::core::shared::value::Value::Identifier(collected[0].lexeme.clone())
    } else {
        let mut text = String::new();
        for t in collected.iter() {
            if matches!(t.kind, TokenKind::Newline | TokenKind::EOF) { break; }
            text.push_str(&t.lexeme);
        }
        crate::core::shared::value::Value::String(text.trim().to_string())
    };

    Statement { kind: StatementKind::Print, value, indent: current_token.indent, line: current_token.line, column: current_token.column }
}
