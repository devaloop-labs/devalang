use crate::core::types::{
    parser::Parser,
    statement::{ Statement, StatementKind },
    token::TokenKind,
    variable::VariableValue,
};

pub fn parse_at(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    if token.kind != TokenKind::At {
        return Err(format!("Expected '@', found {:?}", token.kind));
    }

    parser.next();

    let identifier_token = parser.peek().ok_or("Expected identifier after '@'")?.clone();
    if identifier_token.kind != TokenKind::Identifier {
        return Err(format!("Expected Identifier, found {:?}", identifier_token.kind));
    }

    match identifier_token.lexeme.as_str() {
        "export" => {
            parser.next();

            if let Some(next_token) = parser.peek() {
                if next_token.kind == TokenKind::LBrace {
                    parser.next(); // Consomme le LBrace
                }
            }

            let exportable_tokens = parser.collect_until(|t| { t.kind == TokenKind::RBrace });

            exportable_tokens.iter().for_each(|t| {
                let variable_value = parser.variable_table
                    .get(&t.lexeme)
                    .cloned()
                    .unwrap_or_else(|| VariableValue::Text(t.lexeme.clone()));

                parser.export_table.exports.insert(t.lexeme.clone(), variable_value);
            });

            return Ok(Statement {
                kind: StatementKind::Export,
                value: VariableValue::Array(exportable_tokens),
                indent: token.indent,
                line: token.line,
                column: token.column,
            });
        }
        "import" => {
            parser.next();

            if let Some(next_token) = parser.peek() {
                if next_token.kind == TokenKind::LBrace {
                    parser.next();
                }
            }

            let importable_tokens = parser.collect_until(|t| { t.kind == TokenKind::RBrace });

            parser.next();

            if let Some(from_token) = parser.peek() {
                if from_token.kind == TokenKind::Identifier && from_token.lexeme == "from" {
                    parser.next();
                } else {
                    return Err(format!("Expected 'from', found {:?}", from_token.kind));
                }
            } else {
                return Err("Expected 'from' after import declaration".into());
            }

            let source_token = parser.peek().ok_or("Expected source after 'from'")?.clone();
            if source_token.kind != TokenKind::String {
                return Err(format!("Expected String, found {:?}", source_token.kind));
            }

            let statement = Statement {
                kind: StatementKind::Import {
                    names: importable_tokens
                        .iter()
                        .map(|t| t.lexeme.clone())
                        .collect(),
                    source: source_token.lexeme.clone(),
                },
                value: VariableValue::Array(importable_tokens),
                indent: token.indent,
                line: token.line,
                column: token.column,
            };

            return Ok(statement);
        }
        "load" => {
            parser.next();

            let source_token = parser.peek().ok_or("Expected source after load")?.clone();
            if source_token.kind != TokenKind::String {
                return Err(format!("Expected String, found {:?}", source_token.kind));
            }

            parser.next();

            let as_token = parser.peek().ok_or("Expected 'as' after load")?.clone();
            if as_token.kind != TokenKind::Identifier || as_token.lexeme != "as" {
                return Err(format!("Expected 'as', found {:?}", as_token.kind));
            }

            parser.next();

            let alias_token = parser.peek().ok_or("Expected alias after load")?.clone();
            if alias_token.kind != TokenKind::Identifier {
                return Err(format!("Expected Identifier, found {:?}", alias_token.kind));
            }

            parser.next();

            let statement = Statement {
                kind: StatementKind::Load {
                    source: source_token.lexeme.clone(),
                    alias: alias_token.lexeme.clone(),
                },
                value: VariableValue::Text(alias_token.lexeme.clone()),
                indent: token.indent,
                line: token.line,
                column: token.column,
            };

            return Ok(statement);
        }

        _ => {
            return Err(
                format!(
                    "Expected 'export', 'import' or 'load', found '{}'",
                    identifier_token.lexeme
                )
            );
        }
    }
}
