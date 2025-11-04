use crate::language::syntax::ast::DurationValue;
use anyhow::{Result, anyhow};

pub fn parse_duration_token(token: &str) -> Result<DurationValue> {
    let token = token.trim();
    if token.eq_ignore_ascii_case("auto") {
        return Ok(DurationValue::Auto);
    }

    // Check for milliseconds suffix (e.g., "500ms")
    if let Some(value) = token.strip_suffix("ms") {
        let ms: f32 = value
            .parse()
            .map_err(|_| anyhow!("invalid milliseconds duration: '{}'", token))?;
        return Ok(DurationValue::Milliseconds(ms));
    }

    // Check for bar/beat/measure suffixes (e.g., "1 bar", "2 beat", "3 measure")
    if let Some(result) = parse_temporal_duration(token) {
        return Ok(result);
    }

    // Check for fraction (e.g., "1/4", "1/8")
    if let Some(fraction) = parse_fraction(token) {
        return Ok(DurationValue::Beats(fraction));
    }

    // Try to parse as plain number (milliseconds)
    if let Ok(number) = token.parse::<f32>() {
        return Ok(DurationValue::Milliseconds(number));
    }

    // Fall back to identifier (variable reference)
    Ok(DurationValue::Identifier(token.to_string()))
}

/// Parse temporal duration formats like "1 bar", "2 beat", "3 measure"
/// Also supports: "1bar", "2beats", "1beat", "3 measures", etc.
/// Returns millisecond-based duration proportional to BPM
fn parse_temporal_duration(token: &str) -> Option<DurationValue> {
    // Try parsing with spaces first (e.g., "1 bar", "2 beats")
    let parts_with_space: Vec<&str> = token.split_whitespace().collect();
    if parts_with_space.len() == 2 {
        if let Some(result) = try_parse_temporal(parts_with_space[0], parts_with_space[1]) {
            return Some(result);
        }
    }

    // Try parsing without space (e.g., "1bar", "2beats")
    // Find where the unit starts (first alphabetic character)
    let split_pos = token.chars().position(|c| c.is_alphabetic())?;
    if split_pos == 0 {
        return None; // No number part
    }

    let num_str = &token[..split_pos];
    let unit_str = &token[split_pos..];

    try_parse_temporal(num_str, unit_str)
}

/// Helper function to parse temporal duration once we have number and unit separated
fn try_parse_temporal(num_str: &str, unit_str: &str) -> Option<DurationValue> {
    let count: f32 = num_str.trim().parse().ok()?;
    let unit = unit_str.trim().to_lowercase();

    // Remove trailing 's' if present (e.g., "beats" -> "beat", "bars" -> "bar")
    let unit_singular = if unit.ends_with('s') && unit.len() > 1 {
        &unit[..unit.len() - 1]
    } else {
        &unit
    };

    // 1 bar = 4 beats (at 4/4 time)
    // 1 beat at 120 BPM = 500ms (60000ms / 120 BPM)
    // 1 measure = 4 beats (same as bar)
    let multiplier = match unit_singular {
        "beat" => 1.0,
        "bar" => 4.0,
        "measure" => 4.0,
        _ => return None,
    };

    // We return Identifier with a special format to be resolved at runtime
    // Format: "__temporal__<count>_<unit>"
    let identifier = format!("__temporal__{}_{}s", count * multiplier, unit_singular);
    Some(DurationValue::Identifier(identifier))
}

fn parse_fraction(token: &str) -> Option<f32> {
    let mut split = token.split('/');
    let numerator: f32 = split.next()?.trim().parse().ok()?;
    let denominator: f32 = split.next()?.trim().parse().ok()?;
    if denominator.abs() < f32::EPSILON {
        return None;
    }
    Some(numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_auto() {
        assert!(matches!(
            parse_duration_token("auto"),
            Ok(DurationValue::Auto)
        ));
        assert!(matches!(
            parse_duration_token("AUTO"),
            Ok(DurationValue::Auto)
        ));
    }

    #[test]
    fn test_duration_milliseconds() {
        // Plain number
        match parse_duration_token("500") {
            Ok(DurationValue::Milliseconds(ms)) => assert_eq!(ms, 500.0),
            _ => panic!("Expected milliseconds"),
        }

        // With ms suffix
        match parse_duration_token("500ms") {
            Ok(DurationValue::Milliseconds(ms)) => assert_eq!(ms, 500.0),
            _ => panic!("Expected milliseconds"),
        }

        match parse_duration_token("1000ms") {
            Ok(DurationValue::Milliseconds(ms)) => assert_eq!(ms, 1000.0),
            _ => panic!("Expected milliseconds"),
        }
    }

    #[test]
    fn test_duration_fractions() {
        // Quarter note
        match parse_duration_token("1/4") {
            Ok(DurationValue::Beats(b)) => assert_eq!(b, 0.25),
            _ => panic!("Expected beats"),
        }

        // Half note
        match parse_duration_token("1/2") {
            Ok(DurationValue::Beats(b)) => assert_eq!(b, 0.5),
            _ => panic!("Expected beats"),
        }

        // Full note
        match parse_duration_token("1/1") {
            Ok(DurationValue::Beats(b)) => assert_eq!(b, 1.0),
            _ => panic!("Expected beats"),
        }
    }

    #[test]
    fn test_temporal_beat_with_space() {
        // "1 beat"
        match parse_duration_token("1 beat") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__1_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2 beat"
        match parse_duration_token("2 beat") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__2_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1 beats" (plural)
        match parse_duration_token("1 beats") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__1_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3 beats" (plural)
        match parse_duration_token("3 beats") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__3_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_temporal_beat_without_space() {
        // "1beat"
        match parse_duration_token("1beat") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__1_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2beat"
        match parse_duration_token("2beat") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__2_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1beats" (without space, with 's')
        match parse_duration_token("1beats") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__1_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3beats" (without space, with 's')
        match parse_duration_token("3beats") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__3_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_temporal_bar_with_space() {
        // "1 bar" -> 1 * 4 beats = 4
        match parse_duration_token("1 bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2 bar"
        match parse_duration_token("2 bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__8_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1 bars" (plural)
        match parse_duration_token("1 bars") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3 bars" (plural)
        match parse_duration_token("3 bars") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__12_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_temporal_bar_without_space() {
        // "1bar" -> 4 beats
        match parse_duration_token("1bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2bar"
        match parse_duration_token("2bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__8_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1bars" (without space, with 's')
        match parse_duration_token("1bars") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3bars" (without space, with 's')
        match parse_duration_token("3bars") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__12_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_temporal_measure_with_space() {
        // "1 measure" (same as bar: 4 beats)
        match parse_duration_token("1 measure") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2 measure"
        match parse_duration_token("2 measure") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__8_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1 measures" (plural)
        match parse_duration_token("1 measures") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3 measures" (plural)
        match parse_duration_token("3 measures") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__12_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_temporal_measure_without_space() {
        // "1measure" -> 4 beats
        match parse_duration_token("1measure") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "2measure"
        match parse_duration_token("2measure") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__8_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "1measures" (without space, with 's')
        match parse_duration_token("1measures") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // "3measures" (without space, with 's')
        match parse_duration_token("3measures") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__12_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_mixed_case() {
        // Case insensitive
        match parse_duration_token("1 BEAT") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__1_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        match parse_duration_token("2 Bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__8_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        match parse_duration_token("1MEASURE") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__4_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_float_values() {
        // Float beat values
        match parse_duration_token("0.5 beat") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__0.5_beats"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // Float bar values
        match parse_duration_token("1.5bar") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__6_bars"),
            other => panic!("Unexpected result: {:?}", other),
        }

        // Float measure values
        match parse_duration_token("2.5 measures") {
            Ok(DurationValue::Identifier(id)) => assert_eq!(id, "__temporal__10_measures"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }
}
