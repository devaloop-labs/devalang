

use crate::core::{
    lexer::{
        at::handle_at_lexer,
        brace::{ handle_lbrace_lexer, handle_rbrace_lexer },
        colon::handle_colon_lexer,
        comment::handle_comment_lexer,
        equal::handle_equal_lexer,
        newline::handle_newline_lexer,
        bracket::{ handle_lbracket_lexer, handle_rbracket_lexer },
        dot::handle_dot_lexer,
        identifier::handle_identifier_lexer,
        indent::handle_indent_lexer,
        number::handle_number_lexer,
        quote::{ handle_double_quote_lexer, handle_single_quote_lexer },
    },
    types::token::{ Token, TokenKind },
};

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
            let (new_tokens, new_indent_stack, new_line, new_column) = handle_indent_lexer(
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            // Update the main tokens vector and indent stack
            tokens = new_tokens;
            indent_stack = new_indent_stack;
            line = new_line;
            column = new_column;

            // Reset at_line_start flag
            at_line_start = false;
        }

        // Read the next character
        let Some(ch) = chars.next() else {
            break;
        };

        // Newline handling
        if ch == '\n' {
            handle_newline_lexer(
                ch,
                &mut chars,
                &mut tokens,
                &mut line,
                &mut column,
                &mut at_line_start,
                &mut current_indent
            );

            continue;
        }

        if ch == ' ' || ch == '\t' {
            column += if ch == '\t' { 4 } else { 1 };
            continue;
        }

        if ch == '#' {
            handle_comment_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == ':' {
            handle_colon_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '=' {
            handle_equal_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '[' {
            handle_lbracket_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == ']' {
            handle_rbracket_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '{' {
            handle_lbrace_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '}' {
            handle_rbrace_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '"' {
            handle_double_quote_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '\'' {
            handle_single_quote_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '.' {
            handle_dot_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch == '@' {
            handle_at_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch.is_ascii_digit() {
            handle_number_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

            continue;
        }

        if ch.is_ascii_alphabetic() {
            handle_identifier_lexer(
                ch,
                &mut chars,
                &mut current_indent,
                &mut indent_stack,
                &mut tokens,
                &mut line,
                &mut column
            );

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
