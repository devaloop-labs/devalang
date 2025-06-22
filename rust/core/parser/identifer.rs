use crate::core::{
    parser::{ variable::parse_let_statement, Parser },
    types::{
        statement::{ Statement, StatementKind },
        token::{ Token, TokenKind },
        variable::VariableValue,
    },
};

pub fn parse_identifier(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    // Ne consomme rien ici : on vérifie seulement
    if token.kind != TokenKind::Identifier {
        return Err(format!("Expected Identifier, found {:?}", token.kind));
    }

    match token.lexeme.as_str() {
        "let" => {
            parser.next(); // consomme "let"
            return parse_let_statement(parser); // consomme le reste
        },
        _ => {
            parser.next(); // consomme l'identifiant
        },
    }

    // Sinon, on retourne une déclaration inconnue
    Ok(Statement {
        kind: StatementKind::Unknown("Unknown Identifier statement".into()),
        value: VariableValue::Text(token.lexeme.clone()),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
