use super::super::duration::parse_duration_token;
use super::super::helpers::{parse_array_value, parse_synth_definition};
use crate::language::syntax::ast::{Statement, StatementKind, Value};
/// Core statement parsing: tempo, print, let, var, const, sleep, bank
use anyhow::{Result, anyhow};
use std::iter::Iterator;

/// Parse tempo/bpm statement
pub fn parse_tempo(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let value = parts
        .next()
        .ok_or_else(|| anyhow!("tempo declaration requires a value"))?;
    let bpm: f32 = value
        .as_ref()
        .parse()
        .map_err(|_| anyhow!("invalid tempo value: '{}'", value.as_ref()))?;
    Ok(Statement::tempo(bpm, line_number, 1))
}

/// Parse print statement
pub fn parse_print(line: &str, line_number: usize) -> Result<Statement> {
    let message = line
        .strip_prefix("print")
        .ok_or_else(|| {
            anyhow!(
                "Invalid print statement: expected 'print' keyword at line {}",
                line_number
            )
        })?
        .trim();

    // If message is a quoted string, keep it as String; if it's a number, parse as Number;
    // otherwise treat as an identifier (variable name) so it can be resolved at runtime.
    if message.starts_with('"') && message.ends_with('"') && message.len() >= 2 {
        let cleaned = message[1..message.len() - 1].to_string();
        Ok(Statement::new(
            StatementKind::Print,
            Value::String(cleaned),
            0,
            line_number,
            1,
        ))
    } else if let Ok(num) = message.parse::<f32>() {
        Ok(Statement::new(
            StatementKind::Print,
            Value::Number(num),
            0,
            line_number,
            1,
        ))
    } else {
        // Support simple concatenation using '+' operator in print expressions, e.g.
        //   print "Loop iteration: " + i
        // We split on '+' and create a Value::Array of parts so runtime can resolve
        // each part and join them when printing.
        if message.contains('+') {
            let parts: Vec<Value> = message
                .split('+')
                .map(|p| p.trim())
                .filter(|s| !s.is_empty())
                .map(|tok| {
                    crate::language::syntax::parser::driver::helpers::parse_single_arg(tok)
                        .unwrap_or(Value::Identifier(tok.to_string()))
                })
                .collect();

            Ok(Statement::new(
                StatementKind::Print,
                Value::Array(parts),
                0,
                line_number,
                1,
            ))
        } else {
            // Attempt to parse the message as a full expression (supports function calls)
            let val = crate::language::syntax::parser::driver::helpers::parse_single_arg(message)?;
            Ok(Statement::new(StatementKind::Print, val, 0, line_number, 1))
        }
    }
}

/// Parse sleep statement
pub fn parse_sleep(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let value = parts
        .next()
        .ok_or_else(|| anyhow!("sleep instruction requires a duration"))?;
    let duration = parse_duration_token(value.as_ref())?;
    Ok(Statement::new(
        StatementKind::Sleep,
        Value::Duration(duration),
        0,
        line_number,
        1,
    ))
}

/// Parse bank statement
pub fn parse_bank(
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("bank declaration requires a name"))?
        .as_ref()
        .to_string();

    let alias = if let Some(word) = parts.next() {
        if word.as_ref() == "as" {
            parts.next().map(|v| v.as_ref().to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok(Statement::new(
        StatementKind::Bank { name, alias },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse let statement
pub fn parse_let(
    line: &str,
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("let statement requires a name"))?
        .as_ref()
        .to_string();

    let remainder = line
        .splitn(2, '=')
        .nth(1)
        .map(|r| r.trim().to_string())
        .unwrap_or_default();

    let value = if remainder.is_empty() {
        None
    } else if remainder.starts_with("synth ") {
        // Reuse existing synth parsing logic (supports chained params and param maps)
        if remainder.contains("->") {
            let stmt =
                crate::language::syntax::parser::driver::parse_arrow_call(&remainder, line_number)?;
            let mut synth_map = std::collections::HashMap::new();
            let first_part = remainder.split("->").next().unwrap_or("synth");
            let synth_def_val = parse_synth_definition(first_part)?;
            let mut param_map_present = false;
            if let Value::Map(m) = synth_def_val {
                param_map_present = m
                    .iter()
                    .any(|(k, v)| k != "type" && k != "waveform" && matches!(v, Value::Map(_)));
                synth_map = m;
            }

            if let Value::Map(chain_container) = stmt.value {
                let mut effects_arr: Vec<Value> = Vec::new();

                if let Some(Value::String(first_method)) = chain_container.get("method") {
                    let mut handled_as_synth_param = false;
                    if let Some(Value::Array(args_arr)) = chain_container.get("args") {
                        if first_method == "type" {
                            if let Some(first) = args_arr.get(0) {
                                match first {
                                    Value::String(s) => {
                                        synth_map.insert(
                                            "synth_type".to_string(),
                                            Value::String(s.clone()),
                                        );
                                        handled_as_synth_param = true;
                                    }
                                    Value::Identifier(id) => {
                                        synth_map.insert(
                                            "synth_type".to_string(),
                                            Value::String(id.clone()),
                                        );
                                        handled_as_synth_param = true;
                                    }
                                    _ => {}
                                }
                            }
                        } else if first_method == "adsr" {
                            if let Some(first) = args_arr.get(0) {
                                if let Value::Map(arg_map) = first {
                                    for (k, v) in arg_map.iter() {
                                        if ["attack", "decay", "sustain", "release"]
                                            .contains(&k.as_str())
                                        {
                                            synth_map.insert(k.clone(), v.clone());
                                        }
                                    }
                                    handled_as_synth_param = true;
                                }
                            }
                        }
                    }

                    if !handled_as_synth_param {
                        let mut eff_map = std::collections::HashMap::new();
                        eff_map.insert("type".to_string(), Value::String(first_method.clone()));
                        if let Some(Value::Array(args_arr)) = chain_container.get("args") {
                            if let Some(first) = args_arr.get(0) {
                                if let Value::Map(arg_map) = first {
                                    for (k, v) in arg_map.iter() {
                                        eff_map.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    eff_map.insert("value".to_string(), first.clone());
                                }
                            }
                        }
                        effects_arr.push(Value::Map(eff_map));
                    }
                }

                if let Some(Value::Array(chain_arr)) = chain_container.get("chain") {
                    for call_val in chain_arr.iter() {
                        if let Value::Map(call_map) = call_val {
                            if let Some(Value::String(mname)) = call_map.get("method") {
                                if mname == "type" {
                                    if let Some(Value::Array(args_arr)) = call_map.get("args") {
                                        if let Some(first) = args_arr.get(0) {
                                            match first {
                                                Value::String(s) => {
                                                    synth_map.insert(
                                                        "synth_type".to_string(),
                                                        Value::String(s.clone()),
                                                    );
                                                    continue;
                                                }
                                                Value::Identifier(id) => {
                                                    synth_map.insert(
                                                        "synth_type".to_string(),
                                                        Value::String(id.clone()),
                                                    );
                                                    continue;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                if mname == "adsr" {
                                    if let Some(Value::Array(args_arr)) = call_map.get("args") {
                                        if let Some(first) = args_arr.get(0) {
                                            if let Value::Map(arg_map) = first {
                                                for (k, v) in arg_map.iter() {
                                                    if ["attack", "decay", "sustain", "release"]
                                                        .contains(&k.as_str())
                                                    {
                                                        synth_map.insert(k.clone(), v.clone());
                                                    }
                                                }
                                                continue;
                                            }
                                        }
                                    }
                                }
                                let mut eff_map = std::collections::HashMap::new();
                                eff_map.insert("type".to_string(), Value::String(mname.clone()));
                                if let Some(Value::Array(args_arr)) = call_map.get("args") {
                                    if let Some(first) = args_arr.get(0) {
                                        if let Value::Map(arg_map) = first {
                                            for (k, v) in arg_map.iter() {
                                                eff_map.insert(k.clone(), v.clone());
                                            }
                                        } else {
                                            eff_map.insert("value".to_string(), first.clone());
                                        }
                                    }
                                }
                                effects_arr.push(Value::Map(eff_map));
                            }
                        }
                    }
                }

                if !effects_arr.is_empty() {
                    synth_map.insert("chain".to_string(), Value::Array(effects_arr));
                }
            }

            if param_map_present && synth_map.contains_key("chain") {
                eprintln!(
                    "DEPRECATION: chained params for synth with param map are deprecated — both will be merged, but prefer chained params."
                );
            }

            Some(Value::Map(synth_map))
        } else {
            Some(parse_synth_definition(&remainder)?)
        }
    } else if remainder.starts_with('[') && remainder.ends_with(']') {
        Some(parse_array_value(&remainder)?)
    } else if remainder.starts_with('.') {
        let stmt = crate::language::syntax::parser::driver::trigger::parse_trigger_line(
            &remainder,
            line_number,
        )?;
        Some(crate::language::syntax::ast::Value::Statement(Box::new(
            stmt,
        )))
    } else {
        let lower = remainder.to_lowercase();
        if lower == "true" {
            Some(Value::Boolean(true))
        } else if lower == "false" {
            Some(Value::Boolean(false))
        } else if let Ok(num) = remainder.parse::<f32>() {
            Some(Value::Number(num))
        } else {
            Some(Value::Identifier(remainder))
        }
    };

    Ok(Statement::new(
        StatementKind::Let { name, value },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse var statement
pub fn parse_var(
    line: &str,
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("var statement requires a name"))?
        .as_ref()
        .to_string();

    let remainder = line
        .splitn(2, '=')
        .nth(1)
        .map(|r| r.trim().to_string())
        .unwrap_or_default();

    let value = if remainder.is_empty() {
        None
    } else {
        // Try to parse booleans first, then numbers, then identifiers
        let lower = remainder.to_lowercase();
        if lower == "true" {
            Some(Value::Boolean(true))
        } else if lower == "false" {
            Some(Value::Boolean(false))
        } else if let Ok(num) = remainder.parse::<f32>() {
            Some(Value::Number(num))
        } else {
            Some(Value::Identifier(remainder))
        }
    };

    Ok(Statement::new(
        StatementKind::Var { name, value },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse const statement
pub fn parse_const(
    line: &str,
    mut parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("const statement requires a name"))?
        .as_ref()
        .to_string();

    let remainder = line
        .splitn(2, '=')
        .nth(1)
        .map(|r| r.trim().to_string())
        .unwrap_or_default();

    if remainder.is_empty() {
        return Err(anyhow!("const declaration requires initialization"));
    }

    // Try to parse booleans first, then numbers, then identifiers
    let value = {
        // Reuse the same parsing rules as `let` for RHS expressions so that
        // const can accept synth definitions, arrays, triggers, numbers, and identifiers.
        if remainder.starts_with("synth ") {
            if remainder.contains("->") {
                let stmt = crate::language::syntax::parser::driver::parse_arrow_call(
                    &remainder,
                    line_number,
                )?;
                Some(stmt.value)
            } else {
                Some(parse_synth_definition(&remainder)?)
            }
        } else if remainder.starts_with('[') && remainder.ends_with(']') {
            Some(parse_array_value(&remainder)?)
        } else if remainder.starts_with('.') {
            let stmt = crate::language::syntax::parser::driver::trigger::parse_trigger_line(
                &remainder,
                line_number,
            )?;
            Some(crate::language::syntax::ast::Value::Statement(Box::new(
                stmt,
            )))
        } else {
            let lower = remainder.to_lowercase();
            if lower == "true" {
                Some(Value::Boolean(true))
            } else if lower == "false" {
                Some(Value::Boolean(false))
            } else if let Ok(num) = remainder.parse::<f32>() {
                Some(Value::Number(num))
            } else {
                Some(Value::Identifier(remainder))
            }
        }
    };

    Ok(Statement::new(
        StatementKind::Const { name, value },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse return statement: return <expr>?
pub fn parse_return(line: &str, line_number: usize) -> Result<Statement> {
    // strip 'return' keyword
    let remainder = line
        .strip_prefix("return")
        .ok_or_else(|| anyhow!("invalid return statement"))?
        .trim();

    if remainder.is_empty() {
        Ok(Statement::new(
            StatementKind::Return { value: None },
            Value::Null,
            0,
            line_number,
            1,
        ))
    } else {
        // Parse a single argument/expression
        let val = crate::language::syntax::parser::driver::helpers::parse_single_arg(remainder)?;
        Ok(Statement::new(
            StatementKind::Return {
                value: Some(Box::new(val)),
            },
            Value::Null,
            0,
            line_number,
            1,
        ))
    }
}
