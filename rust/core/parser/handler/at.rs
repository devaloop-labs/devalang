use crate::core::{
    lexer::token::TokenKind,
    parser::{ driver::Parser, statement::{ Statement, StatementKind } },
    shared::value::Value,
    store::global::GlobalStore,
};
pub fn parse_at_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume '@'

    let Some(token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    let keyword = token.lexeme.as_str();

    match keyword {
        "import" => {
            parser.advance(); // consume 'import'

            if !parser.match_token(TokenKind::LBrace) {
                return Statement::error(token, "Expected '{{' after 'import'".to_string());
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
                        let message = format!(
                            "Unexpected token in import list: {:?}",
                            token.kind.clone()
                        );
                        return Statement::error(token.clone(), message);
                    }
                }
            }

            let Some(from_token) = parser.peek_clone() else {
                return Statement::error(token, "Expected 'from' after import list".to_string());
            };

            if from_token.lexeme != "from" {
                return Statement::error(token, "Expected keyword 'from'".to_string());
            }

            parser.advance(); // consume 'from'

            let Some(source_token) = parser.peek() else {
                return Statement::error(token, "Expected string after 'from'".to_string());
            };

            if source_token.kind != TokenKind::String {
                return Statement::error(token, "Expected string after 'from'".to_string());
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
                    return Statement::error(token, "Unexpected token in export list".to_string());
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

            // Exemple : @load "preset.mydeva"
            let Some(path_token) = parser.peek() else {
                return Statement::error(token, "Expected string after 'load'".to_string());
            };

            if path_token.kind != TokenKind::String {
                return Statement::error(token, "Expected string after 'load'".to_string());
            }

            let path = path_token.lexeme.clone();

            parser.advance(); // consume string
            parser.advance(); // consume 'as'

            let Some(as_token) = parser.peek_clone() else {
                return Statement::error(
                    token,
                    "Expected 'as' after path in load statement".to_string()
                );
            };

            if as_token.kind != TokenKind::Identifier {
                return Statement::error(
                    token,
                    "Expected identifier after 'as' in load statement".to_string()
                );
            }

            let alias = as_token.lexeme.clone();

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
            Statement::error(token, message)
        }
    }
}
