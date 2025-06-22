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

pub fn parse(
    tokens: Vec<Token>,
    mut parser: &mut Parser,
    global_store: &mut GlobalStore
) -> Vec<Statement> {
    let mut statements = Vec::new();

    while !parser.is_eof() {
        match parser.peek().map(|t| t.kind.clone()) {
            Some(TokenKind::Identifier) => {
                match parse_identifier(&mut parser) {
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
                match parse_dot(&mut parser) {
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

    // NOTE Debugging VariableTable
    // println!("{:?}", parser.variable_table);

    // NOTE Debugging ExportTable
    // println!("{:?}", parser.export_table);

    statements
}
