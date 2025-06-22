use std::collections::HashMap;

use crate::core::{
    parser::{ parse_variable_value, variable },
    types::{ statement::Statement, token::{ TokenDuration, TokenKind }, variable::VariableValue },
};

pub fn parse_dot(
    parser: &mut crate::core::parser::Parser,
    global_store: &mut crate::core::types::store::GlobalStore
) -> Result<crate::core::types::statement::Statement, String> {
    let mut dot_params: HashMap<String, VariableValue> = HashMap::new();

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
    let mut duration = VariableValue::Unknown;
    if duration_token.kind == TokenKind::Identifier {
        duration = VariableValue::Text(duration_token.lexeme.clone());
    } else if duration_token.kind == TokenKind::Number {
        duration = VariableValue::Number(duration_token.lexeme.parse::<f32>().unwrap_or(0.0));
    }

    parser.next(); // consomme la durée

    // Parsing des paramètres

    if let Some(lbrace_token) = parser.peek() {
        parser.next();
    }

    let mut params: HashMap<String, VariableValue> = HashMap::new();

    let param_tokens = parser.collect_until(|t| { t.kind == TokenKind::RBrace });

    let mut i = 0;

    while i < param_tokens.len() {
        let key_token = &param_tokens[i];

        if key_token.kind != TokenKind::Identifier {
            return Err(format!("Expected identifier as parameter key, found {:?}", key_token.kind));
        }

        let key = key_token.lexeme.clone();
        i += 1;

        if i >= param_tokens.len() || param_tokens[i].kind != TokenKind::Colon {
            return Err(format!("Expected ':' after key '{}'", key));
        }
        i += 1;

        if i >= param_tokens.len() {
            return Err(format!("Expected value after ':' for key '{}'", key));
        }

        let value_token = &param_tokens[i];
        let value = match value_token.kind {
            TokenKind::String => VariableValue::Text(value_token.lexeme.clone()),
            TokenKind::Number =>
                VariableValue::Number(value_token.lexeme.parse::<f32>().unwrap_or(0.0)),
            TokenKind::Boolean =>
                VariableValue::Boolean(value_token.lexeme.parse::<bool>().unwrap_or(false)),
            _ => VariableValue::Unknown,
        };

        params.insert(key, value);
        i += 1;

        // Skip optional comma
        if i < param_tokens.len() && param_tokens[i].kind == TokenKind::Comma {
            i += 1;
        }
    }

    dot_params.insert("duration".to_string(), duration);
    dot_params.insert("params".to_string(), VariableValue::Map(params));

    Ok(Statement {
        kind: crate::core::types::statement::StatementKind::Trigger {
            entity: next_token.lexeme.clone(),
        },
        value: VariableValue::Map(dot_params),
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
