use crate::core::lexer::token::{Token, TokenKind};

pub fn advance_impl(parser: &mut crate::core::parser::driver::parser::Parser) -> Option<&Token> {
    if is_eof_impl(parser) {
        return None;
    }

    parser.previous = parser.tokens.get(parser.token_index).cloned();
    parser.token_index += 1;

    parser.tokens.get(parser.token_index - 1)
}

pub fn peek_impl(parser: &crate::core::parser::driver::parser::Parser) -> Option<&Token> {
    parser.tokens.get(parser.token_index)
}

pub fn peek_clone_impl(parser: &crate::core::parser::driver::parser::Parser) -> Option<Token> {
    parser.tokens.get(parser.token_index).cloned()
}

pub fn peek_nth_impl(
    parser: &crate::core::parser::driver::parser::Parser,
    n: usize,
) -> Option<&Token> {
    if parser.token_index + n < parser.tokens.len() {
        parser.tokens.get(parser.token_index + n)
    } else {
        None
    }
}

pub fn peek_nth_kind_impl(
    parser: &crate::core::parser::driver::parser::Parser,
    n: usize,
) -> Option<TokenKind> {
    peek_nth_impl(parser, n).map(|t| t.kind.clone())
}

pub fn is_eof_impl(parser: &crate::core::parser::driver::parser::Parser) -> bool {
    parser.token_index >= parser.tokens.len()
}

pub fn previous_clone_impl(parser: &crate::core::parser::driver::parser::Parser) -> Option<Token> {
    parser.previous.clone()
}

pub fn match_token_impl(
    parser: &mut crate::core::parser::driver::parser::Parser,
    kind: TokenKind,
) -> bool {
    if let Some(tok) = peek_impl(parser) {
        if tok.kind == kind {
            advance_impl(parser);
            return true;
        }
    }
    false
}

pub fn advance_if_impl(
    parser: &mut crate::core::parser::driver::parser::Parser,
    kind: TokenKind,
) -> bool {
    match_token_impl(parser, kind)
}

pub fn peek_is_impl(parser: &crate::core::parser::driver::parser::Parser, expected: &str) -> bool {
    peek_impl(parser).is_some_and(|t| t.lexeme == expected)
}

pub fn expect_impl(
    parser: &mut crate::core::parser::driver::parser::Parser,
    kind: TokenKind,
) -> Result<&Token, String> {
    let tok = advance_impl(parser).ok_or("Unexpected end of input")?;
    if tok.kind == kind {
        Ok(tok)
    } else {
        Err(format!("Expected {:?}, got {:?}", kind, tok.kind))
    }
}
