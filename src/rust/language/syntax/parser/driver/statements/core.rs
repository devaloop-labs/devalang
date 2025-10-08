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
    let message = line.strip_prefix("print").unwrap().trim();

    // If message is a quoted string, keep it as String; if it's a number, parse as Number;
    // otherwise treat as an identifier (variable name) so it can be resolved at runtime.
    if message.starts_with('"') && message.ends_with('"') && message.len() >= 2 {
        let cleaned = message[1..message.len()-1].to_string();
        Ok(Statement::new(StatementKind::Print, Value::String(cleaned), 0, line_number, 1))
    } else if let Ok(num) = message.parse::<f32>() {
        Ok(Statement::new(StatementKind::Print, Value::Number(num), 0, line_number, 1))
    } else {
        // treat as identifier
        Ok(Statement::new(StatementKind::Print, Value::Identifier(message.to_string()), 0, line_number, 1))
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
        // Parse synth definition: synth waveform { params }
        Some(parse_synth_definition(&remainder)?)
    } else if remainder.starts_with('[') && remainder.ends_with(']') {
        // Parse as array: ["C4", "E4", "G4"] or [1..10]
        Some(parse_array_value(&remainder)?)
    } else {
        // Try to parse as number first
        if let Ok(num) = remainder.parse::<f32>() {
            Some(Value::Number(num))
        } else {
            // Otherwise treat as identifier
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
        // Try to parse as number first
        if let Ok(num) = remainder.parse::<f32>() {
            Some(Value::Number(num))
        } else {
            // Otherwise treat as identifier
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

    // Try to parse as number first
    let value = if let Ok(num) = remainder.parse::<f32>() {
        Some(Value::Number(num))
    } else {
        // Otherwise treat as identifier
        Some(Value::Identifier(remainder))
    };

    Ok(Statement::new(
        StatementKind::Const { name, value },
        Value::Null,
        0,
        line_number,
        1,
    ))
}
