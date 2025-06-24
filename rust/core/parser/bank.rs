use crate::core::types::{
    parser::Parser,
    statement::{ Statement, StatementKind },
    store::GlobalStore,
    token::TokenKind,
    variable::VariableValue,
};

pub fn parse_bank(
    parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    let mut bank_name = String::new();

    if parser.next().map(|t| t.kind.clone()) != Some(TokenKind::Bank) {
        return Err("Expected 'bank' keyword".to_string());
    }

    if let Some(token) = parser.next() {
        if token.kind == TokenKind::Identifier {
            bank_name = token.lexeme.clone();
        } else if token.kind == TokenKind::String {
            bank_name = token.lexeme.trim_matches('"').to_string();
        } else if token.kind == TokenKind::Number {
            bank_name = token.lexeme.clone();
        } else {
            return Err(format!("Expected bank name, found {:?}", token.kind));
        }
    } else {
        return Err("Expected bank name after 'bank' keyword".to_string());
    }

    Ok(Statement {
        kind: StatementKind::Bank,
        value: VariableValue::Text(bank_name.clone()),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
