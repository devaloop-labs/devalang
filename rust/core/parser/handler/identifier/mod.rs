pub mod let_;
pub mod group;
pub mod call;
pub mod spawn;
pub mod sleep;
pub mod synth;

use crate::core::{
    parser::{
        driver::Parser,
        handler::{
            identifier::{
                call::parse_call_token,
                group::parse_group_token,
                let_::parse_let_token,
                sleep::parse_sleep_token,
                spawn::parse_spawn_token,
                synth::parse_synth_token
            },
        },
        statement::Statement,
    },
    store::global::GlobalStore,
};

pub fn parse_identifier_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    let Some(current_token) = parser.peek_clone() else {
        return Statement::unknown();
    };

    let current_token_clone = current_token.clone();
    let current_token_lexeme = current_token_clone.lexeme.clone();

    let statement = match current_token_lexeme.as_str() {
        "let" => parse_let_token(parser, current_token_clone, global_store),
        "group" => parse_group_token(parser, current_token_clone, global_store),
        "call" => parse_call_token(parser, current_token_clone, global_store),
        "spawn" => parse_spawn_token(parser, current_token_clone, global_store),
        "sleep" => parse_sleep_token(parser, current_token_clone, global_store),
        "synth" => parse_synth_token(parser, current_token_clone, global_store),
        _ => {
            parser.advance(); // consume identifier

            println!("Unrecognized identifier: {}", current_token_lexeme);

            return Statement::error(current_token_clone, "Unexpected identifier".to_string());
        }
    };

    return statement;
}
