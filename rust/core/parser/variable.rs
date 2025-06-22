use crate::core::{
    parser::Parser,
    types::{
        statement::{ Statement, StatementKind },
        token::{ Token, TokenKind },
        variable::VariableValue,
    },
};

pub fn parse_let_statement(parser: &mut Parser) -> Result<Statement, String> {
    // "let" a déjà été consommé avant l'appel

    // Expect: Identifier (variable name)
    let name_token = parser.next().ok_or("Expected variable name after 'let'")?.clone();
    if name_token.kind != TokenKind::Identifier {
        return Err(format!("Expected variable name, found {:?}", name_token.kind));
    }

    let variable_name = name_token.lexeme.clone();

    // Expect: '='
    let equal_token = parser.next().ok_or("Expected '=' after variable name")?.clone();
    if equal_token.kind != TokenKind::Equals {
        return Err(format!("Expected '=', found {:?} after variable name", equal_token.kind));
    }

    // Expect: value
    let value_token = parser.next().ok_or("Expected value after '='")?.clone();
    let value = match value_token.kind {
        TokenKind::String => VariableValue::Text(value_token.lexeme.clone()),
        TokenKind::Number =>
            value_token.lexeme
                .parse::<f32>()
                .map(VariableValue::Number)
                .map_err(|_| "Invalid number value".to_string())?,
        TokenKind::Boolean =>
            value_token.lexeme
                .parse::<bool>()
                .map(VariableValue::Boolean)
                .map_err(|_| "Invalid boolean value".to_string())?,
        TokenKind::Identifier => VariableValue::Text(value_token.lexeme.clone()),
        _ => {
            return Err(format!("Invalid value token: {:?}", value_token.kind));
        }
    };

    // NOTE: Insert the variable into the local variable table
    parser.variable_table.variables.insert(
        variable_name.clone(),
        value.clone(),
    );

    Ok(Statement {
        kind: StatementKind::Let {
            name: variable_name,
        },
        value: value,
        indent: name_token.indent,
        line: name_token.line,
        column: name_token.column,
    })
}
