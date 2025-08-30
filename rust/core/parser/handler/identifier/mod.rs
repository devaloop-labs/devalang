pub mod automate;
pub mod call;
pub mod emit;
pub mod function;
pub mod group;
pub mod let_;
pub mod on;
pub mod print;
pub mod sleep;
pub mod spawn;
pub mod synth;

use crate::core::{
    parser::{
        driver::Parser,
        handler::identifier::{
            automate::parse_automate_token, call::parse_call_token, emit::parse_emit_token,
            group::parse_group_token, let_::parse_let_token, on::parse_on_token,
            print::parse_print_token, sleep::parse_sleep_token, spawn::parse_spawn_token,
            synth::parse_synth_token,
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

    match current_token_lexeme.as_str() {
        "let" => parse_let_token(parser, current_token_clone, global_store),
        "group" => parse_group_token(parser, current_token_clone, global_store),
        "call" => parse_call_token(parser, current_token_clone, global_store),
        "spawn" => parse_spawn_token(parser, current_token_clone, global_store),
        "sleep" => parse_sleep_token(parser, current_token_clone, global_store),
        "synth" => parse_synth_token(parser, current_token_clone, global_store),
        "automate" => parse_automate_token(parser, current_token_clone, global_store),
        "print" => parse_print_token(parser, current_token_clone, global_store),
        "on" => parse_on_token(parser, global_store),
        "emit" => parse_emit_token(parser, current_token_clone, global_store),
        _ => {
            parser.advance(); // consume identifier

            crate::core::parser::statement::error_from_token(
                current_token_clone,
                "Unexpected identifier".to_string(),
            )
        }
    }
}
