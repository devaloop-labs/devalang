use super::super::duration::parse_duration_token;
use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use anyhow::{Result, anyhow};

/// Parse fade statement: fade <in|out> <entity>.<property> during <duration>
/// Example: fade in synth.gain during 2 bars
pub fn parse_fade(line: &str, line_number: usize) -> Result<Statement> {
    let trimmed = line.trim_start();

    // Skip 'fade' keyword
    let rest = if trimmed.starts_with("fade ") {
        &trimmed[5..]
    } else {
        return Err(anyhow!("Invalid fade statement"));
    };

    // Parse: fade <in|out> <entity>.<property> during <duration>
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(anyhow!(
            "fade requires format: fade <in|out> <entity>.<property> during <duration>"
        ));
    }

    let direction = parts[0].to_string();
    if direction != "in" && direction != "out" {
        return Err(anyhow!(
            "fade direction must be 'in' or 'out', got '{}'",
            direction
        ));
    }

    let target = parts[1].to_string();
    if !target.contains('.') {
        return Err(anyhow!(
            "fade target must be in format entity.property, got '{}'",
            target
        ));
    }

    if parts[2] != "during" {
        return Err(anyhow!("Expected 'during' keyword in fade statement"));
    }

    // Parse duration (everything after 'over')
    let duration_str = parts[3..].join(" ");
    let duration = parse_duration_token(&duration_str)?;

    Ok(Statement::new(
        StatementKind::Fade {
            direction,
            target,
            duration,
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse section statement: section <name> [during <duration>]:
/// Example: section intro during 8 bars:
/// Example: section intro:  (duration auto-calculated from body)
pub fn parse_section(
    parts: impl Iterator<Item = impl AsRef<str>>,
    line_number: usize,
) -> Result<Statement> {
    let parts_vec: Vec<String> = parts.map(|p| p.as_ref().to_string()).collect();

    if parts_vec.is_empty() {
        return Err(anyhow!("section requires a name"));
    }

    let name = parts_vec[0].trim_end_matches(':').to_string();

    // Check if "during" keyword is present (optional)
    let duration = if parts_vec.len() > 1 && parts_vec[1] == "during" {
        // "during" keyword found
        if parts_vec.len() > 2 {
            // Parse duration (everything after 'during')
            let duration_str = parts_vec[2..].join(" ").trim_end_matches(':').to_string();
            Some(parse_duration_token(&duration_str)?)
        } else {
            return Err(anyhow!("section 'during' requires a duration value"));
        }
    } else {
        // No "during" keyword, duration will be auto-calculated
        None
    };

    Ok(Statement::new(
        StatementKind::Section {
            name,
            duration,
            body: Vec::new(), // Will be filled during indentation parsing
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse timeline statement: timeline [section1 during 4 bars, section2 during 8 beats, ...]
/// Example: timeline [intro during 8 bars, verse during 16 beats, chorus]
pub fn parse_timeline(line: &str, line_number: usize) -> Result<Statement> {
    let trimmed = line.trim_start();

    // Skip 'timeline' keyword
    let rest = if trimmed.starts_with("timeline ") {
        &trimmed[9..]
    } else {
        return Err(anyhow!("Invalid timeline statement"));
    };

    // Parse: timeline [item1, item2, ...]
    if !rest.starts_with('[') || !rest.ends_with(']') {
        return Err(anyhow!(
            "timeline requires format: timeline [section_name, ...]"
        ));
    }

    let inner = &rest[1..rest.len() - 1];
    let mut sections = Vec::new();

    // Split by comma and parse each section entry
    for entry in inner.split(',') {
        let entry = entry.trim();

        // Check if it has "during"
        if let Some(during_idx) = entry.find(" during ") {
            let section_name = entry[..during_idx].trim().to_string();
            let duration_str = entry[during_idx + 8..].trim();

            let duration = parse_duration_token(duration_str)?;
            sections.push((section_name, Some(duration)));
        } else {
            // Just section name, no duration override
            sections.push((entry.to_string(), None));
        }
    }

    if sections.is_empty() {
        return Err(anyhow!("timeline must have at least one section"));
    }

    Ok(Statement::new(
        StatementKind::Timeline { sections },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse schedule statement: schedule:
///   at <time>:
///     <statements>
///   at <time>:
///     <statements>
/// Example: schedule:
///   at 0 bars:
///     call myGroup
///   at 5 bars:
///     call myGroup2
pub fn parse_schedule(_line: &str, line_number: usize) -> Result<Statement> {
    // The actual parsing of schedule events happens during block indentation processing
    // This just creates an empty Schedule that will be populated by parse_lines()
    Ok(Statement::new(
        StatementKind::Schedule {
            events: Vec::new(), // Will be filled during indentation parsing
        },
        Value::Null,
        0,
        line_number,
        1,
    ))
}

/// Parse stop statement: stop <target> / stop all [after <duration>]
/// Aliases: silence, mute
/// Examples:
///   - stop all
///   - stop all after 2 bars
///   - stop synth.gain
///   - silence all
///   - mute myEntity
pub fn parse_stop(line: &str, line_number: usize) -> Result<Statement> {
    let trimmed = line.trim_start();

    // Handle aliases: "silence" and "mute"
    let rest = if trimmed.starts_with("stop ") {
        &trimmed[5..]
    } else if trimmed.starts_with("silence") {
        // "silence" is an alias for "stop all"
        return Ok(Statement::new(
            StatementKind::Stop {
                target: None,
                after: None,
            },
            Value::Null,
            0,
            line_number,
            1,
        ));
    } else if trimmed.starts_with("mute ") {
        &trimmed[5..]
    } else {
        return Err(anyhow!("Invalid stop statement"));
    };

    let parts: Vec<&str> = rest.split_whitespace().collect();

    if parts.is_empty() {
        return Err(anyhow!(
            "stop requires format: stop <entity|all> [after <duration>]"
        ));
    }

    // Check if it's "stop all" or "stop <entity>"
    if parts[0] == "all" {
        // Parse optional "after" clause
        let after = if parts.len() > 2 && parts[1] == "after" {
            let duration_str = parts[2..].join(" ");
            Some(parse_duration_token(&duration_str)?)
        } else if parts.len() > 1 {
            return Err(anyhow!(
                "Invalid stop all syntax. Expected: stop all [after <duration>]"
            ));
        } else {
            None
        };

        Ok(Statement::new(
            StatementKind::Stop {
                target: None,
                after,
            },
            Value::Null,
            0,
            line_number,
            1,
        ))
    } else {
        // stop <entity> format
        let target = parts[0].to_string();

        // Check for optional "after" clause
        let after = if parts.len() > 2 && parts[1] == "after" {
            let duration_str = parts[2..].join(" ");
            Some(parse_duration_token(&duration_str)?)
        } else if parts.len() > 1 {
            return Err(anyhow!(
                "Invalid stop syntax. Expected: stop <entity> [after <duration>]"
            ));
        } else {
            None
        };

        Ok(Statement::new(
            StatementKind::Stop {
                target: Some(target),
                after,
            },
            Value::Null,
            0,
            line_number,
            1,
        ))
    }
}

/// Parse a schedule event header: at <duration> [and during <optional_duration>]:
/// Returns the tuple (start_time, optional_duration) if valid
/// Examples:
///   - at 1 bars:
///   - at 1 bars and during 2 bars:
///   - at 0 bars and during 4 bars:
pub fn parse_schedule_event_header(line: &str) -> Result<(DurationValue, Option<DurationValue>)> {
    let trimmed = line.trim_start();

    if !trimmed.starts_with("at ") {
        return Err(anyhow!("Schedule event must start with 'at'"));
    }

    let rest = &trimmed[3..];
    let rest = rest.trim_end_matches(':').trim();

    // Check if there's "and during" in the line
    if let Some(and_during_idx) = rest.find(" and during ") {
        // Split into start time and optional duration
        let start_str = rest[..and_during_idx].trim();
        let duration_str = rest[and_during_idx + 12..].trim(); // 12 = len(" and during ")

        let start_time = parse_duration_token(start_str)?;
        let optional_duration = parse_duration_token(duration_str)?;

        Ok((start_time, Some(optional_duration)))
    } else {
        // No "and during" clause - only start time
        let start_time = parse_duration_token(rest)?;
        Ok((start_time, None))
    }
}
