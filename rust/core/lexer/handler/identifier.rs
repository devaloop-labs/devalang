use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_identifier_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut ident = char.to_string();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
            chars.next();
            *column += 1;
        } else {
            break;
        }
    }

    let kind = match ident.as_str() {
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "bank" => TokenKind::Bank,
        "bpm" => TokenKind::Tempo,
        "loop" => TokenKind::Loop,
        _ => TokenKind::Identifier,
    };

    tokens.push(Token {
        kind: kind.clone(),
        lexeme: ident,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
