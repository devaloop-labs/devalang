use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_newline_lexer(
    ch: char,
    _chars: &mut std::iter::Peekable<std::str::Chars>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
    at_line_start: &mut bool,
    current_indent: &mut usize,
) {
    tokens.push(Token {
        kind: TokenKind::Newline,
        lexeme: ch.to_string(),
        line: *line,
        column: 0,
        indent: *current_indent,
    });

    *line += 1;
    *column = 1;
    *at_line_start = true;
}
