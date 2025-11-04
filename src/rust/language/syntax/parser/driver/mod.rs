pub mod directive;
pub mod duration;
pub mod effects;
pub mod helpers;
pub mod preprocessing;
pub mod routing;
pub mod statements;
pub mod trigger;
// Re-export statement-level parsers so they are available at driver root
use crate::language::syntax::ast::nodes::{Statement, StatementKind, Value};
use anyhow::{Result, anyhow};
pub use statements::*;
use std::path::{Path, PathBuf};

/// Find the closest keyword suggestion using Levenshtein distance
/// Returns the suggestion if distance is <= 2 (typo-like)
fn find_keyword_suggestion(input: &str, keywords: &[&str]) -> Option<String> {
    // Calculate Levenshtein distance between two strings
    fn levenshtein(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1, // deletion
                        matrix[i + 1][j] + 1, // insertion
                    ),
                    matrix[i][j] + cost, // substitution
                );
            }
        }
        matrix[len1][len2]
    }

    let mut best = (usize::MAX, "");
    for &keyword in keywords {
        let distance = levenshtein(input, keyword);
        if distance < best.0 && distance <= 2 {
            best = (distance, keyword);
        }
    }

    if best.0 <= 2 {
        Some(best.1.to_string())
    } else {
        None
    }
}

/// Entry point: parse source into a list of Statements
pub fn parse(source: &str, path: PathBuf) -> Result<Vec<Statement>> {
    // Pre-process: merge ALL multiline statements with braces
    let braces_pre = preprocessing::preprocess_multiline_braces(source);

    // Then merge multiline arrow calls (without braces)
    let preprocessed = preprocessing::preprocess_multiline_arrow_calls(&braces_pre);

    let lines: Vec<_> = preprocessed.lines().collect();
    parse_lines(&lines, 0, lines.len(), 0, &path)
}

/// Parse a range of lines into statements, handling indentation for blocks.
fn parse_lines(
    lines: &Vec<&str>,
    start: usize,
    end: usize,
    indent: usize,
    path: &Path,
) -> Result<Vec<Statement>> {
    use crate::language::syntax::ast::nodes::StatementKind;

    let mut i = start;
    let mut statements: Vec<Statement> = Vec::new();

    while i < end {
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        let current_indent = raw.len() - raw.trim_start().len();
        if current_indent < indent {
            break;
        }

        // parse header line
        let mut statement = parse_line(trimmed, i + 1, path)?;
        statement.indent = current_indent;
        statement.line = i + 1;

        // determine body range for block statements
        let body_start = i + 1;
        let mut body_end = body_start;
        while body_end < end {
            let l = lines[body_end];
            if l.trim().is_empty() || l.trim().starts_with('#') {
                body_end += 1;
                continue;
            }
            let indent_l = l.len() - l.trim_start().len();
            if indent_l <= current_indent {
                break;
            }
            body_end += 1;
        }

        // If we found a body, parse it and attach appropriately based on kind
        if body_end > body_start {
            let body = parse_lines(lines, body_start, body_end, current_indent + 1, path)?;

            // To avoid borrowing `statement.kind` and then assigning to it
            // (which the borrow checker forbids), take ownership of the kind
            // temporarily and then replace it with the updated version.
            let orig_kind = std::mem::replace(&mut statement.kind, StatementKind::Unknown);
            match orig_kind {
                StatementKind::If { condition, .. } => {
                    // Keep If simple here; attach else/else-if later in post-processing
                    statement.kind = StatementKind::If {
                        condition,
                        body: body.clone(),
                        else_body: None,
                    };
                }
                StatementKind::For {
                    variable, iterable, ..
                } => {
                    statement.kind = StatementKind::For {
                        variable,
                        iterable,
                        body: body.clone(),
                    };
                }
                StatementKind::Loop { count, .. } => {
                    statement.kind = StatementKind::Loop {
                        count,
                        body: body.clone(),
                    };
                }
                StatementKind::On { event, args, .. } => {
                    statement.kind = StatementKind::On {
                        event,
                        args,
                        body: body.clone(),
                    };
                }
                StatementKind::Automate { target } => {
                    statement.kind = StatementKind::Automate { target };
                    // store raw body as value map if needed elsewhere
                    let raw_lines: Vec<String> = lines[body_start..body_end]
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    let raw_body = raw_lines.join("\n");
                    let mut map = std::collections::HashMap::new();
                    map.insert("body".to_string(), Value::String(raw_body));
                    statement.value = Value::Map(map);
                }
                StatementKind::Function {
                    name, parameters, ..
                } => {
                    statement.kind = StatementKind::Function {
                        name: name.clone(),
                        parameters,
                        body: body.clone(),
                    };
                    statement.value = Value::Identifier(name.clone());
                }
                StatementKind::Group { .. } => {
                    // attach body for groups, keep name in value if present
                    let group_name = match &statement.value {
                        Value::Identifier(s) => s.clone(),
                        _ => "".to_string(),
                    };
                    statement.kind = StatementKind::Group {
                        name: group_name,
                        body: body.clone(),
                    };
                }
                StatementKind::Routing { .. } => {
                    // Parse routing body statements
                    statement.kind = StatementKind::Routing { body: body.clone() };
                }
                StatementKind::Tempo {
                    value,
                    body: Some(_),
                } => {
                    // Attach body to tempo block
                    statement.kind = StatementKind::Tempo {
                        value,
                        body: Some(body.clone()),
                    };
                }
                other => {
                    // keep original kind for other statements
                    statement.kind = other;
                }
            }

            // If this statement was a plain 'else' marker (Comment with value "else"),
            // the parsed body was not attached to the marker (we intentionally
            // keep the marker lightweight). We must append the parsed body
            // statements into the parent statements vector so the post-process
            // pass can collect them and attach to the previous If.
            i = body_end;
            statements.push(statement);

            if let StatementKind::Comment = &statements.last().unwrap().kind {
                if let Value::String(s) = &statements.last().unwrap().value {
                    if s == "else" {
                        // append the body statements after the marker
                        for stmt in body.into_iter() {
                            statements.push(stmt);
                        }
                    }
                }
            }
            continue;
        }

        // No body found
        statements.push(statement);
        i += 1;
    }

    // Post-process to attach else/else-if blocks
    fn attach_else_blocks(statements: &mut Vec<Statement>) {
        use crate::language::syntax::ast::StatementKind;

        // Helper: attach a new else-body (Vec<Statement>) to an existing If Statement.
        // If the If already has a nested else-if chain (encoded as a single If in
        // its else_body vector), descend into that chain and attach to the deepest
        // nested If instead of overwriting the entire else_body.
        fn attach_else_to_if_statement(target: Statement, new_else: Vec<Statement>) -> Statement {
            use crate::language::syntax::ast::StatementKind;

            match target.kind {
                StatementKind::If {
                    condition,
                    body,
                    else_body,
                } => {
                    match else_body {
                        None => Statement::new(
                            StatementKind::If {
                                condition,
                                body,
                                else_body: Some(new_else),
                            },
                            target.value,
                            target.indent,
                            target.line,
                            target.column,
                        ),
                        Some(mut eb) => {
                            if eb.len() == 1 {
                                let inner = eb.remove(0);
                                // If inner is an If, recurse into it
                                if let StatementKind::If { .. } = inner.kind {
                                    let updated_inner =
                                        attach_else_to_if_statement(inner, new_else);
                                    Statement::new(
                                        StatementKind::If {
                                            condition,
                                            body,
                                            else_body: Some(vec![updated_inner]),
                                        },
                                        target.value,
                                        target.indent,
                                        target.line,
                                        target.column,
                                    )
                                } else {
                                    // Not an If inside else_body; overwrite
                                    Statement::new(
                                        StatementKind::If {
                                            condition,
                                            body,
                                            else_body: Some(new_else),
                                        },
                                        target.value,
                                        target.indent,
                                        target.line,
                                        target.column,
                                    )
                                }
                            } else {
                                // Multiple statements in existing else_body; overwrite for now
                                Statement::new(
                                    StatementKind::If {
                                        condition,
                                        body,
                                        else_body: Some(new_else),
                                    },
                                    target.value,
                                    target.indent,
                                    target.line,
                                    target.column,
                                )
                            }
                        }
                    }
                }
                _ => target,
            }
        }

        let mut idx = 0;
        while idx < statements.len() {
            // Handle plain 'else' marker: Comment with value "else"
            if let StatementKind::Comment = &statements[idx].kind {
                if let Value::String(s) = &statements[idx].value {
                    if s == "else" {
                        let else_indent = statements[idx].indent;

                        // Collect following statements that are part of the else body
                        let mut body: Vec<Statement> = Vec::new();
                        let j = idx + 1;
                        while j < statements.len() && statements[j].indent > else_indent {
                            body.push(statements.remove(j));
                        }

                        // Remove the 'else' marker itself
                        statements.remove(idx);

                        // Find previous If to attach to (search backwards) and attach
                        // the parsed else body to the deepest nested If in the chain.
                        let mut k = idx as isize - 1;
                        while k >= 0 {
                            if let StatementKind::If { .. } = &statements[k as usize].kind {
                                // Take ownership of the previous If statement, attach into it,
                                // then put back the updated statement.
                                let prev = std::mem::replace(
                                    &mut statements[k as usize],
                                    Statement::new(StatementKind::Unknown, Value::Null, 0, 0, 1),
                                );
                                let updated = attach_else_to_if_statement(prev, body);
                                statements[k as usize] = updated;
                                break;
                            }
                            k -= 1;
                        }

                        continue;
                    }
                }
            }

            // Handle 'else if' which was parsed as an If with value == "else-if"
            if let StatementKind::If { .. } = &statements[idx].kind {
                if let Value::String(s) = &statements[idx].value {
                    if s == "else-if" {
                        // Remove this else-if statement and attach it as the
                        // else_body (single-statement vector) of the previous If.
                        let else_if_stmt = statements.remove(idx);

                        // find previous If and attach this else-if into its nested chain
                        let mut k = idx as isize - 1;
                        while k >= 0 {
                            if let StatementKind::If { .. } = &statements[k as usize].kind {
                                let prev = std::mem::replace(
                                    &mut statements[k as usize],
                                    Statement::new(StatementKind::Unknown, Value::Null, 0, 0, 1),
                                );
                                let updated = attach_else_to_if_statement(prev, vec![else_if_stmt]);
                                statements[k as usize] = updated;
                                break;
                            }
                            k -= 1;
                        }

                        continue;
                    }
                }
            }

            idx += 1;
        }
    }

    attach_else_blocks(&mut statements);

    Ok(statements)
}

fn parse_line(line: &str, line_number: usize, path: &Path) -> Result<Statement> {
    use crate::language::syntax::parser::driver::statements::*;

    if line.starts_with('@') {
        return directive::parse_directive(line, line_number, path);
    }

    if line.starts_with('.') {
        return trigger::parse_trigger_line(line, line_number);
    }

    let mut parts = line.split_whitespace();
    // Extract first token as keyword and strip a trailing ':' if present
    let first_token = parts
        .next()
        .ok_or_else(|| anyhow!("empty line"))?
        .to_string();
    let keyword = first_token.trim_end_matches(':').to_lowercase();

    // Check for routing statements (node, fx, route, duck, sidechain)
    let routing_keywords = ["node", "fx", "route", "duck", "sidechain"];
    if routing_keywords.contains(&keyword.as_str()) {
        return crate::language::syntax::parser::driver::routing::parse_routing_statement(
            line,
            line_number,
        );
    }

    // Check if this is a property assignment first: target.property = value
    if line.contains('=') && keyword.contains('.') {
        return parse_assign(line, line_number);
    }

    // Check if this looks like a trigger (contains a dot like "drums.kick")
    if keyword.contains('.') && !keyword.contains('(') {
        // Reconstruct the line and parse as trigger
        return trigger::parse_trigger_line(line, line_number);
    }

    // Check if this is a bind statement first (before arrow call)
    if keyword == "bind" && line.contains("->") {
        return parse_bind(line, line_number);
    }

    // If this line contains an arrow call, parse it as an ArrowCall, UNLESS the
    // statement starts with a reserved keyword that must be handled (let/var/const/etc.).
    // This ensures constructs like `let name = .bank.kick -> reverse(...)` are
    // parsed by `parse_let` rather than being mis-parsed as an ArrowCall.
    let reserved_keywords = [
        "bpm", "tempo", "print", "sleep", "rest", "wait", "pattern", "bank", "let", "var", "const",
        "for", "loop", "if", "else", "group", "automate", "call", "spawn", "on", "emit", "routing",
        "return", "break",
    ];
    if line.contains("->") && !reserved_keywords.contains(&keyword.as_str()) {
        return statements::parse_arrow_call(line, line_number);
    }

    return match keyword.as_str() {
        "bpm" | "tempo" => statements::core::parse_tempo(line, line_number),
        "print" => statements::core::parse_print(line, line_number),
        "sleep" | "rest" | "wait" => statements::core::parse_sleep(parts, line_number),
        "trigger" => Err(anyhow!(
            "keyword 'trigger' is deprecated; use dot notation like '.alias' instead"
        )),
        "pattern" => parse_pattern(parts, line_number),
        "bank" => parse_bank(parts, line_number),
        "let" => parse_let(line, parts, line_number),
        "var" => parse_var(line, parts, line_number),
        "const" => parse_const(line, parts, line_number),
        "for" => parse_for(parts, line_number),
        "loop" => parse_loop(parts, line_number),
        "if" => statements::structure::parse_if(parts, line_number),
        "else" => statements::structure::parse_else(line, line_number),
        "group" => statements::structure::parse_group(parts, line_number),
        "automate" => {
            crate::language::syntax::parser::driver::statements::structure::parse_automate(
                parts,
                line_number,
            )
        }
        "call" => parse_call(line, parts, line_number),
        "break" => statements::structure::parse_break(parts, line_number),
        "function" => statements::structure::parse_function(line, line_number),
        "spawn" => parse_spawn(parts, line_number),
        "on" => parse_on(parts, line_number),
        "emit" => parse_emit(line, parts, line_number),
        "return" => statements::core::parse_return(line, line_number),
        "routing" => {
            crate::language::syntax::parser::driver::routing::parse_routing_command(line_number)
        }
        _ => {
            // Provide helpful suggestions for common typos FIRST
            let suggestion = find_keyword_suggestion(&keyword, &reserved_keywords);

            // If we found a suggestion, it's likely a typo, not a trigger call
            if suggestion.is_some() {
                let suggestion_str = suggestion.unwrap();
                // Store message in a structured format: "MESSAGE|||FILE:LINE|||SUGGESTION"
                // This allows the collector to parse it and build a StructuredError
                let error_msg = format!(
                    "Unknown statement '{}' at {}:{}|||{}|||{}",
                    keyword,
                    path.display(),
                    line_number,
                    path.display(),
                    suggestion_str
                );

                // Push structured error to WASM registry if available
                #[cfg(feature = "wasm")]
                {
                    use crate::web::registry::debug;
                    if debug::is_debug_errors_enabled() {
                        debug::push_parse_error_from_parts(
                            format!(
                                "Unknown statement '{}'. Did you mean '{}' ?",
                                keyword, suggestion_str
                            ),
                            line_number,
                            1,
                            "UnknownStatement".to_string(),
                        );
                    }
                }

                return Ok(Statement::new(
                    StatementKind::Unknown,
                    Value::String(error_msg),
                    0,
                    line_number,
                    1,
                ));
            }

            // Check if this looks like a potential trigger identifier (single word, no special chars except dots)
            // This handles cases like variable references: let kick = drums.kick; kick
            if keyword
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                // Parse as trigger
                return trigger::parse_trigger_line(line, line_number);
            }

            // Generic error for truly unknown statements
            // Format: "MESSAGE|||FILE:LINE|||NO_SUGGESTION"
            let error_msg = format!(
                "Unknown statement '{}' at {}:{}|||{}|||",
                keyword,
                path.display(),
                line_number,
                path.display()
            );

            // Push structured error to WASM registry if available
            #[cfg(feature = "wasm")]
            {
                use crate::web::registry::debug;
                if debug::is_debug_errors_enabled() {
                    debug::push_parse_error_from_parts(
                        format!("Unknown statement '{}'", keyword),
                        line_number,
                        1,
                        "UnknownStatement".to_string(),
                    );
                }
            }

            return Ok(Statement::new(
                StatementKind::Unknown,
                Value::String(error_msg),
                0,
                line_number,
                1,
            ));
        }
    };
}

/// Re-export helper parsing functions from helpers.rs so other modules can call them
pub use helpers::{
    parse_array_value, parse_condition, parse_function_args, parse_map_value, parse_single_arg,
    parse_synth_definition,
};

/// SimpleParser is a small wrapper used by other modules/tests in the crate.
/// It forwards to the top-level parse implementation above.
pub struct SimpleParser;

impl SimpleParser {
    pub fn parse(source: &str, path: PathBuf) -> Result<Vec<Statement>> {
        crate::language::syntax::parser::driver::parse(source, path)
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Statement>> {
        let buf = std::path::PathBuf::from(path.as_ref());
        let s = std::fs::read_to_string(&buf)?;
        Self::parse(&s, buf)
    }
}
