use std::collections::HashMap;

use devalang_types::Value;

use crate::core::{
    lexer::token::Token,
    parser::{
        driver::parser::Parser,
        handler::dot::parse_dot_token,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};

pub fn parse_synth_token(
    parser: &mut Parser,
    _current_token: Token,
    _global_store: &mut GlobalStore,
) -> Statement {
    parser.advance(); // consume 'synth'

    let Some(synth_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Expect a provider/waveform identifier (can be dotted: alias.synth)
    // Also accept a dot-led entity by delegating to the dot parser (e.g. .module.export)
    let synth_waveform = if let Some(first_token) = parser.peek_clone() {
        use crate::core::lexer::token::TokenKind;

        if first_token.kind == TokenKind::Dot {
            // Parse dot-entity and extract its entity string
            let dot_stmt = parse_dot_token(parser, _global_store);
            // Extract entity if the parsed statement is a Trigger
            match dot_stmt.kind {
                StatementKind::Trigger { entity, .. } => entity,
                _ => String::new(),
            }
        } else {
            if first_token.kind != crate::core::lexer::token::TokenKind::Identifier
                && first_token.kind != crate::core::lexer::token::TokenKind::Number
                && first_token.kind != crate::core::lexer::token::TokenKind::Synth
            {
                return crate::core::parser::statement::error_from_token(
                    first_token.clone(),
                    "Expected identifier after 'synth'".to_string(),
                );
            }

            // Collect dotted parts on the same line
            let mut parts: Vec<String> = Vec::new();
            let current_line = first_token.line;
            loop {
                let Some(tok) = parser.peek_clone() else {
                    break;
                };
                if tok.line != current_line {
                    break;
                }
                match tok.kind {
                    crate::core::lexer::token::TokenKind::Identifier
                    | crate::core::lexer::token::TokenKind::Number
                    | crate::core::lexer::token::TokenKind::Synth => {
                        parts.push(tok.lexeme.clone());
                        parser.advance();
                        // If next isn't a dot on same line, stop
                        if let Some(next) = parser.peek_clone() {
                            if !(next.line == current_line
                                && next.kind == crate::core::lexer::token::TokenKind::Dot)
                            {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    crate::core::lexer::token::TokenKind::Dot => {
                        parser.advance();
                    }
                    _ => break,
                }
            }

            parts.join(".")
        }
    } else {
        return crate::core::parser::statement::error_from_token(
            synth_token,
            "Expected identifier after 'synth'".to_string(),
        );
    };

    // Skip formatting before optional parameters map
    while parser.check_token(crate::core::lexer::token::TokenKind::Newline)
        || parser.check_token(crate::core::lexer::token::TokenKind::Indent)
        || parser.check_token(crate::core::lexer::token::TokenKind::Dedent)
        || parser.check_token(crate::core::lexer::token::TokenKind::Whitespace)
    {
        parser.advance();
    }

    // Expect synth optional parameters map
    let parameters = if let Some(params) = parser.parse_map_value() {
        // If parameters are provided, we expect a map
        if let Value::Map(map) = params {
            map
        } else {
            return crate::core::parser::statement::error_from_token(
                synth_token,
                "Expected a map for synth parameters".to_string(),
            );
        }
    } else {
        // If no parameters are provided, we can still create the statement with an empty map
        HashMap::new()
    };

    Statement {
        kind: StatementKind::Synth,
        value: Value::Map(HashMap::from([
            ("entity".to_string(), Value::String("synth".to_string())),
            (
                "value".to_string(),
                Value::Map(HashMap::from([
                    // Store waveform as identifier to allow resolution from variables/exports
                    ("waveform".to_string(), Value::Identifier(synth_waveform)),
                    ("parameters".to_string(), Value::Map(parameters)),
                ])),
            ),
        ])),
        indent: synth_token.indent,
        line: synth_token.line,
        column: synth_token.column,
    }
}
