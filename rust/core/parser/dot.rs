use std::{ collections::HashMap, hash::Hash };

use crate::core::types::{
    statement::{ Statement, StatementKind },
    token::{ Token, TokenDuration, TokenKind, TokenParamValue },
    variable::{ Variable, VariableValue },
};

pub fn parse_dot(
    parser: &mut crate::core::parser::Parser,
    global_store: &mut crate::core::types::store::GlobalStore
) -> Result<crate::core::types::statement::Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();
    let mut trigger_value: String = String::from("Unknown Trigger");

    if token.kind != crate::core::types::token::TokenKind::Dot {
        return Err(format!("Expected Dot, found {:?}", token.kind));
    }

    parser.next();

    let next_token = parser.peek().ok_or("Expected identifier after dot")?.clone();
    if next_token.kind != crate::core::types::token::TokenKind::Identifier {
        return Err(format!("Expected Identifier after Dot, found {:?}", next_token.kind));
    }

    parser.next(); 

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

    parser.next();
   
    let dot_params: Vec<Token> = parser.collect_until(|t| {
        t.kind == TokenKind::Newline
    });

    let mut params_value: VariableValue = VariableValue::Null;

    for param in dot_params {
        let token_value = match param.kind {
            TokenKind::String => VariableValue::Text(param.lexeme),
            TokenKind::Number => VariableValue::Number(param.lexeme.parse().unwrap_or(0.0)),
            TokenKind::Boolean => VariableValue::Boolean(param.lexeme.parse().unwrap_or(false)),
            TokenKind::Map => {
                let mut map: HashMap<String, TokenParamValue> = HashMap::new();
                let entries: Vec<&str> = param.lexeme.split(',').collect();
                for entry in entries {
                    let parts: Vec<&str> = entry.split(':').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().to_string();
                        let value = match parts[1].trim() {
                            "true" => TokenParamValue::Boolean(true),
                            "false" => TokenParamValue::Boolean(false),
                            _ => TokenParamValue::String(parts[1].trim().to_string()),
                        };
                        map.insert(key, value);
                    }
                }
                VariableValue::Map(map)
            }
            TokenKind::Identifier => {
                VariableValue::Text(param.lexeme.clone())
            }
            TokenKind::Unknown => {
                Err(format!("Unsupported token type in dot parameters: {:?}", param.kind))?;
                VariableValue::Null
            },
            _ => {
                Err(format!("Unsupported token type in dot parameters: {:?}", param.kind))?;
                VariableValue::Unknown
            },
        };
        params_value = token_value;
    }

    Ok(Statement {
        kind: StatementKind::Trigger {
            entity: next_token.lexeme.clone(),
            duration: duration.clone(),
        },
        value: params_value,
        indent: token.indent,
        line: token.line,
        column: token.column,
    })
}
