use crate::core::{ parser::variable, types::token::TokenKind };

pub fn parse_dot(
    parser: &mut crate::core::parser::Parser
) -> Result<crate::core::types::statement::Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    // Ne consomme rien ici : on vérifie seulement
    if token.kind != crate::core::types::token::TokenKind::Dot {
        return Err(format!("Expected Dot, found {:?}", token.kind));
    }

    parser.next(); // consomme le point

    // On attend un identifiant après le point
    let next_token = parser.peek().ok_or("Expected identifier after dot")?.clone();
    if next_token.kind != crate::core::types::token::TokenKind::Identifier {
        return Err(format!("Expected Identifier after Dot, found {:?}", next_token.kind));
    }

    parser.next(); // consomme l'identifiant

    // Récupère la durée
    let duration_token = parser.peek().ok_or("Expected duration after identifier")?.clone();
    // if duration_token.kind != crate::core::types::token::TokenKind::Number {
    //     return Err(format!("Expected Number after Identifier, found {:?}", duration_token.kind));
    // }

    if duration_token.kind == TokenKind::Identifier {
        // Chercher dans la table des variables
        // let variable_value = parser.import_table
        //     .get_import(&duration_token.lexeme)
        //     .ok_or(format!("Variable '{}' not found", duration_token.lexeme))?;

        // println!("Variable found: {:?}", variable_value);

        // TODO parse variable

        let var_name = &duration_token.lexeme;

        let variable_value = parser.variable_table.variables
            .get(var_name)
            .or_else(|| parser.import_table.imports.get(var_name))
            .ok_or(format!("❌ Variable '{}' not found in local or imported scope", var_name))?;

        // println!("✅ Resolved {:?}", parser.variable_table);
        // println!("✅ Resolved {:?}", parser.export_table);
        // println!("✅ Resolved {:?}", parser.import_table);
        println!("✅ Resolved {:?}", variable_value);
    }

    parser.next(); // consomme la durée

    // TODO parser la valeur de la variable
    
    Ok(crate::core::types::statement::Statement {
        kind: crate::core::types::statement::StatementKind::Trigger {
            entity: next_token.lexeme.clone(),
        },
        value: crate::core::types::variable::VariableValue::Text(next_token.lexeme.clone()),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
