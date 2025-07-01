use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_number_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut number = char.to_string();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            number.push(c);
            chars.next();
            *column += 1;
        } else {
            break;
        }
    }

    tokens.push(Token {
        kind: TokenKind::Number,
        lexeme: number,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
