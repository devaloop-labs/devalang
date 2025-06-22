use crate::core::types::token::{ Token, TokenKind };

pub fn lex(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut line = 1;
    let mut column = 1;

    let mut indent_stack: Vec<usize> = vec![0];
    let mut current_indent = 0;
    let mut at_line_start = true;

    let mut chars = input.chars().peekable();

    while let Some(_) = chars.peek() {
        if at_line_start {
            current_indent = 0;

            while let Some(&c) = chars.peek() {
                if c == ' ' {
                    current_indent += 1;
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }

            let last_indent = *indent_stack.last().unwrap();
            if current_indent > last_indent {
                indent_stack.push(current_indent);
                tokens.push(Token {
                    kind: TokenKind::Indent,
                    lexeme: String::new(),
                    line,
                    column,
                    indent: current_indent,
                });
            } else {
                while current_indent < *indent_stack.last().unwrap() {
                    indent_stack.pop();
                    tokens.push(Token {
                        kind: TokenKind::Dedent,
                        lexeme: String::new(),
                        line,
                        column,
                        indent: current_indent,
                    });
                }
            }

            at_line_start = false;
        }

        let Some(ch) = chars.next() else {
            break;
        };

        if ch == '\n' {
            tokens.push(Token {
                kind: TokenKind::Newline,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });

            line += 1;
            column = 1;
            at_line_start = true;

            continue;
        }

        if ch == ' ' || ch == '\t' {
            column += if ch == '\t' { 4 } else { 1 };
            continue;
        }

        if ch == '#' {
            let mut comment = String::new();
            while let Some(&c) = chars.peek() {
                if c == '\n' {
                    break;
                }
                comment.push(c);
                chars.next();
                column += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Comment(comment.trim().to_string()),
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            continue;
        }

        if ch == ':' {
            tokens.push(Token {
                kind: TokenKind::Colon,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == '=' {
            if let Some('=') = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::DoubleEquals,
                    lexeme: ch.to_string(),
                    line,
                    column,
                    indent: current_indent,
                });
                column += 2;
            } else {
                tokens.push(Token {
                    kind: TokenKind::Equals,
                    lexeme: ch.to_string(),
                    line,
                    column,
                    indent: current_indent,
                });
                column += 1;
            }
            continue;
        }

        if ch == '[' {
            tokens.push(Token {
                kind: TokenKind::LBracket,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == ']' {
            tokens.push(Token {
                kind: TokenKind::RBracket,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == '{' {
            tokens.push(Token {
                kind: TokenKind::LBrace,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == '}' {
            tokens.push(Token {
                kind: TokenKind::RBrace,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == '"' {
            let mut string_content = String::new();
            column += 1; // skip the opening quote

            while let Some(next_ch) = chars.next() {
                column += 1;
                if next_ch == '"' {
                    break; // closing quote reached
                } else {
                    string_content.push(next_ch);
                }
            }

            tokens.push(Token {
                kind: TokenKind::String,
                lexeme: string_content,
                line,
                column,
                indent: current_indent,
            });

            continue;
        }

        if ch == '\'' {
            let mut string_content = String::new();
            column += 1;

            while let Some(next_ch) = chars.next() {
                column += 1;
                if next_ch == '\'' {
                    break;
                } else {
                    string_content.push(next_ch);
                }
            }

            tokens.push(Token {
                kind: TokenKind::String,
                lexeme: string_content,
                line,
                column,
                indent: current_indent,
            });

            continue;
        }

        if ch == '.' {
            tokens.push(Token {
                kind: TokenKind::Dot,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch == '@' {
            tokens.push(Token {
                kind: TokenKind::At,
                lexeme: ch.to_string(),
                line,
                column,
                indent: current_indent,
            });
            column += 1;
            continue;
        }

        if ch.is_ascii_digit() {
            let mut number = ch.to_string();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    number.push(c);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }

            // let value = number.parse::<f32>().unwrap();
            tokens.push(Token {
                kind: TokenKind::Number,
                lexeme: number,
                line,
                column,
                indent: current_indent,
            });
            continue;
        }

        if ch.is_ascii_alphabetic() {
            let mut ident = ch.to_string();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }

            let kind = match ident.as_str() {
                "bank" => TokenKind::Bank,
                "bpm" => TokenKind::Tempo,
                "loop" => TokenKind::Loop,
                _ => TokenKind::Identifier,
            };

            tokens.push(Token {
                kind,
                lexeme: ident,
                line,
                column,
                indent: current_indent,
            });
            continue;
        }

        // Skip unknown char
        column += 1;
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        current_indent = *indent_stack.last().unwrap();
        tokens.push(Token {
            kind: TokenKind::Dedent,
            lexeme: String::new(),
            line,
            column,
            indent: current_indent,
        });
    }

    tokens.push(Token {
        kind: TokenKind::EOF,
        lexeme: String::new(),
        line: line + 1, // EOF is considered to be on the next line
        column: 0, // EOF has no column
        indent: 0, // EOF has no indent
    });

    tokens
}
