use anyhow::Result;
use std::collections::HashMap;

use crate::engine::audio::events::AudioEvent;
use crate::language::syntax::ast::Value;

use super::AudioInterpreter;

pub fn extract_pattern_data(
    _interpreter: &AudioInterpreter,
    value: &Value,
) -> (Option<String>, Option<HashMap<String, f32>>) {
    match value {
        Value::String(pattern) => (Some(pattern.clone()), None),
        Value::Map(map) => {
            let pattern = map.get("pattern").and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            });

            let mut options = HashMap::new();
            for (key, val) in map.iter() {
                if key != "pattern" {
                    if let Value::Number(num) = val {
                        options.insert(key.clone(), *num);
                    }
                }
            }

            let opts = if options.is_empty() {
                None
            } else {
                Some(options)
            };
            (pattern, opts)
        }
        _ => (None, None),
    }
}

pub fn execute_pattern(
    interpreter: &mut AudioInterpreter,
    target: &str,
    pattern: &str,
    options: Option<HashMap<String, f32>>,
) -> Result<()> {
    execute_pattern_with_source(interpreter, target, pattern, options, None)
}

pub fn execute_pattern_with_source(
    interpreter: &mut AudioInterpreter,
    target: &str,
    pattern: &str,
    options: Option<HashMap<String, f32>>,
    source: Option<&str>,
) -> Result<()> {
    let swing = options
        .as_ref()
        .and_then(|o| o.get("swing").copied())
        .unwrap_or(0.0);
    let humanize = options
        .as_ref()
        .and_then(|o| o.get("humanize").copied())
        .unwrap_or(0.0);
    let velocity_mult = options
        .as_ref()
        .and_then(|o| o.get("velocity").copied())
        .unwrap_or(1.0);
    let tempo_override = options.as_ref().and_then(|o| o.get("tempo").copied());

    let effective_bpm = tempo_override.unwrap_or(interpreter.bpm);
    let resolved_uri = resolve_sample_uri(interpreter, target);

    let pattern_chars: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
    let step_count = pattern_chars.len() as f32;
    if step_count == 0.0 {
        return Ok(());
    }

    let beat_duration =
        crate::engine::audio::duration::seconds_per_beat(effective_bpm).unwrap_or(0.0);
    let bar_duration = beat_duration * 4.0;
    let step_duration = if step_count > 0.0 {
        bar_duration / step_count
    } else {
        0.0
    };

    for (i, &ch) in pattern_chars.iter().enumerate() {
        let is_hit = match ch {
            'x' | 'X' | 'o' => true,
            _ => false,
        };

        if is_hit {
            let mut time = interpreter.cursor_time + (i as f32 * step_duration);
            if swing > 0.0 && i % 2 == 1 {
                time += step_duration * swing;
            }

            #[cfg(any(feature = "cli", feature = "wasm"))]
            if humanize > 0.0 {
                let offset = crate::engine::special_vars::gen_range_f32(-humanize, humanize);
                time += offset;
            }

            // v0.2.0: Support velocity variations based on symbol
            // x = normal (1.0), X = accent (1.3), o = soft (0.6), . = visual (skip)
            let velocity_modifier = match ch {
                'x' => 1.0, // Normal hit
                'X' => 1.3, // Accent (louder, may saturate)
                'o' => 0.6, // Soft hit (quieter)
                _ => 1.0,
            };

            let event = AudioEvent::Sample {
                node_id: interpreter.resolve_event_node_id(source.unwrap_or(target)),
                uri: resolved_uri.clone(),
                start_time: time,
                velocity: velocity_mult * velocity_modifier,
                effects: None,
                source: source.map(|s| s.to_string()),
            };
            interpreter.events.events.push(event);
        }
    }

    interpreter.cursor_time += bar_duration;
    Ok(())
}

pub fn resolve_sample_uri(interpreter: &AudioInterpreter, target: &str) -> String {
    if let Some(dot_pos) = target.find('.') {
        let bank_alias = &target[..dot_pos];
        let trigger_name = &target[dot_pos + 1..];
        if let Some(Value::Map(bank_map)) = interpreter.variables.get(bank_alias) {
            if let Some(Value::String(bank_name)) = bank_map.get("_name") {
                return format!("devalang://bank/{}/{}", bank_name, trigger_name);
            }
        }
    }
    target.to_string()
}
