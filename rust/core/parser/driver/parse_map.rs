use crate::core::lexer::token::TokenKind;
use devalang_types::Value;

pub fn parse_map_value(parser: &mut crate::core::parser::driver::parser::Parser) -> Option<Value> {
    let logger = devalang_utils::logger::Logger::new();
    use devalang_utils::logger::LogLevel;
    if !parser.match_token(TokenKind::LBrace) {
        return None;
    }

    let mut map = std::collections::HashMap::new();

    while !parser.check_token(TokenKind::RBrace) && !parser.is_eof() {
        // Skip separators and formatting before the key
        while parser.check_token(TokenKind::Newline)
            || parser.check_token(TokenKind::Whitespace)
            || parser.check_token(TokenKind::Indent)
            || parser.check_token(TokenKind::Dedent)
            || parser.check_token(TokenKind::Comma)
        {
            parser.advance();
        }

        // Check if we are at the closing brace of the map
        if parser.check_token(TokenKind::RBrace) {
            break;
        }

        let key = if let Some(token) = parser.advance() {
            match token.kind {
                TokenKind::Whitespace
                | TokenKind::Indent
                | TokenKind::Dedent
                | TokenKind::Newline => {
                    continue;
                }
                _ => token.lexeme.clone(),
            }
        } else {
            break;
        };

        // Skip newlines and whitespace before colon
        while parser.check_token(TokenKind::Newline) || parser.check_token(TokenKind::Whitespace) {
            parser.advance();
        }

        if !parser.match_token(TokenKind::Colon) {
            logger.log_message(
                LogLevel::Error,
                &format!("Expected ':' after map key '{}'", key),
            );
            break;
        }

        // Skip separators and formatting before value
        while parser.check_token(TokenKind::Newline)
            || parser.check_token(TokenKind::Whitespace)
            || parser.check_token(TokenKind::Indent)
            || parser.check_token(TokenKind::Dedent)
            || parser.check_token(TokenKind::Comma)
        {
            parser.advance();
        }

        let value = if let Some(token) = parser.peek_clone() {
            match token.kind {
                TokenKind::String => {
                    parser.advance();
                    Value::String(token.lexeme.clone())
                }
                TokenKind::Number => {
                    // Handle number, decimal number and optional fraction form (e.g., 1/4)
                    let mut number_str = token.lexeme.clone();
                    parser.advance(); // consume the first number

                    // decimal support: number '.' number
                    if let Some(dot_token) = parser.peek_clone() {
                        if dot_token.kind == TokenKind::Dot {
                            parser.advance(); // consume the dot

                            if let Some(decimal_token) = parser.peek_clone() {
                                if decimal_token.kind == TokenKind::Number {
                                    parser.advance(); // consume the number after the dot
                                    number_str.push('.');
                                    number_str.push_str(&decimal_token.lexeme);
                                } else {
                                    logger.log_message(
                                        LogLevel::Error,
                                        &format!(
                                            "Expected number after dot, got {:?}",
                                            decimal_token
                                        ),
                                    );
                                    return Some(Value::Null);
                                }
                            } else {
                                logger.log_message(
                                    LogLevel::Error,
                                    "Expected number after dot, but reached EOF",
                                );
                                return Some(Value::Null);
                            }
                        }
                    }

                    // Fraction support: number '/' number  -> Duration::Beat("num/den")
                    if let Some(slash_tok) = parser.peek_clone() {
                        if slash_tok.kind == TokenKind::Slash {
                            // consume '/'
                            parser.advance();
                            if let Some(den_tok) = parser.peek_clone() {
                                match den_tok.kind {
                                    TokenKind::Number | TokenKind::Identifier => {
                                        let frac = format!("{}/{}", number_str, den_tok.lexeme);
                                        parser.advance();
                                        return Some(Value::Duration(
                                            devalang_types::Duration::Beat(frac),
                                        ));
                                    }
                                    _ => {
                                        logger.log_message(
                                            LogLevel::Error,
                                            &format!(
                                                "Expected number or identifier after '/', got {:?}",
                                                den_tok
                                            ),
                                        );
                                        return Some(Value::Null);
                                    }
                                }
                            } else {
                                logger.log_message(
                                    LogLevel::Error,
                                    "Expected denominator after '/', but reached EOF",
                                );
                                return Some(Value::Null);
                            }
                        }
                    }

                    Value::Number(number_str.parse::<f32>().unwrap_or(0.0))
                }

                TokenKind::Identifier => {
                    // Support dotted identifiers in map values: alias.param or nested
                    let current_line = token.line;
                    let mut parts: Vec<String> = vec![token.lexeme.clone()];
                    parser.advance();
                    loop {
                        let Some(next) = parser.peek_clone() else {
                            break;
                        };
                        if next.line != current_line {
                            break;
                        }
                        if next.kind == TokenKind::Dot {
                            // Consume '.' and the following identifier/number on same line
                            parser.advance(); // dot
                            if let Some(id2) = parser.peek_clone() {
                                if id2.line == current_line
                                    && (id2.kind == TokenKind::Identifier
                                        || id2.kind == TokenKind::Number)
                                {
                                    parts.push(id2.lexeme.clone());
                                    parser.advance(); // consume part
                                    continue;
                                }
                            }
                            break;
                        } else {
                            break;
                        }
                    }
                    Value::Identifier(parts.join("."))
                }
                TokenKind::LBracket => {
                    // Allow arrays as map values
                    if let Some(v) =
                        crate::core::parser::driver::parse_array::parse_array_value(parser)
                    {
                        v
                    } else {
                        Value::Null
                    }
                }
                TokenKind::LBrace => {
                    // Allow inline nested maps as map values
                    if let Some(v) = parse_map_value(parser) {
                        v
                    } else {
                        Value::Null
                    }
                }
                _ => {
                    logger.log_message(
                        LogLevel::Error,
                        &format!("Unexpected token in map value: {:?}", token),
                    );
                    Value::Null
                }
            }
        } else {
            Value::Null
        };

        map.insert(key, value);

        // Optionally skip a trailing comma after the value
        while parser.check_token(TokenKind::Comma)
            || parser.check_token(TokenKind::Whitespace)
            || parser.check_token(TokenKind::Newline)
        {
            parser.advance();
        }
    }

    if !parser.match_token(TokenKind::RBrace) {
        logger.log_message(LogLevel::Error, "Expected '}' at end of map");
    }

    Some(Value::Map(map))
}
