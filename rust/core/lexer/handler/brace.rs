use crate::core::lexer::token::{Token, TokenKind};

pub fn handle_rbrace_lexer(
    ch: char,
    _chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    tokens.push(Token {
        kind: TokenKind::RBrace,
        lexeme: ch.to_string(),
        line: *line,
        column: *column,
        indent: *current_indent,
    });

    *column += 1;
}

pub fn handle_lbrace_lexer(
    ch: char,
    _chars: &mut std::iter::Peekable<std::str::Chars>,
    current_indent: &mut usize,
    _indent_stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    line: &mut usize,
    column: &mut usize,
) {
    tokens.push(Token {
        kind: TokenKind::LBrace,
        lexeme: ch.to_string(),
        line: *line,
        column: *column,
        indent: *current_indent,
    });

    *column += 1;
}
