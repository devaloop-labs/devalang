use std::{ collections::HashMap, iter };

use crate::core::types::{
    statement::{ Statement, StatementIterator, StatementKind },
    token::{ Token, TokenDuration, TokenKind, TokenParamValue },
    variable::VariableValue,
};

pub fn parse_loop(
    parser: &mut crate::core::parser::Parser,
    global_store: &mut crate::core::types::store::GlobalStore
) -> Result<crate::core::types::statement::Statement, String> {
    let token = parser.peek().ok_or("Unexpected EOF")?.clone();

    parser.next(); // consomme le mot-clé Loop

    // Récuperation des paramètres de la boucle
    let mut iterator = StatementIterator::Unknown;

    let iterable_tokens: Vec<Token> = parser.collect_until(|t| { t.kind == TokenKind::Colon });

    iterable_tokens.iter().for_each(|t| {
        match t.kind {
            TokenKind::Identifier => {
                iterator = StatementIterator::Identifier(t.lexeme.clone());
            }
            TokenKind::Number => {
                if let Ok(num) = t.lexeme.parse::<f32>() {
                    iterator = StatementIterator::Number(num);
                } else {
                    eprintln!("⚠️ Invalid number in loop iterator: {}", t.lexeme);
                }
            }
            TokenKind::Array => {
                println!("🔍 Parsing array in loop iterator: {:?}", t.lexeme);
                iterator = StatementIterator::Array(vec![]);
            }
            TokenKind::Map => {
                println!("🔍 Parsing map in loop iterator: {:?}", t.lexeme);
                iterator = StatementIterator::Map(HashMap::new());
            }
            _ => {
                eprintln!("⚠️ Unsupported token type in loop iterator: {:?}", t.kind);
            }
        }
    });

    parser.next(); // consomme le token de deux-points (:)

    // Remplir les paramètres de la boucle
    let loop_body_tokens: Vec<Token> = parser.collect_until(|t| { t.kind == TokenKind::Dedent });

    let loop_statement = Statement {
        kind: StatementKind::Loop {
            iterator,
        },
        value: VariableValue::Array(loop_body_tokens),
        line: token.line,
        column: token.column,
        indent: token.indent,
    };

    Ok(loop_statement)
}
