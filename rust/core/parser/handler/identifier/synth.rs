use std::collections::HashMap;

use crate::core::{
    lexer::token::Token,
    parser::{ driver::Parser, statement::{ Statement, StatementKind } },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_synth_token(
    parser: &mut Parser,
    _current_token: Token,
    _global_store: &mut GlobalStore
) -> Statement {
    parser.advance(); // consume 'synth'

    let Some(synth_token) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Expect an identifier (synth waveform)
    let Some(identifier_token) = parser.peek_clone() else {
        return Statement::error(synth_token, "Expected identifier after 'synth'".to_string());
    };

    let synth_waveform = identifier_token.lexeme.clone();

    parser.advance(); // consume identifier

    // Expect synth optional parameters map
    let parameters = if let Some(params) = parser.parse_map_value() {
        // If parameters are provided, we expect a map
        if let Value::Map(map) = params {
            map
        } else {
            return Statement::error(synth_token, "Expected a map for synth parameters".to_string());
        }
    } else {
        // If no parameters are provided, we can still create the statement with an empty map
        HashMap::new()
    };

    Statement {
        kind: StatementKind::Synth,
        value: Value::Map(
            HashMap::from([
                ("entity".to_string(), Value::String("synth".to_string())),
                (
                    "value".to_string(),
                    Value::Map(
                        HashMap::from([
                            ("waveform".to_string(), Value::String(synth_waveform)),
                            ("parameters".to_string(), Value::Map(parameters)),
                        ])
                    ),
                ),
            ])
        ),
        indent: synth_token.indent,
        line: synth_token.line,
        column: synth_token.column,
    }
}
