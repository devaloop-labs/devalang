use crate::core::types::{
    parser::Parser,
    statement::{ Statement, StatementKind },
    store::GlobalStore,
    token::TokenKind,
    variable::VariableValue,
};

pub fn parse_tempo(
    parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    parser.next(); // Consume the tempo token

    // Parse the tempo value
    if let Some(token) = parser.next() {

        if token.kind == TokenKind::Number {
            let value = token.lexeme.parse::<f32>().map_err(|_| "Invalid tempo value".to_string())?;
            return Ok(Statement {
                kind: StatementKind::Tempo,
                value: VariableValue::Number(value as f32),
                indent: token.indent,
                line: token.line,
                column: token.column,
            });
        } else if token.kind == TokenKind::Identifier {
            let value = token.lexeme.clone();

            return Ok(Statement {
                kind: StatementKind::Tempo,
                value: VariableValue::Text(value),
                indent: token.indent,
                line: token.line,
                column: token.column,
            });
        }
    }

    Err(format!("Expected a number after tempo keyword, found {}", token.lexeme))
}
