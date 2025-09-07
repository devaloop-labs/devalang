use crate::core::lexer::token::TokenKind;
use crate::core::store::global::GlobalStore;
use devalang_types::Value;

pub fn parse_tokens_impl(
    parser: &mut crate::core::parser::driver::parser::Parser,
    tokens: Vec<crate::core::lexer::token::Token>,
    global_store: &mut GlobalStore,
) -> Vec<crate::core::parser::statement::Statement> {
    // Filter out Whitespace tokens but keep Newline tokens because some constructs (e.g., print ...)
    // rely on end-of-line semantics.
    parser.tokens = tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Whitespace)
        .collect();
    parser.token_index = 0;

    let mut statements = Vec::new();

    while !crate::core::parser::driver::cursor::is_eof_impl(parser) {
        let token = match crate::core::parser::driver::cursor::peek_impl(parser) {
            Some(t) => t.clone(),
            None => {
                break;
            }
        };

        if token.kind == TokenKind::Newline {
            crate::core::parser::driver::cursor::advance_impl(parser);
            continue;
        }

        let statement = match &token.kind {
            TokenKind::At => crate::core::parser::handler::at::parse_at_token(parser, global_store),
            TokenKind::Identifier => {
                if
                    let Some(next) = crate::core::parser::driver::cursor
                        ::peek_nth_impl(parser, 1)
                        .cloned()
                {
                    if next.kind == TokenKind::Arrow {
                        crate::core::parser::handler::arrow_call::parse_arrow_call(
                            parser,
                            global_store
                        )
                    } else {
                        crate::core::parser::handler::identifier::parse_identifier_token(
                            parser,
                            global_store
                        )
                    }
                } else {
                    crate::core::parser::handler::identifier::parse_identifier_token(
                        parser,
                        global_store
                    )
                }
            }
            TokenKind::Dot =>
                crate::core::parser::handler::dot::parse_dot_token(parser, global_store),
            TokenKind::Tempo =>
                crate::core::parser::handler::tempo::parse_tempo_token(parser, global_store),
            TokenKind::Bank =>
                crate::core::parser::handler::bank::parse_bank_token(parser, global_store),
            TokenKind::Pattern =>
                crate::core::parser::handler::pattern::parse_pattern_token(parser, global_store),
            TokenKind::Loop =>
                crate::core::parser::handler::loop_::parse_loop_token(parser, global_store),
            TokenKind::If =>
                crate::core::parser::handler::condition::parse_condition_token(
                    parser,
                    global_store
                ),
            TokenKind::Function =>
                crate::core::parser::handler::identifier::function::parse_function_token(
                    parser,
                    global_store
                ),
            TokenKind::On =>
                crate::core::parser::handler::identifier::on::parse_on_token(parser, global_store),
            TokenKind::Emit =>
                crate::core::parser::handler::identifier::emit::parse_emit_token(
                    parser,
                    token.clone(),
                    global_store
                ),

            | TokenKind::Else // Ignore else, already handled in `parse_condition_token`
            | TokenKind::Comment
            | TokenKind::Equals
            | TokenKind::Colon
            | TokenKind::Number
            | TokenKind::String
            | TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::Comma
            | TokenKind::Dedent
            | TokenKind::Indent => {
                crate::core::parser::driver::cursor::advance_impl(parser);
                continue;
            }

            TokenKind::EOF => {
                break;
            }

            _ => {
                crate::core::parser::driver::cursor::advance_impl(parser);
                crate::core::parser::statement::Statement::unknown_with_pos(
                    token.indent,
                    token.line,
                    token.column
                )
            }
        };

        statements.push(statement);
    }

    statements
}

pub fn parse_condition_until_colon_impl(
    parser: &mut crate::core::parser::driver::parser::Parser,
) -> Option<Value> {
    let tokens =
        crate::core::parser::driver::block::collect_until(parser, |t| t.kind == TokenKind::Colon);
    if tokens.is_empty() {
        return None;
    }

    let condition = tokens
        .iter()
        .map(|t| t.lexeme.clone())
        .collect::<Vec<_>>()
        .join(" ");

    Some(Value::String(condition))
}
