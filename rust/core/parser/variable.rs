use std::collections::HashMap;

use crate::core::{
    parser::Parser,
    types::{
        statement::{ Statement, StatementKind },
        token::{ Token, TokenKind, TokenParamValue },
        variable::VariableValue,
    },
};

pub fn parse_let_statement(parser: &mut Parser) -> Result<Statement, String> {
    let name_token = parser.next().ok_or("Expected variable name after 'let'")?.clone();
    if name_token.kind != TokenKind::Identifier {
        return Err(format!("Expected variable name, found {:?}", name_token.kind));
    }

    let variable_name = name_token.lexeme.clone();

    let equal_token = parser.next().ok_or("Expected '=' after variable name")?.clone();
    if equal_token.kind != TokenKind::Equals {
        return Err(format!("Expected '=', found {:?} after variable name", equal_token.kind));
    }

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
        TokenKind::LBrace => {
            let mut object: HashMap<String, VariableValue> = HashMap::new();
            while let Some(next_token) = parser.next() {
                if next_token.kind == TokenKind::RBrace {
                    break;
                }
                if next_token.kind != TokenKind::Identifier {
                    return Err(
                        format!("Expected identifier in object, found {:?}", next_token.kind)
                    );
                }
                let key = next_token.lexeme.clone();

                let colon_token = parser.next().ok_or("Expected ':' after object key")?.clone();
                if colon_token.kind != TokenKind::Colon {
                    return Err(format!("Expected ':', found {:?}", colon_token.kind));
                }

                let value_token = parser.next().ok_or("Expected value after ':'")?.clone();
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
                    _ => {
                        return Err(format!("Invalid object value token: {:?}", value_token.kind));
                    }
                };
                object.insert(key, value);
            }

            let mut variable_object_map: HashMap<String, TokenParamValue> = HashMap::new();

            for (key, value) in object {
                let token_value = match value {
                    VariableValue::Text(s) => TokenParamValue::String(s),
                    VariableValue::Number(n) => TokenParamValue::Number(n),
                    VariableValue::Boolean(b) => TokenParamValue::Boolean(b),
                    _ => {
                        continue;
                    }
                };
                variable_object_map.insert(key, token_value);
            }

            VariableValue::Map(variable_object_map)
        }
        _ => {
            return Err(format!("Invalid value token: {:?}", value_token.kind));
        }
    };

    parser.variable_table.variables.insert(variable_name.clone(), value.clone());

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

pub fn parse_variable_value(parser: &mut Parser, token: &Token) -> Result<VariableValue, String> {
    match token.kind {
        TokenKind::String => Ok(VariableValue::Text(token.lexeme.clone())),
        TokenKind::Number => {
            token.lexeme
                .parse::<f32>()
                .map(VariableValue::Number)
                .map_err(|_| "Invalid number value".to_string())
        }
        TokenKind::Boolean => {
            token.lexeme
                .parse::<bool>()
                .map(VariableValue::Boolean)
                .map_err(|_| "Invalid boolean value".to_string())
        }
        TokenKind::Identifier => Ok(VariableValue::Text(token.lexeme.clone())),
        _ => Err(format!("Invalid variable value token: {:?}", token.kind)),
    }
}
