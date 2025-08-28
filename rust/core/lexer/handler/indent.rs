use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_indent_lexer(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    *current_indent = 0;
    let mut col = *column;

    while let Some(&c) = chars.peek() {
        if c == ' ' {
            *current_indent += 1;
            chars.next();
            col += 1;
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                lexeme: " ".to_string(),
                line: *line,
                column: col,
                indent: *current_indent,
            });
        } else if c == '\t' {
            *current_indent += 4;
            chars.next();
            col += 4;
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                lexeme: "\t".to_string(),
                line: *line,
                column: col,
                indent: *current_indent,
            });
        } else {
            break;
        }
    }

    *column = col;

    let last_indent = *indent_stack.last().unwrap();
    if *current_indent > last_indent {
        indent_stack.push(*current_indent);
        tokens.push(Token {
            kind: TokenKind::Indent,
            lexeme: String::from("<INDENT>"),
            line: *line,
            column: *column,
            indent: *current_indent,
        });
    } else {
        while *current_indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            tokens.push(Token {
                kind: TokenKind::Dedent,
                lexeme: String::from("<DEDENT>"),
                line: *line,
                column: *column,
                indent: *current_indent,
            });
        }
    }
}
