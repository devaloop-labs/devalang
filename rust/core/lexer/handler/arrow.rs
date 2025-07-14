use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_arrow_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut arrow_call = char.to_string();

    while let Some(&c) = chars.peek() {
        if c == '>' {
            chars.next();
            arrow_call.push(c);
            *column += 1;
        } else {
            break;
        }
    }

    tokens.push(Token {
        kind: TokenKind::Arrow,
        lexeme: arrow_call,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
