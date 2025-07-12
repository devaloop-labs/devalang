use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_operator_lexer(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let next = chars.peek().copied();

    let (kind, len) = match (ch, next) {
        ('=', Some('=')) => (TokenKind::DoubleEquals, 2),
        ('!', Some('=')) => (TokenKind::NotEquals, 2),
        ('>', Some('=')) => (TokenKind::GreaterEqual, 2),
        ('<', Some('=')) => (TokenKind::LessEqual, 2),
        ('=', _) => (TokenKind::Equals, 1),
        ('>', _) => (TokenKind::Greater, 1),
        ('<', _) => (TokenKind::Less, 1),
        _ => {
            return;
        }
    };

    if len == 2 {
        chars.next(); // consume second char
    }

    tokens.push(Token {
        kind,
        lexeme: if len == 2 {
            format!("{}{}", ch, next.unwrap())
        } else {
            ch.to_string()
        },
        line: *line,
        column: *column,
        indent: *current_indent,
    });

    *column += len;
}
