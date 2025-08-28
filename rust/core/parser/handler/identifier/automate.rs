use std::collections::HashMap;

use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    shared::value::Value,
    store::global::GlobalStore,
};

// Grammar:
// automate <identifier>:
//     param <name> { <percent>% = <number> ... }
// Produces StatementKind::Automate with value map:
// { target: Identifier, params: Map<paramName, Map<percent, Number>> }
pub fn parse_automate_token(
    parser: &mut Parser,
    current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume 'automate'

    // Expect target identifier
    let Some(target_token) = parser.peek_clone() else {
        return Statement::error(
            current_token,
            "Expected target after 'automate'".to_string(),
        );
    };

    if target_token.kind != TokenKind::Identifier && target_token.kind != TokenKind::String {
        return Statement::error(
            target_token,
            "Expected valid target after 'automate'".to_string(),
        );
    }
    parser.advance(); // consume target

    // Expect ':'
    let Some(colon_token) = parser.peek_clone() else {
        return Statement::error(
            target_token,
            "Expected ':' after automate target".to_string(),
        );
    };
    if colon_token.kind != TokenKind::Colon {
        return Statement::error(
            colon_token,
            "Expected ':' after automate target".to_string(),
        );
    }
    parser.advance(); // consume ':'

    let base_indent = current_token.indent;

    // Collect tokens inside block (indented > base_indent)
    let mut index = parser.token_index;
    let mut tokens_inside = Vec::new();
    while index < parser.tokens.len() {
        let tok = parser.tokens[index].clone();
        if tok.indent <= base_indent && tok.kind != TokenKind::Newline {
            break;
        }
        tokens_inside.push(tok);
        index += 1;
    }
    parser.token_index = index;

    // Now parse block manually to capture 'param' entries without reusing general parser kinds
    let mut local = Parser {
        resolve_modules: parser.resolve_modules,
        tokens: tokens_inside,
        token_index: 0,
        current_module: parser.current_module.clone(),
        previous: None,
    };

    let mut params: HashMap<String, Value> = HashMap::new();

    while let Some(tok) = local.peek_clone() {
        match tok.kind {
            TokenKind::Identifier if tok.lexeme == "param" => {
                local.advance(); // consume 'param'
                // param name
                let Some(name_tok) = local.peek_clone() else {
                    return Statement::error(
                        tok,
                        "Expected parameter name after 'param'".to_string(),
                    );
                };
                if name_tok.kind != TokenKind::Identifier && name_tok.kind != TokenKind::String {
                    return Statement::error(name_tok, "Expected valid parameter name".to_string());
                }
                local.advance(); // consume name

                // Expect '{'
                if !local.match_token(TokenKind::LBrace) {
                    return Statement::error(
                        name_tok,
                        "Expected '{' to start parameter block".to_string(),
                    );
                }

                // Collect entries like: 0% = 0.0
                let mut envelope: HashMap<String, Value> = HashMap::new();
                while let Some(inner) = local.peek_clone() {
                    if inner.kind == TokenKind::RBrace {
                        local.advance();
                        break;
                    }
                    // Skip formatting tokens inside the param block
                    if matches!(
                        inner.kind,
                        TokenKind::Newline
                            | TokenKind::Indent
                            | TokenKind::Dedent
                            | TokenKind::Comma
                    ) {
                        local.advance();
                        continue;
                    }

                    // Read percentage token: could be number followed by '%' as Dot or Identifier? '%' not defined.
                    // Our lexer has no Percent token, so accept either Number or Identifier containing e.g. '0%'.
                    let percent_token = inner.clone();
                    local.advance();

                    let percent_key = percent_token.lexeme.clone();

                    // Expect '='
                    // Skip any stray formatting between key and '='
                    while let Some(t) = local.peek_kind() {
                        if matches!(
                            t,
                            TokenKind::Indent | TokenKind::Dedent | TokenKind::Newline
                        ) {
                            local.advance();
                            continue;
                        }
                        break;
                    }
                    if !local.match_token(TokenKind::Equals) {
                        return Statement::error(
                            percent_token,
                            "Expected '=' in param entry".to_string(),
                        );
                    }

                    // Read value (number or identifier)
                    // Skip formatting before value
                    while let Some(t) = local.peek_kind() {
                        if matches!(
                            t,
                            TokenKind::Indent | TokenKind::Dedent | TokenKind::Newline
                        ) {
                            local.advance();
                            continue;
                        }
                        break;
                    }

                    let value = if let Some(vtok) = local.peek_clone() {
                        match vtok.kind {
                            // Handle negative numbers where '-' is lexed as Arrow
                            TokenKind::Arrow => {
                                // Check if next token is a number
                                let mut num_str = String::from("-");
                                local.advance(); // consume '-'
                                if let Some(ntok) = local.peek_clone() {
                                    if ntok.kind == TokenKind::Number {
                                        num_str.push_str(&ntok.lexeme);
                                        local.advance(); // consume number
                                        if let Some(dot) = local.peek_clone() {
                                            if dot.kind == TokenKind::Dot {
                                                local.advance();
                                                if let Some(frac) = local.peek_clone() {
                                                    if frac.kind == TokenKind::Number {
                                                        num_str.push('.');
                                                        num_str.push_str(&frac.lexeme);
                                                        local.advance();
                                                    }
                                                }
                                            }
                                        }
                                        Value::Number(num_str.parse::<f32>().unwrap_or(0.0))
                                    } else {
                                        Value::Unknown
                                    }
                                } else {
                                    Value::Unknown
                                }
                            }
                            TokenKind::Number => {
                                // Possibly a float with dot
                                let mut number_str = vtok.lexeme.clone();
                                local.advance();
                                if let Some(dot) = local.peek_clone() {
                                    if dot.kind == TokenKind::Dot {
                                        local.advance();
                                        if let Some(frac) = local.peek_clone() {
                                            if frac.kind == TokenKind::Number {
                                                number_str.push('.');
                                                number_str.push_str(&frac.lexeme);
                                                local.advance();
                                            }
                                        }
                                    }
                                }
                                Value::Number(number_str.parse::<f32>().unwrap_or(0.0))
                            }
                            TokenKind::Identifier => {
                                local.advance();
                                Value::Identifier(vtok.lexeme.clone())
                            }
                            TokenKind::String => {
                                local.advance();
                                Value::String(vtok.lexeme.clone())
                            }
                            _ => {
                                local.advance();
                                Value::Unknown
                            }
                        }
                    } else {
                        Value::Null
                    };

                    envelope.insert(percent_key, value);
                }

                params.insert(name_tok.lexeme.clone(), Value::Map(envelope));
            }
            _ => {
                local.advance();
            }
        }
    }

    let mut value_map = HashMap::new();
    value_map.insert(
        "target".to_string(),
        Value::String(target_token.lexeme.clone()),
    );
    value_map.insert("params".to_string(), Value::Map(params));

    Statement {
        kind: StatementKind::Automate {
            target: target_token.lexeme.clone(),
        },
        value: Value::Map(value_map),
        indent: current_token.indent,
        line: current_token.line,
        column: current_token.column,
    }
}
