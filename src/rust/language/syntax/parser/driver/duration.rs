use crate::language::syntax::ast::DurationValue;
use anyhow::{Result, anyhow};

pub fn parse_duration_token(token: &str) -> Result<DurationValue> {
    let token = token.trim();
    if token.eq_ignore_ascii_case("auto") {
        return Ok(DurationValue::Auto);
    }

    if let Some(value) = token.strip_suffix("ms") {
        let ms: f32 = value
            .parse()
            .map_err(|_| anyhow!("invalid milliseconds duration: '{}'", token))?;
        return Ok(DurationValue::Milliseconds(ms));
    }

    if let Some(fraction) = parse_fraction(token) {
        return Ok(DurationValue::Beats(fraction));
    }

    if let Ok(number) = token.parse::<f32>() {
        return Ok(DurationValue::Milliseconds(number));
    }

    Ok(DurationValue::Identifier(token.to_string()))
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
