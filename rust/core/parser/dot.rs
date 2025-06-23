use std::{ collections::HashMap, hash::Hash };

use crate::core::types::{
    statement::Statement,
    token::{ Token, TokenDuration, TokenKind, TokenParamValue },
    variable::{ Variable, VariableValue },
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
    let mut duration = TokenDuration::Unknown;

    match duration_token.lexeme.as_str() {
        "auto" => {
            duration = TokenDuration::Auto;
        }
        "infinite" => {
            duration = TokenDuration::Infinite;
        }
        _ => {
            if let Ok(num) = duration_token.lexeme.parse::<f32>() {
                duration = TokenDuration::Number(num);
            } else if let Ok(boolean) = duration_token.lexeme.parse::<bool>() {
                duration = TokenDuration::Unknown;
            } else if duration_token.kind == TokenKind::Identifier {
                duration = TokenDuration::Identifier(duration_token.lexeme.clone());
            } else {
                return Err(format!("Invalid duration format: {}", duration_token.lexeme));
            }
        }
    }

    parser.next(); // consomme la durée

    let params_map: VariableValue = parse_params_map(parser);

    dot_params.insert("params".to_string(), params_map);

    let mut dot_params_map: HashMap<String, TokenParamValue> = HashMap::new();

    for (key, value) in dot_params {
        let token_value = match value {
            VariableValue::Text(s) => TokenParamValue::String(s),
            VariableValue::Number(n) => TokenParamValue::Number(n),
            VariableValue::Boolean(b) => TokenParamValue::Boolean(b),
            VariableValue::Array(arr) => {
                // Convertit l'array en une liste de TokenParamValue
                let tokens: Vec<TokenParamValue> = arr
                    .into_iter()
                    .map(|t| {
                        match t.kind {
                            TokenKind::String => TokenParamValue::String(t.lexeme),
                            TokenKind::Number =>
                                TokenParamValue::Number(t.lexeme.parse().unwrap_or(0.0)),
                            TokenKind::Boolean =>
                                TokenParamValue::Boolean(t.lexeme.parse().unwrap_or(false)),
                            TokenKind::Map => {
                                // Si c'est une map, on la parse récursivement
                                let mut new_map: HashMap<String, TokenParamValue> = HashMap::new();

                                let split_result = t.lexeme
                                    .split(',')
                                    .map(|kv| {
                                        let mut parts = kv.splitn(2, ':');
                                        let k = parts.next().unwrap_or("").trim().to_string();
                                        let v = parts.next().unwrap_or("").trim().to_string();
                                        (k, TokenParamValue::String(v))
                                    })
                                    .collect::<HashMap<String, TokenParamValue>>();

                                for (k, v) in split_result {
                                    new_map.insert(k, v);
                                }

                                TokenParamValue::Map(new_map)
                            }

                            TokenKind::Identifier => TokenParamValue::String(t.lexeme),

                            _ => { TokenParamValue::Unknown }
                        }
                    })
                    .collect();
                TokenParamValue::Array(tokens)
            }
            _ => {
                continue;
            }
        };
        dot_params_map.insert(key, token_value);
    }

    let params_value = VariableValue::Map(dot_params_map);

    Ok(Statement {
        kind: crate::core::types::statement::StatementKind::Trigger {
            entity: next_token.lexeme.clone(),
            duration: duration.clone(),
        },
        value: params_value,
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}

fn parse_params_map(parser: &mut crate::core::parser::Parser) -> VariableValue {
    let mut params_map: HashMap<String, VariableValue> = HashMap::new();
    let mut next_token = parser.peek().cloned();
    let mut current_key: Option<String> = None;
    let mut current_value: Option<VariableValue> = None;
    let mut params_tokens: Vec<Token> = Vec::new();

    while let Some(token) = next_token {
        match token.kind {
            TokenKind::RBrace => {
                parser.next(); // consomme le '}'
                break;
            }
            TokenKind::Identifier => {
                if let Some(key) = current_key.take() {
                    // Si on a déjà une clé, on l'utilise pour ajouter la valeur précédente
                    if let Some(value) = current_value.take() {
                        params_map.insert(key, value);
                    }
                }
                current_key = Some(token.lexeme.clone());
                current_value = Some(VariableValue::Text(token.lexeme.clone()));
                parser.next(); // consomme l'identifiant
            }
            TokenKind::Number | TokenKind::String | TokenKind::Boolean => {
                if let Some(key) = current_key.take() {
                    current_value = Some(Variable::from_token(token.clone()).value);
                    params_map.insert(key, current_value.clone().unwrap());
                }
                parser.next(); // consomme la valeur
            }
            TokenKind::Map => {
                if let Some(key) = current_key.take() {
                    current_value = Some(parse_params_map(parser));
                    params_map.insert(key, current_value.clone().unwrap());
                }
                parser.next(); // consomme la map
            }
            TokenKind::EOF => {
                parser.next();
                break;
            }
            _ => {
                panic!("Expected identifier or value, found {:?}", token.kind);
            }
        }

        params_tokens.push(token.clone());

        next_token = parser.peek().cloned();
    }
    if let Some(key) = &current_key {
        if let Some(value) = &current_value {
            params_map.insert(key.clone(), value.clone());
        }
    }

    VariableValue::Array(params_tokens)
}
