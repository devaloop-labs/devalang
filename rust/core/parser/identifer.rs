use crate::core::{
    parser::{ bank::parse_bank, variable::parse_let_statement, Parser },
    types::{
        statement::{ Statement, StatementKind },
        store::GlobalStore,
        token::{ Token, TokenKind },
        variable::VariableValue,
    },
};

pub fn parse_identifier(
    parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    // Ne consomme rien ici : on vérifie seulement
    if token.kind != TokenKind::Identifier {
        return Err(format!("Expected Identifier, found {:?}", token.kind));
    }

    match token.lexeme.as_str() {
        "let" => {
            parser.next(); // consomme "let"
            return parse_let_statement(parser); // consomme le reste
        }
        "bank" => {
            parser.next(); // consomme "bank"
            return parse_bank(parser, global_store);
        }
        _ => {
            parser.next(); // consomme l'identifiant
        }
    }

    let statment_value = match token.kind {
        TokenKind::Identifier => {
            // On essaie de récupérer la valeur de la variable
            let var_name = &token.lexeme;
            let current_module = global_store.modules
                .get(&parser.current_module);
            let mut variable_value = VariableValue::Unknown;

            if current_module.is_none() {
                return Err(format!("Module '{}' not found", parser.current_module));
            } else {
                variable_value = current_module.unwrap()
                    .variable_table
                    .variables
                    .get(var_name)
                    .cloned()
                    .unwrap_or(VariableValue::Unknown);
            }

            println!("Trying to get variable '{}' in module '{}'", var_name, parser.current_module);

            // On retourne la valeur de la variable
            variable_value
        }
        TokenKind::Number => VariableValue::Number(token.lexeme.parse().unwrap_or(0.0)),
        TokenKind::String => {
            // On essaie de récupérer la valeur de la variable
            let var_name = &token.lexeme;
            let current_module = global_store.modules
                .get(&parser.current_module);
            let mut variable_value = VariableValue::Unknown;

            if current_module.is_none() {
                return Err(format!("Module '{}' not found", parser.current_module));
            } else {
                variable_value = current_module.unwrap()
                    .variable_table
                    .variables
                    .get(var_name)
                    .cloned()
                    .unwrap_or(VariableValue::Unknown);
            }

            println!("Trying to get variable '{}' in module '{}'", var_name, parser.current_module);

            // On retourne la valeur de la variable
            variable_value
        }
        _ => VariableValue::Unknown,
    };

    println!("Trying get variable : {:?}", global_store.modules);

    // Sinon, on retourne une déclaration inconnue
    Ok(Statement {
        kind: StatementKind::Unknown,
        value: statment_value,
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
