pub mod at;
pub mod brace;
pub mod colon;
pub mod comment;
pub mod dot;
pub mod equal;
pub mod identifier;
pub mod newline;
pub mod number;
pub mod indent;
pub mod string;

use crate::core::lexer::{
    handler::{
        at::handle_at_lexer,
        brace::{ handle_lbrace_lexer, handle_rbrace_lexer },
        colon::handle_colon_lexer,
        comment::handle_comment_lexer,
        dot::handle_dot_lexer,
        equal::handle_equal_lexer,
        identifier::handle_identifier_lexer,
        indent::handle_indent_lexer,
        newline::handle_newline_lexer,
        number::handle_number_lexer,
        string::handle_string_lexer,
    },
    token::{ Token, TokenKind },
};

fn advance_char<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    line: &mut usize,
    column: &mut usize
) -> Option<char> {
    let c = chars.next()?;
    if c == '\n' {
        // Do not increment column on newline here
    } else {
        *column += 1;
    }
    Some(c)
}

pub fn handle_content_lexing(content: String) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    let mut line = 1;
    let mut column = 1;

    let mut indent_stack: Vec<usize> = vec![0];
    let mut current_indent = 0;
    let mut at_line_start = true;

    let mut chars = content.chars().peekable();

    while chars.peek().is_some() {
        if at_line_start {
            handle_indent_lexer(
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            at_line_start = false;
        }

        let Some(ch) = advance_char(&mut chars, &mut line, &mut column) else {
            break;
        };

        match ch {
            '\n' => {
                handle_newline_lexer(
                    ch,
                    &mut chars,
                    &mut tokens,
                    &mut line,
                    &mut column,
                    &mut at_line_start,
                    &mut current_indent
                );
            }
            ' ' | '\t' => {
                // Already handled by indent_lexer
            }
            '#' => {
                handle_comment_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            ':' => {
                handle_colon_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '=' => {
                handle_equal_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '{' => {
                handle_lbrace_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '}' => {
                handle_rbrace_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '.' => {
                handle_dot_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '@' => {
                handle_at_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            '\"' | '\'' => {
                handle_string_lexer(
                    ch,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            c if c.is_ascii_digit() => {
                handle_number_lexer(
                    c,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            c if c.is_ascii_alphabetic() => {
                handle_identifier_lexer(
                    c,
                    &mut chars,
                    &mut current_indent,
                    &mut indent_stack,
                    &mut tokens,
                    &mut line,
                    &mut column
                );
            }
            _ => {
                // Ignore unknown char
            }
        }
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
        line: line + 1,
        column: 0,
        indent: 0,
    });

    // NOTE: Debug only
    // for token in &tokens {
    //     println!(
    //         "{:?} @ line {}, col {}, indent {}",
    //         token.kind,
    //         token.line,
    //         token.column,
    //         token.indent
    //     );
    // }

    Ok(tokens)
}
