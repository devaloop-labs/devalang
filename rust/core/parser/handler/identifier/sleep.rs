use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_sleep_token(
    parser: &mut Parser,
    current_token: Token,
    global_store: &mut GlobalStore
) -> Statement {
    parser.advance(); // consume "sleep"

    let duration = if let Some(token) = parser.peek_clone() {
        if token.kind == TokenKind::Number {
            parser.advance();
            token.lexeme.parse().unwrap_or(0.0)
        } else {
            return Statement::error(token, "Expected number after 'sleep'".to_string());
        }
    } else {
        return Statement::error(current_token, "Expected number after 'sleep'".to_string());
    };

    return Statement {
        kind: StatementKind::Sleep,
        value: Value::Number(duration),
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    };
}
