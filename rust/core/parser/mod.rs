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
        loop_::parse_loop, tempo::parse_tempo,
    },
    types::{
        module::Module,
        parser::Parser,
        statement::Statement,
        store::{ GlobalStore, VariableTable },
        token::{ Token, TokenKind },
        variable::VariableValue,
    },
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

fn parse_variable_value(
    value: VariableValue,
    parser: &mut Parser,
    global_store: &mut GlobalStore
) -> VariableValue {
    match value {
        VariableValue::Text(text) => VariableValue::Text(text),
        VariableValue::Number(num) => VariableValue::Number(num),
        VariableValue::Array(tokens) => VariableValue::Array(tokens),
        _ => {
            eprintln!("⚠️ Unsupported variable value type: {:?}", value);
            VariableValue::Text("Unsupported type".to_string())
        }
    }
}
