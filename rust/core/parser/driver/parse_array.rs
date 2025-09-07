use crate::core::lexer::token::TokenKind;
use devalang_types::Value;

pub fn parse_array_value(
    parser: &mut crate::core::parser::driver::parser::Parser,
) -> Option<Value> {
    let logger = devalang_utils::logger::Logger::new();
    use devalang_utils::logger::LogLevel;
    if !parser.match_token(TokenKind::LBracket) {
        return None;
    }

    let mut arr: Vec<Value> = Vec::new();

    while !parser.check_token(TokenKind::RBracket) && !parser.is_eof() {
        // Skip formatting tokens
        while parser.check_token(TokenKind::Newline)
            || parser.check_token(TokenKind::Whitespace)
            || parser.check_token(TokenKind::Indent)
            || parser.check_token(TokenKind::Dedent)
            || parser.check_token(TokenKind::Comma)
        {
            parser.advance();
        }

        if parser.check_token(TokenKind::RBracket) {
            break;
        }

        if let Some(token) = parser.peek_clone() {
            let value = match token.kind {
                TokenKind::String => {
                    parser.advance();
                    Value::String(token.lexeme.clone())
                }
                TokenKind::Number => {
                    // Support decimals and fraction literals (e.g., 1/4 -> Duration::Beat("1/4"))
                    let mut number_str = token.lexeme.clone();
                    parser.advance();
                    if let Some(dot) = parser.peek_clone() {
                        if dot.kind == TokenKind::Dot {
                            if let Some(next) = parser.peek_nth(1).cloned() {
                                if next.kind == TokenKind::Number {
                                    parser.advance(); // consume dot
                                    parser.advance(); // consume next number
                                    number_str.push('.');
                                    number_str.push_str(&next.lexeme);
                                }
                            }
                        }
                    }

                    // Fraction form: number '/' number or identifier
                    if let Some(slash_tok) = parser.peek_clone() {
                        if slash_tok.kind == TokenKind::Slash {
                            // consume '/'
                            parser.advance();
                            if let Some(den_tok) = parser.peek_clone() {
                                match den_tok.kind {
                                    TokenKind::Number | TokenKind::Identifier => {
                                        let frac = format!("{}/{}", number_str, den_tok.lexeme);
                                        parser.advance();
                                        Value::Duration(devalang_types::Duration::Beat(frac))
                                    }
                                    _ => Value::Number(number_str.parse::<f32>().unwrap_or(0.0)),
                                }
                            } else {
                                Value::Number(number_str.parse::<f32>().unwrap_or(0.0))
                            }
                        } else {
                            Value::Number(number_str.parse::<f32>().unwrap_or(0.0))
                        }
                    } else {
                        Value::Number(number_str.parse::<f32>().unwrap_or(0.0))
                    }
                }
                TokenKind::Identifier => {
                    parser.advance();
                    Value::Identifier(token.lexeme.clone())
                }
                TokenKind::LBrace => {
                    // Allow inline maps inside arrays
                    if let Some(v) = crate::core::parser::driver::parse_map::parse_map_value(parser)
                    {
                        v
                    } else {
                        Value::Null
                    }
                }
                TokenKind::LBracket => {
                    // Nested arrays
                    if let Some(v) = parse_array_value(parser) {
                        v
                    } else {
                        Value::Null
                    }
                }
                _ => {
                    parser.advance();
                    Value::Null
                }
            };

            // Only push non-null (retain alignment with permissive parsing)
            if value != Value::Null {
                arr.push(value);
            }

            // Optional trailing comma handled by the skipper at loop start
        } else {
            break;
        }
    }

    if !parser.match_token(TokenKind::RBracket) {
        logger.log_message(LogLevel::Error, "Expected ']' at end of array");
    }

    Some(Value::Array(arr))
}
