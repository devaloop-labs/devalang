use anyhow::{Context, Result};

use super::utils::{compute_indent, is_identifier_start, lex_identifier, lex_number};
use crate::language::syntax::tokens::{Keyword, Token, TokenKind, keyword_from_ident};

#[derive(Debug, Default)]
pub struct Lexer {
    source: String,
}

impl Lexer {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            source: content.into(),
        }
    }

    pub fn with_source(mut self, content: impl Into<String>) -> Self {
        self.source = content.into();
        self
    }

    pub fn lex(self) -> Result<Vec<Token>> {
        lex_source(&self.source)
    }
}

fn lex_source(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut indent_stack = vec![0usize];

    for (line_idx, raw_line) in source.lines().enumerate() {
        let (indent_level, cursor_start) = compute_indent(raw_line);
        let trimmed = raw_line[cursor_start..].trim_end();

        let current_indent = *indent_stack.last().context("indent stack corruption")?;
        let line_number = line_idx + 1;

        if indent_level > current_indent {
            indent_stack.push(indent_level);
            tokens.push(Token::new(
                TokenKind::Indent,
                String::new(),
                indent_level,
                line_number,
                1,
            ));
        } else {
            while indent_level < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(Token::new(
                    TokenKind::Dedent,
                    String::new(),
                    indent_level,
                    line_number,
                    1,
                ));
            }
        }

        let mut cursor = cursor_start;
        let bytes = raw_line.as_bytes();
        let len = raw_line.len();

        while cursor < len {
            let ch = raw_line.as_bytes()[cursor];
            let column = cursor + 1;

            match ch {
                b' ' | b'\t' => {
                    cursor += 1;
                }
                b'#' => {
                    // Comment to end of line
                    if !trimmed.is_empty() {
                        tokens.push(Token::new(
                            TokenKind::Comment,
                            raw_line[cursor..].trim().to_string(),
                            indent_level,
                            line_number,
                            column,
                        ));
                    }
                    break;
                }
                b'"' | b'\'' => {
                    let quote = ch as char;
                    let mut end = cursor + 1;
                    let mut escaped = false;
                    while end < len {
                        let c = bytes[end] as char;
                        if c == quote && !escaped {
                            end += 1;
                            break;
                        }
                        escaped = !escaped && c == '\\';
                        end += 1;
                    }
                    let lexeme = &raw_line[cursor..end];
                    tokens.push(Token::new(
                        TokenKind::String,
                        lexeme,
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor = end;
                }
                b'0'..=b'9' => {
                    let (end, kind) = lex_number(raw_line, cursor);
                    let lexeme = &raw_line[cursor..end];
                    tokens.push(Token::new(kind, lexeme, indent_level, line_number, column));
                    cursor = end;
                }
                b'@' => {
                    tokens.push(Token::new(
                        TokenKind::Keyword(Keyword::At),
                        "@",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'-' => {
                    if cursor + 1 < len && bytes[cursor + 1] == b'>' {
                        tokens.push(Token::new(
                            TokenKind::Arrow,
                            "->",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 2;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Minus,
                            "-",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
                b'=' => {
                    if cursor + 1 < len && bytes[cursor + 1] == b'=' {
                        tokens.push(Token::new(
                            TokenKind::DoubleEquals,
                            "==",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 2;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Equals,
                            "=",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
                b'!' => {
                    if cursor + 1 < len && bytes[cursor + 1] == b'=' {
                        tokens.push(Token::new(
                            TokenKind::NotEquals,
                            "!=",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 2;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Unknown,
                            "!",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
                b'>' => {
                    if cursor + 1 < len && bytes[cursor + 1] == b'=' {
                        tokens.push(Token::new(
                            TokenKind::GreaterEqual,
                            ">=",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 2;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Greater,
                            ">",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
                b'<' => {
                    if cursor + 1 < len && bytes[cursor + 1] == b'=' {
                        tokens.push(Token::new(
                            TokenKind::LessEqual,
                            "<=",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 2;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Less,
                            "<",
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
                b'{' => {
                    tokens.push(Token::new(
                        TokenKind::LBrace,
                        "{",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'}' => {
                    tokens.push(Token::new(
                        TokenKind::RBrace,
                        "}",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'[' => {
                    tokens.push(Token::new(
                        TokenKind::LBracket,
                        "[",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b']' => {
                    tokens.push(Token::new(
                        TokenKind::RBracket,
                        "]",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'(' => {
                    tokens.push(Token::new(
                        TokenKind::LParen,
                        "(",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b')' => {
                    tokens.push(Token::new(
                        TokenKind::RParen,
                        ")",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b',' => {
                    tokens.push(Token::new(
                        TokenKind::Comma,
                        ",",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b':' => {
                    tokens.push(Token::new(
                        TokenKind::Colon,
                        ":",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'+' => {
                    tokens.push(Token::new(
                        TokenKind::Plus,
                        "+",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'*' => {
                    tokens.push(Token::new(
                        TokenKind::Asterisk,
                        "*",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'/' => {
                    tokens.push(Token::new(
                        TokenKind::Slash,
                        "/",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                b'.' => {
                    tokens.push(Token::new(
                        TokenKind::Dot,
                        ".",
                        indent_level,
                        line_number,
                        column,
                    ));
                    cursor += 1;
                }
                _ => {
                    if is_identifier_start(ch as char) {
                        let end = lex_identifier(raw_line, cursor);
                        let ident = &raw_line[cursor..end];
                        let lower = ident.to_ascii_lowercase();
                        let kind = if let Some(keyword) = keyword_from_ident(&lower) {
                            TokenKind::Keyword(keyword)
                        } else if lower == "true" || lower == "false" {
                            TokenKind::Boolean
                        } else {
                            TokenKind::Identifier
                        };
                        tokens.push(Token::new(kind, ident, indent_level, line_number, column));
                        cursor = end;
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Unknown,
                            (ch as char).to_string(),
                            indent_level,
                            line_number,
                            column,
                        ));
                        cursor += 1;
                    }
                }
            }
        }

        if !trimmed.is_empty() {
            tokens.push(Token::new(
                TokenKind::Newline,
                "\\n",
                indent_level,
                line_number,
                raw_line.len() + 1,
            ));
        }
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::new(
            TokenKind::Dedent,
            String::new(),
            0,
            source.lines().count() + 1,
            1,
        ));
    }

    tokens.push(Token::new(
        TokenKind::Eof,
        String::new(),
        0,
        source.lines().count() + 1,
        1,
    ));

    Ok(tokens)
}
