use crate::language::syntax::ast::DurationValue;
use anyhow::{Result, anyhow};

/// Convert a parsed Devalang duration into seconds for a given tempo.
///
/// Temporal identifiers use the internal normalized format
/// `__temporal__<beats>_beat`, so all named units are resolved to beats before
/// reaching runtime.
pub fn duration_to_seconds(duration: &DurationValue, bpm: f32) -> Result<f32> {
    if !bpm.is_finite() || bpm <= 0.0 {
        return Err(anyhow!("Invalid BPM for duration conversion: {}", bpm));
    }

    match duration {
        DurationValue::Milliseconds(ms) => Ok(ms / 1000.0),
        DurationValue::Beats(beats) => Ok(beats * seconds_per_beat(bpm)?),
        DurationValue::Beat(value) => parse_beat_fraction(value)
            .map(|beats| beats * seconds_per_beat(bpm).unwrap_or(0.0))
            .ok_or_else(|| anyhow!("Invalid beat fraction: {}", value)),
        DurationValue::Number(ms) => Ok(ms / 1000.0),
        DurationValue::Identifier(identifier) => parse_temporal_identifier(identifier)
            .map(|beats| beats * seconds_per_beat(bpm).unwrap_or(0.0))
            .ok_or_else(|| anyhow!("Cannot resolve identifier duration: {}", identifier)),
        DurationValue::Auto => Ok(seconds_per_beat(bpm)?),
    }
}

pub fn seconds_per_beat(bpm: f32) -> Result<f32> {
    if !bpm.is_finite() || bpm <= 0.0 {
        Err(anyhow!("Invalid BPM for beat duration: {}", bpm))
    } else {
        Ok(60.0 / bpm)
    }
}

fn parse_beat_fraction(value: &str) -> Option<f32> {
    if let Ok(beats) = value.trim().parse::<f32>() {
        return Some(beats);
    }

    let mut parts = value.split('/');
    let numerator = parts.next()?.trim().parse::<f32>().ok()?;
    let denominator = parts.next()?.trim().parse::<f32>().ok()?;
    if parts.next().is_some() || denominator.abs() <= f32::EPSILON {
        return None;
    }

    Some(numerator / denominator)
}

fn parse_temporal_identifier(identifier: &str) -> Option<f32> {
    let rest = identifier.strip_prefix("__temporal__")?;
    let last_underscore = rest.rfind('_')?;
    rest[..last_underscore].parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_to_seconds_converts_supported_units() {
        assert_eq!(
            duration_to_seconds(&DurationValue::Milliseconds(500.0), 120.0).unwrap(),
            0.5
        );
        assert_eq!(
            duration_to_seconds(&DurationValue::Beats(2.0), 120.0).unwrap(),
            1.0
        );
        assert_eq!(
            duration_to_seconds(&DurationValue::Beat("1/4".to_string()), 120.0).unwrap(),
            0.125
        );
        assert_eq!(
            duration_to_seconds(
                &DurationValue::Identifier("__temporal__4_beat".to_string()),
                120.0,
            )
            .unwrap(),
            2.0
        );
    }

    #[test]
    fn test_duration_to_seconds_rejects_invalid_values() {
        assert!(duration_to_seconds(&DurationValue::Auto, 0.0).is_err());
        assert!(duration_to_seconds(&DurationValue::Beat("1/0".to_string()), 120.0).is_err());
        assert!(
            duration_to_seconds(&DurationValue::Identifier("missing".to_string()), 120.0).is_err()
        );
    }
}
