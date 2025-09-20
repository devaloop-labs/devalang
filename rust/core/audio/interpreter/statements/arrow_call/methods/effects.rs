use crate::core::audio::engine::AudioEngine;
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};
use std::collections::HashMap;

fn parse_value_to_f32(v: &Value) -> Option<f32> {
    match v {
        Value::Number(n) => Some(*n as f32),
        Value::String(s) => s.parse::<f32>().ok(),
        Value::Identifier(s) => s.parse::<f32>().ok(),
        Value::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

// Simple nearest-neighbour resampler that produces an output with the same frame count
// while applying a time-scaling factor per frame. channels = interleaved channel count.
fn resample_segment_nearest(
    src: &[i16],
    channels: usize,
    start_rate: f32,
    end_rate: f32,
) -> Vec<i16> {
    if src.is_empty() || channels == 0 {
        return Vec::new();
    }

    let frames = src.len() / channels;
    if frames == 0 {
        return Vec::new();
    }

    // Copy source into frames x channels matrix access via frame*channels + ch
    let mut out = vec![0i16; frames * channels];

    for f in 0..frames {
        let t = if frames > 1 {
            f as f32 / (frames - 1) as f32
        } else {
            0.0
        };
        let rate = start_rate + t * (end_rate - start_rate);
        let inv_rate = if rate == 0.0 { 1.0 } else { 1.0 / rate };

        // determine source frame position (nearest)
        let src_frame_pos = (f as f32 * inv_rate).clamp(0.0, (frames - 1) as f32);
        let src_idx = src_frame_pos.round() as usize;

        for ch in 0..channels {
            let s_idx = src_idx.saturating_mul(channels).saturating_add(ch);
            let o_idx = f.saturating_mul(channels).saturating_add(ch);
            let sample = if s_idx < src.len() { src[s_idx] } else { 0 };
            out[o_idx] = sample;
        }
    }

    out
}

// Basic effect application for chainable effects. This is intentionally minimal:
// - Accepts a target synth name and args parsed from the arrow call.
// - Applies effects by mutating the AudioEngine or scheduling transforms.
// Current effects implemented: echo, reverb, slide (as parameter modifiers).

pub fn apply_effect_chain(
    method: &str,
    args: &Vec<Value>,
    target: &str,
    audio_engine: &mut AudioEngine,
    variable_table: &devalang_types::VariableTable,
) {
    match method {
        "echo" => apply_echo(args, target, audio_engine, variable_table),
        "reverb" => apply_reverb(args, target, audio_engine, variable_table),
        "slide" => apply_slide(args, target, audio_engine, variable_table),
        "arp" => apply_arp_effect(args, target, audio_engine, variable_table),
        _ => {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Error,
                &format!("Unknown chainable effect '{}' on '{}'.", method, target),
            );
        }
    }
}

fn parse_map_arg(args: &Vec<Value>) -> Option<HashMap<String, Value>> {
    // Typical usage: method({ key: value }) -> args[0] is a Map
    if let Some(Value::Map(m)) = args.first() {
        return Some(m.clone());
    }
    None
}

fn apply_echo(
    args: &Vec<Value>,
    _target: &str,
    engine: &mut AudioEngine,
    _variable_table: &devalang_types::VariableTable,
) {
    let map = parse_map_arg(args);
    let mut delay_ms = 250.0_f32;
    let mut feedback = 0.5_f32;

    if let Some(m) = map {
        if let Some(Value::Number(n)) = m.get("delay") {
            delay_ms = *n as f32;
        }
        if let Some(Value::Number(n)) = m.get("feedback") {
            feedback = *n as f32;
        }
    }

    // Very small and cheap echo: we will add a delayed, attenuated copy of the buffer
    let sample_rate = engine.sample_rate as f32;
    let channels = engine.channels as usize;
    let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize * channels;

    if delay_samples == 0 || engine.buffer.is_empty() {
        return;
    }

    // Mix a single echo pass
    let mut out = engine.buffer.clone();
    for i in delay_samples..engine.buffer.len() {
        let src = engine.buffer[i - delay_samples] as f32;
        let added = (src * feedback)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        out[i] = out[i].saturating_add(added);
    }

    engine.buffer = out;
}

fn apply_reverb(
    args: &Vec<Value>,
    _target: &str,
    engine: &mut AudioEngine,
    _variable_table: &devalang_types::VariableTable,
) {
    let map = parse_map_arg(args);
    let mut room_size = 0.5_f32;
    if let Some(m) = map {
        if let Some(Value::Number(n)) = m.get("room_size") {
            room_size = *n as f32;
        }
    }

    // Cheap reverb: multiple short comb filters (very approximate)
    if engine.buffer.is_empty() {
        return;
    }

    let sample_rate = engine.sample_rate as f32;
    let channels = engine.channels as usize;
    let reverb_delay_samples = ((0.03 * room_size) * sample_rate) as usize * channels;
    if reverb_delay_samples == 0 {
        return;
    }

    let mut out = engine.buffer.clone();
    for i in reverb_delay_samples..engine.buffer.len() {
        let src = engine.buffer[i - reverb_delay_samples] as f32;
        let added = (src * room_size * 0.5)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        out[i] = out[i].saturating_add(added);
    }

    engine.buffer = out;
}

fn apply_slide(
    args: &Vec<Value>,
    _target: &str,
    _engine: &mut AudioEngine,
    _variable_table: &devalang_types::VariableTable,
) {
    // Slide: apply a linear pitch glide across the most recently recorded note ranges
    let map = parse_map_arg(args);
    let mut from_semitones = 0.0_f32;
    let mut to_semitones = 0.0_f32;

    if let Some(m) = map {
        if let Some(v) = m.get("from") {
            if let Some(f) = parse_value_to_f32(v) {
                from_semitones = f;
            }
        }
        if let Some(v) = m.get("to") {
            if let Some(t) = parse_value_to_f32(v) {
                to_semitones = t;
            }
        }
    }

    // Compute rate multipliers from semitone offsets
    let start_rate = 2f32.powf(from_semitones / 12.0);
    let end_rate = 2f32.powf(to_semitones / 12.0);

    let channels = _engine.channels as usize;
    if channels == 0 {
        return;
    }

    // For each recorded last note range for the target, apply a per-frame resample glide
    if let Some(ranges) = _engine.last_notes.get(_target) {
        for (start_sample, total_samples) in ranges.iter() {
            if *total_samples == 0 {
                continue;
            }
            // clamp range
            let end = (*start_sample)
                .saturating_add(*total_samples)
                .min(_engine.buffer.len());
            if *start_sample >= end {
                continue;
            }

            // work on a copy of the segment
            let seg = _engine.buffer[*start_sample..end].to_vec();
            let processed = resample_segment_nearest(&seg, channels, start_rate, end_rate);

            // Mix processed back into buffer (replace to preserve duration)
            for i in 0..processed.len().min(seg.len()) {
                _engine.buffer[*start_sample + i] = processed[i];
            }
        }
    } else {
        let logger = Logger::new();
        logger.log_message(
            LogLevel::Warning,
            "Slide requested but no recent notes found for target",
        );
    }
}

fn apply_arp_effect(
    args: &Vec<Value>,
    _target: &str,
    _engine: &mut AudioEngine,
    _variable_table: &devalang_types::VariableTable,
) {
    // Arp effect: split the last note into N slices and re-pitch each slice across a spread
    let map = parse_map_arg(args);
    let mut steps: usize = 4;
    let mut spread_semitones: f32 = 0.0;

    if let Some(m) = map {
        if let Some(v) = m.get("steps") {
            if let Some(s) = parse_value_to_f32(v) {
                steps = (s as usize).max(1);
            }
        }
        if let Some(v) = m.get("spread") {
            if let Some(s) = parse_value_to_f32(v) {
                spread_semitones = s;
            }
        }
    }

    let channels = _engine.channels as usize;
    if channels == 0 {
        return;
    }

    if let Some(ranges) = _engine.last_notes.get(_target) {
        for (start_sample, total_samples) in ranges.iter() {
            if *total_samples == 0 {
                continue;
            }
            let end_sample = (*start_sample)
                .saturating_add(*total_samples)
                .min(_engine.buffer.len());
            if *start_sample >= end_sample {
                continue;
            }

            let seg = _engine.buffer[*start_sample..end_sample].to_vec();
            let frames = seg.len() / channels;
            if frames == 0 {
                continue;
            }

            // For each step, compute semitone and pitch multiplier and place slice at computed offset
            for step in 0..steps {
                let t = if steps > 1 {
                    step as f32 / (steps - 1) as f32
                } else {
                    0.0
                };
                let semis = t * spread_semitones;
                let rate = 2f32.powf(semis / 12.0);

                // Resample the entire segment to the same duration using nearest approach with rate
                let processed = resample_segment_nearest(&seg, channels, rate, rate);

                // place the processed slice starting at fractional positions across original segment
                let offset_frames =
                    ((t * frames as f32).round() as usize).min(frames.saturating_sub(1));
                let offset_samples = offset_frames.saturating_mul(channels);

                // mix into engine buffer
                for i in 0..processed.len() {
                    let dst_idx = *start_sample + offset_samples + i;
                    if dst_idx >= _engine.buffer.len() {
                        break;
                    }
                    // simple additive mix and clamp
                    let sum = (_engine.buffer[dst_idx] as i32) + (processed[i] as i32);
                    _engine.buffer[dst_idx] = sum.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                }
            }
        }
    } else {
        let logger = Logger::new();
        logger.log_message(
            LogLevel::Warning,
            "Arp effect requested but no recent notes found for target",
        );
    }
}
