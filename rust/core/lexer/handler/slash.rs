use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_slash_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut slash = char.to_string();

    tokens.push(Token {
        kind: TokenKind::Slash,
        lexeme: slash,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
