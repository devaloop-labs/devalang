pub mod identifer;
pub mod variable;
pub mod at;
pub mod dot;

use crate::core::{
    parser::{ at::parse_at, dot::parse_dot, identifer::parse_identifier },
    types::{
        parser::Parser,
        statement::Statement,
        store::{ GlobalStore, VariableTable },
        token::{ Token, TokenKind },
    },
};

pub fn parse_with_resolving(
    tokens: Vec<Token>,
    mut parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Vec<Statement> {
    // Réinitialisation du parser
    // parser.set_tokens(tokens.clone());

    // Résolution des exports
    let export_table = parser.export_table.clone();
    let import_table = parser.import_table.clone();

    for (name, value) in export_table.exports.iter() {
        println!("🔄 Resolving export: {} -> {:?}", name, value);
        // On ajoute chaque export à la table des variables du parser
        parser.variable_table.variables.insert(name.clone(), value.clone());
        parser.export_table.exports.insert(name.clone(), value.clone());
    }

    for (name, value) in import_table.imports.iter() {
        println!("🔄 Resolving import: {} -> {:?}", name, value);
        // On ajoute chaque import à la table des variables du parser
        parser.import_table.imports.insert(name.clone(), value.clone());
        parser.variable_table.variables.insert(name.clone(), value.clone());
    }

    // NOTE Debugging VariableTable
    println!("Local variable table : {:?}", parser.variable_table);

    // NOTE Debugging ExportTable
    println!("Local export table : {:?}", parser.export_table);

    // NOTE Debugging ExportTable
    println!("Local import table : {:?}", parser.import_table);

    // Parser une seconde fois, cette fois avec le imports/exports résolus dans le parser + global_store
    let statements = parse_without_resolving(tokens, &mut parser, global_store);

    statements
}

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

            Some(TokenKind::At) => {
                match parse_at(&mut parser, global_store) {
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

    println!("Parsed statements: {:?}", statements);

    statements
}

// pub fn parse(
//     tokens: Vec<Token>,
//     mut parser: &mut Parser,
//     global_store: &mut GlobalStore
// ) -> Vec<Statement> {
//     let mut statements = Vec::new();

//     while !parser.is_eof() {
//         match parser.peek().map(|t| t.kind.clone()) {
//             Some(TokenKind::Identifier) => {
//                 match parse_identifier(&mut parser, global_store) {
//                     Ok(statement) => statements.push(statement),
//                     Err(e) => eprintln!("Error parsing identifier: {}", e),
//                 }
//             }

//             Some(TokenKind::At) => {
//                 match parse_at(&mut parser, global_store) {
//                     Ok(statement) => statements.push(statement),
//                     Err(e) => eprintln!("Error parsing @ statement: {}", e),
//                 }
//             }

//             Some(TokenKind::Dot) => {
//                 match parse_dot(&mut parser, global_store) {
//                     Ok(statement) => statements.push(statement),
//                     Err(e) => eprintln!("Error parsing dot statement: {}", e),
//                 }
//             }

//             | Some(TokenKind::LBrace)
//             | Some(TokenKind::RBrace)
//             | Some(TokenKind::LBracket)
//             | Some(TokenKind::RBracket)
//             | Some(TokenKind::DbQuote)
//             | Some(TokenKind::Quote)
//             | Some(TokenKind::Number)
//             | Some(TokenKind::String)
//             | Some(TokenKind::Newline)
//             | Some(TokenKind::Indent)
//             | Some(TokenKind::Dedent) => {
//                 parser.next(); // juste consommer pour le moment
//             }
//             Some(_) => {
//                 parser.next(); // fallback : avance
//             }
//             None => {
//                 break;
//             }
//         }
//     }

//     // NOTE Debugging VariableTable
//     println!("Local variable table : {:?}", parser.variable_table);

//     // NOTE Debugging ExportTable
//     println!("Local export table : {:?}", parser.export_table);

//     statements
// }
