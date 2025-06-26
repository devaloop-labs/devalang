use crate::core::types::token::{Token, TokenKind};

pub fn handle_single_quote_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut string_content = String::new();
    *column += 1;

    while let Some(next_ch) = chars.next() {
        *column += 1;
        if next_ch == '\'' {
            break;
        } else {
            string_content.push(next_ch);
        }
    }

    tokens.push(Token {
        kind: TokenKind::String,
        lexeme: string_content,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}

pub fn handle_double_quote_lexer(
    char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize
) {
    let mut string_content = String::new();
    *column += 1; // skip the opening quote

    while let Some(next_ch) = chars.next() {
        *column += 1;
        if next_ch == '"' {
            break; // closing quote reached
        } else {
            string_content.push(next_ch);
        }
    }

    tokens.push(Token {
        kind: TokenKind::String,
        lexeme: string_content,
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
