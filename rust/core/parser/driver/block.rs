use crate::core::lexer::token::Token;
use crate::core::lexer::token::TokenKind;
use crate::core::store::global::GlobalStore;

pub fn collect_block_tokens(
    parser: &mut crate::core::parser::driver::parser::Parser,
    base_indent: usize,
) -> Vec<Token> {
    let mut tokens = Vec::new();

    while let Some(tok) = parser.peek() {
        if tok.indent <= base_indent && tok.kind != TokenKind::Newline {
            break;
        }
        if let Some(t) = parser.advance() {
            tokens.push(t.clone());
        } else {
            // Unexpected EOF while collecting block tokens: stop collecting
            break;
        }
    }

    tokens
}

pub fn collect_until<F>(
    parser: &mut crate::core::parser::driver::parser::Parser,
    condition: F,
) -> Vec<Token>
where
    F: Fn(&Token) -> bool,
{
    let mut collected = Vec::new();
    while let Some(token) = parser.peek() {
        if condition(token) {
            break;
        }
        if token.kind == TokenKind::EOF {
            break;
        }
        if let Some(t) = parser.advance() {
            collected.push(t.clone());
        } else {
            break;
        }
    }

    collected
}

pub fn parse_block(
    parser: &crate::core::parser::driver::parser::Parser,
    tokens: Vec<Token>,
    global_store: &mut GlobalStore,
) -> Vec<crate::core::parser::statement::Statement> {
    let mut inner_parser = crate::core::parser::driver::parser::Parser {
        resolve_modules: parser.resolve_modules,
        tokens,
        token_index: 0,
        current_module: parser.current_module.clone(),
        previous: None,
    };

    inner_parser.parse_tokens(inner_parser.tokens.clone(), global_store)
}

pub fn parse_block_until_next_else(
    parser: &mut crate::core::parser::driver::parser::Parser,
    base_indent: usize,
    global_store: &mut GlobalStore,
) -> Vec<crate::core::parser::statement::Statement> {
    let mut block_tokens = Vec::new();

    while let Some(tok) = parser.peek() {
        // Stop if we encounter an 'else' at same indent level
        if tok.lexeme == "else" && tok.indent == base_indent {
            break;
        }
        if let Some(t) = parser.advance() {
            block_tokens.push(t.clone());
        } else {
            break;
        }
    }

    parse_block(parser, block_tokens, global_store)
}

pub fn parse_block_until_else_or_dedent(
    parser: &mut crate::core::parser::driver::parser::Parser,
    base_indent: usize,
    global_store: &mut GlobalStore,
) -> Vec<crate::core::parser::statement::Statement> {
    let mut tokens = Vec::new();

    while let Some(tok) = parser.peek() {
        if tok.lexeme == "else" && tok.indent == base_indent {
            break;
        }
        if tok.indent < base_indent && tok.kind != TokenKind::Newline {
            break;
        }
        if let Some(t) = parser.advance() {
            tokens.push(t.clone());
        } else {
            break;
        }
    }

    parse_block(parser, tokens, global_store)
}
