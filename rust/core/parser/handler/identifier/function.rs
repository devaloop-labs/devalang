use crate::core::{
    lexer::{ token::TokenKind },
    parser::{ statement::{ Statement, StatementKind }, driver::Parser },
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn parse_function_token(parser: &mut Parser, global_store: &mut GlobalStore) -> Statement {
    parser.advance(); // consume 'fn'

    let fn_token = match parser.previous_clone() {
        Some(tok) => tok,
        None => return Statement::unknown(),
    };

    let name_token = match parser.peek_clone() {
        Some(tok) => tok,
        None => return Statement::error(fn_token, "Expected function name after 'fn'".to_string()),
    };

    if name_token.kind != TokenKind::Identifier {
        return Statement::error(
            name_token.clone(),
            "Expected function name to be an identifier".to_string()
        );
    }

    let function_name = name_token.lexeme.clone();
    parser.advance(); // consume function name

    let mut parameters = Vec::new();
    let mut body = Vec::new();

    // Expect '('
    if parser.peek_kind() != Some(TokenKind::LParen) {
        return Statement::error(name_token.clone(), "Expected '(' after function name".to_string());
    }
    parser.advance(); // consume '('

    // Parse parameters until ')'
    let tokens = parser.collect_until(|t| t.kind == TokenKind::RParen || t.kind == TokenKind::EOF);
    for token in tokens {
        if token.kind == TokenKind::Identifier {
            parameters.push(token.lexeme.clone());
        }
    }

    if parser.peek_kind() == Some(TokenKind::RParen) {
        parser.advance(); // consume ')'
    } else {
        return Statement::error(name_token.clone(), "Expected ')' after parameters".to_string());
    }

    // Expect colon
    if parser.peek_kind() != Some(TokenKind::Colon) {
        return Statement::error(name_token.clone(), "Expected ':' after ')'".to_string());
    }
    parser.advance(); // consume ':'

    // Collect ALL tokens indented after this line until Dedent
    let base_indent = fn_token.indent;
    let mut body_tokens = Vec::new();

    while let Some(tok) = parser.peek() {
        if tok.kind == TokenKind::Dedent && tok.indent <= base_indent {
            break;
        }
        body_tokens.push(parser.advance().unwrap().clone());
    }

    // arse those tokens into block statements
    body = parser.parse_block(body_tokens.clone(), global_store);

    // Skip Dedent if present
    if let Some(tok) = parser.peek() {
        if tok.kind == TokenKind::Dedent {
            parser.advance();
        }
    }

    Statement {
        kind: StatementKind::Function {
            name: function_name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
        },
        value: Value::Null,
        indent: fn_token.indent,
        line: fn_token.line,
        column: fn_token.column,
    }
}
