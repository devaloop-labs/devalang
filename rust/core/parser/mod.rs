pub mod identifer;
pub mod variable;
pub mod at;
pub mod dot;
pub mod bank;
pub mod loop_;
pub mod tempo;

use crate::{
    core::{
        parser::{
            at::parse_at,
            bank::parse_bank,
            dot::parse_dot,
            identifer::parse_identifier,
            loop_::parse_loop,
            tempo::parse_tempo,
        },
        preprocessor::resolver::resolve_statement,
        types::{
            module::Module,
            parser::Parser,
            statement::{
                Statement,
                StatementKind,
                StatementResolved,
                StatementResolvedValue,
                StatementValue,
            },
            store::{ GlobalStore, VariableTable },
            token::{ Token, TokenKind },
            variable::VariableValue,
        },
    },
    utils::logger::log_message,
};

pub fn parse_without_resolving(
    tokens: Vec<Token>,
    mut parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Vec<Statement> {
    let mut statements = Vec::new();

    while !parser.is_eof() {
        let mut error_statement = Statement {
            kind: StatementKind::Error,
            value: VariableValue::Null,
            line: parser.peek().map_or(0, |t| t.line),
            column: parser.peek().map_or(0, |t| t.column),
            indent: parser.peek().map_or(0, |t| t.indent),
        };

        match parser.peek().map(|t| t.kind.clone()) {
            Some(TokenKind::Identifier) => {
                match parse_identifier(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            Some(TokenKind::Bank) => {
                match parse_bank(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            Some(TokenKind::At) => {
                match parse_at(&mut parser) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            Some(TokenKind::Dot) => {
                match parse_dot(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            Some(TokenKind::Loop) => {
                match parse_loop(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            Some(TokenKind::Tempo) => {
                match parse_tempo(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        error_statement.value = VariableValue::Text(e.to_string());
                        statements.push(error_statement);
                    }
                }
            }

            | Some(TokenKind::LBrace)
            | Some(TokenKind::RBrace)
            | Some(TokenKind::LBracket)
            | Some(TokenKind::RBracket)
            | Some(TokenKind::DbQuote)
            | Some(TokenKind::Quote)
            | Some(TokenKind::Number)
            | Some(TokenKind::String)
            | Some(TokenKind::Newline)
            | Some(TokenKind::Indent)
            | Some(TokenKind::Dedent) => {
                parser.next(); // juste consommer pour le moment
            }
            Some(_) => {
                parser.next(); // fallback : avance
            }
            None => {
                break;
            }
        }
    }

    let mut errors = Vec::new();

    statements.iter().for_each(|statement| {
        match &statement.kind {
            StatementKind::Error => {
                let error_message = format!(
                    "Error at line {}, column {}: {:?}",
                    statement.line,
                    statement.column,
                    statement.value
                );

                errors.push(statement.clone());

                log_message(&error_message, "ERROR");
            }
            _ => {
                // Pour les autres types de déclarations, on peut éventuellement faire d'autres traitements
            }
        }
    });

    if errors.len() > 0 {
        log_message(
            &format!(
                "{} error(s) found while parsing, parsing stopped.",
                errors.len()
            ),
            "INFO"
        );

        vec![]
    } else {
        statements
    }
}

pub fn parse_without_resolving_with_module(tokens: Vec<Token>, module: &Module) -> Vec<Statement> {
    let mut parser = Parser::new(tokens.clone());

    // Mettre à jour le contexte du module courant
    parser.current_module = module.path.clone();

    let mut global_store = GlobalStore::new();
    global_store.insert_module(module.path.clone(), module.clone());

    let statements = parse_without_resolving(tokens, &mut parser, &mut global_store);
    // Mettre à jour le module avec les déclarations
    let mut updated_module = module.clone();
    updated_module.statements = statements.clone();

    return statements;
}

pub fn parse_with_resolving_with_module(
    tokens: Vec<Token>,
    module: &Module
) -> Vec<StatementResolved> {
    let mut parser = Parser::new(tokens.clone());

    // Mettre à jour le contexte du module courant
    parser.current_module = module.path.clone();

    let mut global_store = GlobalStore::new();
    global_store.insert_module(module.path.clone(), module.clone());

    let statements = parse_without_resolving(tokens, &mut parser, &mut global_store);

    // Résoudre les déclarations

    let mut resolved_statements = Vec::new();

    for statement in statements {
        let resolved_statement = resolve_statement(&statement, module);
        resolved_statements.push(resolved_statement);
    }

    return resolved_statements;
}
