use crate::core::lexer::token::{ Token, TokenKind };

pub fn handle_identifier_lexer(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut [usize],
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut ident = ch.to_string();

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
        "pattern" => TokenKind::Pattern,
        "bpm" => TokenKind::Tempo,
        "loop" => TokenKind::Loop,
        "for" => TokenKind::Loop,
        "synth" => TokenKind::Synth,
        "fn" => TokenKind::Function,
        "as" => TokenKind::As,
        "on" => TokenKind::On,
        "emit" => TokenKind::Emit,
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
