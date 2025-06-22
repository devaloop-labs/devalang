use std::collections::HashMap;

use crate::core::{
    parser::variable,
    types::{ statement::Statement, token::TokenKind, variable::VariableValue },
};

pub fn parse_dot(
    parser: &mut crate::core::parser::Parser,
    global_store: &mut crate::core::types::store::GlobalStore
) -> Result<crate::core::types::statement::Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();
    let mut trigger_value: String = String::from("Unknown Trigger");

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
    if duration_token.kind == TokenKind::Identifier {
        // let var_name = &duration_token.lexeme;

        // println!("🔍 Resolving variable '{}'", var_name);
        // println!("🔍 Current module: '{}'", parser.current_module);
        // println!("🔍 Available modules : '{:?}'", global_store.modules);
        // println!("🔍 Current module variables: '{:?}'", global_store.modules.get(&parser.current_module));

        // let variable_value = global_store.modules
        //     .get(&parser.current_module)
        //     .and_then(|module| module.export_table.exports.get(var_name))
        //     .ok_or(format!("❌ Variable '{}' not found in local or imported scope", var_name))?;

        // trigger_value = match variable_value {
        //     VariableValue::Text(text) => text.clone(),
        //     VariableValue::Number(num) => num.to_string(),
        //     VariableValue::Unknown => {
        //         return Err(format!("❌ Variable '{}' is unknown", var_name));
        //     }
        //     _ => {
        //         return Err(format!("❌ Unsupported variable type for '{}'", var_name));
        //     }
        // };

        // // println!("✅ Resolved {:?}", parser.variable_table);
        // // println!("✅ Resolved {:?}", parser.export_table);
        // // println!("✅ Resolved {:?}", parser.import_table);
        // println!("✅ Resolved {:?}", variable_value);

        // NOTE : Attendre la prochaine itération pour parser la variable
        return Err(format!("Expected duration after identifier, found variable '{:?}'", duration_token));
    }

    parser.next(); // consomme la durée

    // TODO parser la valeur de la variable

    Ok(Statement {
        kind: crate::core::types::statement::StatementKind::Trigger {
            entity: next_token.lexeme.clone(),
        },
        value: VariableValue::Text(trigger_value),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
