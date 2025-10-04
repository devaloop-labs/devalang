pub fn compute_indent(line: &str) -> (usize, usize) {
    let mut indent = 0usize;
    let mut cursor = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => {
                indent += 1;
                cursor += 1;
            }
            '\t' => {
                indent += 4;
                cursor += 1;
            }
            _ => break,
        }
    }
    (indent, cursor)
}

pub fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

pub fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '$' | '.')
}

pub fn lex_identifier(line: &str, start: usize) -> usize {
    let mut end = start;
    for ch in line[start..].chars() {
        if is_identifier_continue(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    end
}

pub fn lex_number(line: &str, start: usize) -> (usize, crate::language::syntax::tokens::TokenKind) {
    let mut end = start;
    let mut has_dot = false;
    for ch in line[start..].chars() {
        match ch {
            '0'..='9' => {
                end += 1;
            }
            '.' if !has_dot => {
                has_dot = true;
                end += 1;
            }
            _ => break,
        }
    }

    let mut kind = crate::language::syntax::tokens::TokenKind::Number;

    if line[end..].starts_with("ms") {
        end += 2;
        kind = crate::language::syntax::tokens::TokenKind::Duration;
    }

    (end, kind)
}
