use devalang_types::Value;

use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};
pub fn parse_at_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume '@'

    let Some(token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    let keyword = token.lexeme.as_str();

    match keyword {
        "use" => {
            parser.advance(); // consume 'use'
            let Some(use_token) = parser.previous_clone() else {
                return Statement::unknown();
            };

            // Expect plugin author
            let Some(author_token) = parser.peek_clone() else {
                return crate::core::parser::statement::error_from_token(
                    use_token,
                    "Expected plugin author".to_string(),
                );
            };
            if author_token.kind != TokenKind::Identifier {
                return crate::core::parser::statement::error_from_token(
                    author_token,
                    "Expected identifier for plugin author".to_string(),
                );
            }
            parser.advance(); // consume author

            // Expect '.'
            if !parser.match_token(TokenKind::Dot) {
                return crate::core::parser::statement::error_from_token(
                    author_token,
                    "Expected '.' after plugin author".to_string(),
                );
            }

            // Expect plugin name
            let Some(plugin_token) = parser.peek_clone() else {
                return crate::core::parser::statement::error_from_token(
                    author_token,
                    "Expected plugin name".to_string(),
                );
            };

            let name = match plugin_token.kind {
                TokenKind::Identifier | TokenKind::Number => {
                    parser.advance();
                    format!("{}.{}", author_token.lexeme, plugin_token.lexeme)
                }
                _ => {
                    return crate::core::parser::statement::error_from_token(
                        plugin_token,
                        "Expected identifier or number for plugin name".to_string(),
                    );
                }
            };

            // Optional alias
            let alias = if parser.match_token(TokenKind::As) {
                let Some(alias_token) = parser.peek_clone() else {
                    return crate::core::parser::statement::error_from_token(
                        use_token,
                        "Expected identifier after 'as'".to_string(),
                    );
                };
                if alias_token.kind != TokenKind::Identifier {
                    return crate::core::parser::statement::error_from_token(
                        alias_token,
                        "Expected identifier after 'as'".to_string(),
                    );
                }
                parser.advance();
                Some(alias_token.lexeme.clone())
            } else {
                None
            };

            Statement {
                kind: StatementKind::Use {
                    name: name.clone(),
                    alias,
                },
                value: Value::Null,
                indent: use_token.indent,
                line: use_token.line,
                column: use_token.column,
            }
        }

        "import" => {
            parser.advance(); // consume 'import'

            if !parser.match_token(TokenKind::LBrace) {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected '{{' after 'import'".to_string(),
                );
            }

            let mut names = Vec::new();
            while let Some(token) = parser.peek() {
                match &token.kind {
                    TokenKind::Identifier => {
                        names.push(token.lexeme.clone());
                        parser.advance();
                    }
                    TokenKind::Comma => {
                        parser.advance();
                    }
                    TokenKind::RBrace => {
                        parser.advance();
                        break;
                    }
                    _ => {
                        let message =
                            format!("Unexpected token in import list: {:?}", token.kind.clone());
                        return crate::core::parser::statement::error_from_token(
                            token.clone(),
                            message,
                        );
                    }
                }
            }

            let Some(from_token) = parser.peek_clone() else {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected 'from' after import list".to_string(),
                );
            };

            if from_token.lexeme != "from" {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected keyword 'from'".to_string(),
                );
            }

            parser.advance(); // consume 'from'

            let Some(source_token) = parser.peek() else {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected string after 'from'".to_string(),
                );
            };

            if source_token.kind != TokenKind::String {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected string after 'from'".to_string(),
                );
            }

            let source = source_token.lexeme.clone();
            parser.advance(); // consume string

            Statement {
                kind: StatementKind::Import { names, source },
                value: Value::Null,
                indent: token.indent,
                line: token.line,
                column: token.column,
            }
        }

        "export" => {
            parser.advance(); // consume 'export'

            parser.advance(); // consume '{'

            let names_tokens = parser.collect_until(|t| TokenKind::RBrace == t.kind);
            let mut names: Vec<String> = Vec::new();

            for token in names_tokens {
                if token.kind == TokenKind::Identifier {
                    names.push(token.lexeme.clone());
                } else if token.kind == TokenKind::Comma {
                    continue; // Ignore commas
                } else if token.kind == TokenKind::RBrace {
                    break; // Stop at the closing brace
                } else {
                    return crate::core::parser::statement::error_from_token(
                        token,
                        "Unexpected token in export list".to_string(),
                    );
                }
            }

            Statement {
                kind: StatementKind::Export {
                    names: names.clone(),
                    source: parser.current_module.clone(),
                },
                value: Value::Null,
                indent: token.indent,
                line: token.line,
                column: token.column,
            }
        }

        "load" => {
            parser.advance(); // consume 'load'

            // Example: @load "preset.mydeva"
            let Some(path_token) = parser.peek() else {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected string after 'load'".to_string(),
                );
            };

            if path_token.kind != TokenKind::String {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected string after 'load'".to_string(),
                );
            }

            let path = path_token.lexeme.clone();

            parser.advance(); // consume string

            if !parser.match_token(TokenKind::As) {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected 'as' after path in load statement".to_string(),
                );
            }

            let Some(alias_token) = parser.peek_clone() else {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected identifier after 'as' in load statement".to_string(),
                );
            };

            if alias_token.kind != TokenKind::Identifier {
                return crate::core::parser::statement::error_from_token(
                    token,
                    "Expected identifier after 'as' in load statement".to_string(),
                );
            }

            let alias = alias_token.lexeme.clone();

            parser.advance(); // consume identifier

            Statement {
                kind: StatementKind::Load {
                    source: path,
                    alias,
                },
                value: Value::Null,
                indent: token.indent,
                line: token.line,
                column: token.column,
            }
        }

        _ => {
            let message = format!("Unknown keyword after '@' : {}", keyword);
            crate::core::parser::statement::error_from_token(token, message)
        }
    }
}
