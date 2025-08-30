use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_slash_lexer(
    ch: char,
    _chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut [usize],
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    let slash = ch.to_string();

    tokens.push(Token {
        kind: TokenKind::Slash,
        lexeme: slash,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
