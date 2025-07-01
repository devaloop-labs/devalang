use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_equal_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    if let Some('=') = chars.peek() {
        chars.next();
        tokens.push(Token {
            kind: TokenKind::DoubleEquals,
            lexeme: char.to_string(),
            line: *line,
            column: *column,
            indent: *current_indent,
        });
        *column += 2;
    } else {
        tokens.push(Token {
            kind: TokenKind::Equals,
            lexeme: char.to_string(),
            line: *line,
            column: *column,
            indent: *current_indent,
        });
        *column += 1;
    }
}
