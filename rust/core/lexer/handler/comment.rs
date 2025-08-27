use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_comment_lexer(
    _char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut comment = String::new();

    while let Some(&c) = chars.peek() {
        if c == '\n' {
            break;
        }
        comment.push(c);
        chars.next();
        *column += 1;
    }

    tokens.push(Token {
        kind: TokenKind::Comment,
        lexeme: comment.trim().to_string(),
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
