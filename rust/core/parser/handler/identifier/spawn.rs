use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_spawn_token(
    parser: &mut Parser,
    current_token: Token,
    global_store: &mut GlobalStore
) -> Statement {
        parser.advance(); // consume "spawn"

        let value = if let Some(token) = parser.peek_clone() {
            parser.advance();
            match token.kind {
                TokenKind::Identifier => Value::Identifier(token.lexeme.clone()),
                TokenKind::String => Value::String(token.lexeme.clone()),
                _ => {
                    return Statement::error(
                        token,
                        "Expected identifier or string after 'spawn'".to_string()
                    );
                }
            }
        } else {
            return Statement::error(
                current_token,
                "Expected identifier or string after 'spawn'".to_string()
            );
        };

        return Statement {
            kind: StatementKind::Spawn,
            value,
            indent: current_token.indent,
            line: current_token.line,
            column: current_token.column,
        };
}