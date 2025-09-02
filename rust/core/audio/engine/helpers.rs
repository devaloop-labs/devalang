use devalang_types::Value;
use std::collections::HashMap;

type OptMap = Option<HashMap<String, Value>>;

pub fn env_maps_from_automation(
    automation: &Option<HashMap<String, Value>>,
) -> (OptMap, OptMap, OptMap) {
    if let Some(auto) = automation {
        let vol = match auto.get("volume") {
            Some(Value::Map(m)) => Some(m.clone()),
            _ => None,
        };
        let pan = match auto.get("pan") {
            Some(Value::Map(m)) => Some(m.clone()),
            _ => None,
        };
        let pit = match auto.get("pitch") {
            Some(Value::Map(m)) => Some(m.clone()),
            _ => None,
        };
        (vol, pan, pit)
    } else {
        (None, None, None)
    }
}

pub fn eval_env_map(
    env_opt: &Option<HashMap<String, Value>>,
    progress: f32,
    default_val: f32,
) -> f32 {
    let env = match env_opt {
        Some(m) => m,
        None => {
            return default_val;
        }
    };
    let mut points: Vec<(f32, f32)> = Vec::with_capacity(env.len());
    for (k, v) in env.iter() {
        let key = if k.ends_with('%') {
            &k[..k.len() - 1]
        } else {
            &k[..]
        };
        if let Ok(mut p) = key.parse::<f32>() {
            p = (p / 100.0).clamp(0.0, 1.0);
            let val = match v {
                Value::Number(n) => *n,
                Value::String(s) => s.parse::<f32>().unwrap_or(default_val),
                Value::Identifier(s) => s.parse::<f32>().unwrap_or(default_val),
                _ => default_val,
            };
            points.push((p, val));
        }
    }
    if points.is_empty() {
        return default_val;
    }
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let t = progress.clamp(0.0, 1.0);
    if t <= points[0].0 {
        return points[0].1;
    }
    if t >= points[points.len() - 1].0 {
        return points[points.len() - 1].1;
    }
    for w in points.windows(2) {
        let (p0, v0) = w[0];
        let (p1, v1) = w[1];
        if t >= p0 && t <= p1 {
            let ratio = if (p1 - p0).abs() < f32::EPSILON {
                0.0
            } else {
                (t - p0) / (p1 - p0)
            };
            return v0 + (v1 - v0) * ratio;
        }
    }
    default_val
}

pub fn oscillator_sample(waveform: &str, current_freq: f32, t: f32) -> f32 {
    let phase = 2.0 * std::f32::consts::PI * current_freq * t;
    match waveform {
        "sine" => phase.sin(),
        "square" => {
            if phase.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        "saw" => 2.0 * (current_freq * t - (current_freq * t + 0.5).floor()),
        "triangle" => (2.0 * (2.0 * (current_freq * t).fract() - 1.0)).abs() * 2.0 - 1.0,
        _ => 0.0,
    }
}

pub fn adsr_envelope_value(
    i: usize,
    attack_samples: usize,
    decay_samples: usize,
    sustain_samples: usize,
    release_samples: usize,
    sustain_level: f32,
) -> f32 {
    let attack_start = 0usize;
    let decay_start = attack_samples;
    let sustain_start = attack_samples + decay_samples;
    let release_start = attack_samples + decay_samples + sustain_samples;

    if i < attack_start + attack_samples && attack_samples > 0 {
        let k = i - attack_start;
        let denom = if attack_samples > 1 { (attack_samples - 1) as f32 } else { 1.0 };
        (k as f32) / denom
    } else if i < decay_start + decay_samples && decay_samples > 0 {
        let k = i - decay_start;
        let denom = if decay_samples > 1 { (decay_samples - 1) as f32 } else { 1.0 };
        let ratio = (k as f32) / denom;
        1.0 - (1.0 - sustain_level) * ratio
    } else if i < sustain_start + sustain_samples {
        sustain_level
    } else if release_samples > 0 {
        // release: interpolate from sustain_level down to 0 inclusive
        let k = i.saturating_sub(release_start);
        let denom = if release_samples > 1 { (release_samples - 1) as f32 } else { 1.0 };
        let ratio = (k as f32) / denom;
        let val = sustain_level * (1.0 - ratio);
        if val < 0.0 { 0.0 } else { val }
    } else {
        0.0
    }
}

pub fn pan_gains(pan_val: f32) -> (f32, f32) {
    let left_gain = 1.0 - pan_val.max(0.0);
    let right_gain = 1.0 + pan_val.min(0.0);
    (left_gain, right_gain)
}

pub fn mix_stereo_samples_into_buffer(
    engine: &mut super::synth::AudioEngine,
    start_sample: usize,
    channels: usize,
    stereo_samples: &[i16],
) {
    let offset = start_sample * channels;
    let required_len = offset + stereo_samples.len();

    if engine.buffer.len() < required_len {
        engine.buffer.resize(required_len, 0);
    }

    for (i, sample) in stereo_samples.iter().enumerate() {
        engine.buffer[offset + i] = engine.buffer[offset + i].saturating_add(*sample);
    }
}
