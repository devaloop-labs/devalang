pub mod identifer;
pub mod variable;
pub mod at;
pub mod dot;
pub mod bank;
pub mod loop_;
pub mod tempo;

use crate::core::{
    parser::{
        at::parse_at,
        bank::parse_bank,
        dot::parse_dot,
        identifer::parse_identifier,
        loop_::parse_loop,
        tempo::parse_tempo,
    }, preprocessor::resolver::resolve_statement, types::{
        module::Module,
        parser::Parser,
        statement::{Statement, StatementResolved, StatementResolvedValue},
        store::{ GlobalStore, VariableTable },
        token::{ Token, TokenKind },
        variable::VariableValue,
    }
};

pub fn parse_without_resolving(
    tokens: Vec<Token>,
    mut parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Vec<Statement> {
    let mut statements = Vec::new();

    while !parser.is_eof() {
        match parser.peek().map(|t| t.kind.clone()) {
            Some(TokenKind::Identifier) => {
                match parse_identifier(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing identifier: {}", e),
                }
            }

            Some(TokenKind::Bank) => {
                match parse_bank(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing bank statement: {}", e),
                }
            }

            Some(TokenKind::At) => {
                match parse_at(&mut parser) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing @ statement: {}", e),
                }
            }

            Some(TokenKind::Dot) => {
                match parse_dot(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing dot statement: {}", e),
                }
            }

            Some(TokenKind::Loop) => {
                match parse_loop(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing loop statement: {}", e),
                }
            }

            Some(TokenKind::Tempo) => {
                match parse_tempo(&mut parser, global_store) {
                    Ok(statement) => statements.push(statement),
                    Err(e) => eprintln!("Error parsing tempo statement: {}", e),
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

    statements
}

pub fn parse_without_resolving_with_module(
    tokens: Vec<Token>,
    module: &Module
) -> Vec<Statement> {
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
    module: &Module,
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