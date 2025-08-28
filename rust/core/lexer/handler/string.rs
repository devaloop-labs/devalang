use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_string_lexer(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    let quote_char = ch;

    let start_column = *column;
    let start_line = *line;

    *column += 1;

    let mut string_content = String::new();

    while let Some(&next_ch) = chars.peek() {
        if next_ch == quote_char {
            chars.next();
            *column += 1;
            break;
        } else if next_ch == '\\' {
            chars.next();
            *column += 1;

            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => string_content.push('\n'),
                    't' => string_content.push('\t'),
                    '\\' => string_content.push('\\'),
                    '"' => string_content.push('"'),
                    '\'' => string_content.push('\''),
                    other => {
                        string_content.push('\\');
                        string_content.push(other);
                    }
                }
                *column += 1;
            }
        } else {
            chars.next();
            if next_ch == '\n' {
                *line += 1;
                *column = 1;
            } else {
                *column += 1;
            }
            string_content.push(next_ch);
        }
    }

    tokens.push(Token {
        kind: TokenKind::String,
        lexeme: string_content,
        indent: *current_indent,
        line: start_line,
        column: start_column,
    });
}
