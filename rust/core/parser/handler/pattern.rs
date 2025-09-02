use devalang_types::Value;

use crate::core::{
    lexer::token::TokenKind,
    parser::{
        driver::Parser,
        statement::{Statement, StatementKind},
    },
    store::global::GlobalStore,
};

pub fn parse_pattern_token(parser: &mut Parser, _global_store: &mut GlobalStore) -> Statement {
    // consume 'pattern'
    parser.advance();

    let Some(tok) = parser.previous_clone() else {
        return Statement::unknown();
    };

    // Parse pattern name
    let mut name = String::new();
    if let Some(next) = parser.peek_clone() {
        if next.kind == TokenKind::Identifier {
            parser.advance();
            name = next.lexeme.clone();
        }
    }

    // optional 'with <target>' sequence
    let mut target: Option<String> = None;
    if parser.peek_is("with") {
        parser.advance(); // consume 'with'
        if let Some(tok2) = parser.peek_clone() {
            // target can be identifier or dotted identifier
            if tok2.kind == TokenKind::Identifier {
                parser.advance();
                let mut base = tok2.lexeme.clone();
                if let Some(dot) = parser.peek_clone() {
                    if dot.kind == TokenKind::Dot {
                        parser.advance();
                        if let Some(suf) = parser.peek_clone() {
                            if suf.kind == TokenKind::Identifier || suf.kind == TokenKind::Number {
                                parser.advance();
                                base.push('.');
                                base.push_str(&suf.lexeme);
                            }
                        }
                    }
                }
                target = Some(base);
            }
        }
    }

    // optional '=' and pattern string
    let mut value: Value = Value::Null;
    if parser.peek_is("=") {
        parser.advance();
        if let Some(tok3) = parser.peek_clone() {
            if tok3.kind == TokenKind::String {
                parser.advance();
                value = Value::String(tok3.lexeme.clone());
            }
        }
    }

    Statement {
        kind: StatementKind::Pattern { name, target },
        value,
        indent: tok.indent,
        line: tok.line,
        column: tok.column,
    }
}
