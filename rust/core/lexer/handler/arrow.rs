use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_arrow_lexer(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut [usize],
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    // If next char is '>', this is an arrow '->'.
    if let Some(&c) = chars.peek() {
        if c == '>' {
            let mut arrow_call = ch.to_string();
            chars.next();
            arrow_call.push(c);
            *column += 1;

            tokens.push(Token {
                kind: TokenKind::Arrow,
                lexeme: arrow_call,
                line: *line,
                column: *column,
                indent: *current_indent,
            });
            return;
        }
    }

    // Otherwise, treat '-' as the start of a negative number if followed by digits.
    let mut lexeme = String::from("-");
    if let Some(&next) = chars.peek() {
        if next.is_ascii_digit() {
            // consume digits
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    chars.next();
                    lexeme.push(d);
                    *column += 1;
                } else {
                    break;
                }
            }
            // optional decimal part
            if let Some(&dot) = chars.peek() {
                if dot == '.' {
                    chars.next();
                    lexeme.push(dot);
                    *column += 1;
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            chars.next();
                            lexeme.push(d);
                            *column += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(Token {
                kind: TokenKind::Number,
                lexeme,
                line: *line,
                column: *column,
                indent: *current_indent,
            });
            return;
        }
    }

    // Fallback: lone '-' not part of '->' or a number; emit Unknown to avoid mis-parsing as Arrow
    tokens.push(Token {
        kind: TokenKind::Unknown,
        lexeme: "-".to_string(),
        line: *line,
        column: *column,
        indent: *current_indent,
    });
}
