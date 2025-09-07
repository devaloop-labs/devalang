use devalang_types::Value;
use std::collections::HashMap;

pub fn pad_samples_impl(
    engine: &mut crate::core::audio::engine::driver::AudioEngine,
    samples: &[i16],
    time_secs: f32,
    effects_map: Option<HashMap<String, Value>>,
) {
    let sample_rate = engine.sample_rate as f32;
    let channels = engine.channels as usize;

    let offset = (time_secs * (sample_rate) * (channels as f32)) as usize;
    let total_samples = samples.len();

    let mut gain = 1.0;
    let mut pan = 0.0;
    let mut fade_in = 0.0;
    let mut fade_out = 0.0;
    let mut pitch = 1.0;
    let mut drive = 0.0;
    let mut reverb = 0.0;
    let mut delay = 0.0; // delay time in seconds
    let delay_feedback = 0.35; // default feedback

    if let Some(map) = &effects_map {
        for (key, val) in map {
            match (key.as_str(), val) {
                ("gain", Value::Number(v)) => {
                    gain = *v;
                }
                ("pan", Value::Number(v)) => {
                    pan = *v;
                }
                ("fadeIn", Value::Number(v)) => {
                    fade_in = *v;
                }
                ("fadeOut", Value::Number(v)) => {
                    fade_out = *v;
                }
                ("pitch", Value::Number(v)) => {
                    pitch = *v;
                }
                ("drive", Value::Number(v)) => {
                    drive = *v;
                }
                ("reverb", Value::Number(v)) => {
                    reverb = *v;
                }
                ("delay", Value::Number(v)) => {
                    delay = *v;
                }
                _ => eprintln!("⚠️ Unknown or invalid effect '{}'", key),
            }
        }
    }

    let fade_in_samples = (fade_in * (sample_rate)) as usize;
    let fade_out_samples = (fade_out * (sample_rate)) as usize;

    // If no fade specified, apply a tiny default fade (2 ms) when sample boundaries are non-zero
    let default_boundary_fade_ms = 1.0_f32; // 1 ms
    let default_fade_samples = (default_boundary_fade_ms * (sample_rate)) as usize;
    let mut effective_fade_in = fade_in_samples;
    let mut effective_fade_out = fade_out_samples;
    if effective_fade_in == 0 {
        if let Some(&first) = samples.first() {
            if first.abs() > 64 {
                // increased threshold to detect only strong abrupt starts
                effective_fade_in = default_fade_samples.max(1);
            }
        }
    }
    if effective_fade_out == 0 {
        if let Some(&last) = samples.last() {
            if last.abs() > 64 {
                // increased threshold to detect only strong abrupt ends
                effective_fade_out = default_fade_samples.max(1);
            }
        }
    }

    // Ensure fades do not exceed half the sample length to avoid silencing short samples
    if total_samples > 0 {
        let cap = total_samples / 2;
        if effective_fade_in > cap {
            effective_fade_in = cap.max(1);
        }
        if effective_fade_out > cap {
            effective_fade_out = cap.max(1);
        }
    }

    let delay_samples = if delay > 0.0 {
        (delay * (sample_rate)) as usize
    } else {
        0
    };
    let mut delay_buffer: Vec<f32> = vec![0.0; total_samples + delay_samples];

    for i in 0..total_samples {
        let pitch_index = if pitch != 1.0 {
            ((i as f32) / pitch) as usize
        } else {
            i
        };

        let mut adjusted = if pitch_index < total_samples {
            samples[pitch_index] as f32
        } else {
            0.0
        };

        adjusted *= gain;

        if effective_fade_in > 0 && i < effective_fade_in {
            if effective_fade_in == 1 {
                adjusted *= 0.0;
            } else {
                adjusted *= (i as f32) / (effective_fade_in as f32);
            }
        }
        if effective_fade_out > 0 && i >= total_samples.saturating_sub(effective_fade_out) {
            if effective_fade_out == 1 {
                adjusted *= 0.0;
            } else {
                adjusted *= ((total_samples - 1 - i) as f32) / ((effective_fade_out - 1) as f32);
            }
        }

        if drive > 0.0 {
            let normalized = adjusted / (i16::MAX as f32);
            let pre_gain = (10f32).powf(drive / 20.0);
            let driven = (normalized * pre_gain).tanh();
            adjusted = driven * (i16::MAX as f32);
        }

        if delay_samples > 0 && i >= delay_samples {
            let echo = delay_buffer[i - delay_samples] * delay_feedback;
            adjusted += echo;
        }
        if delay_samples > 0 {
            delay_buffer[i] = adjusted;
        }

        if reverb > 0.0 {
            let reverb_delay = (0.03 * (sample_rate)) as usize;
            if i >= reverb_delay {
                adjusted += (engine.buffer[offset + i - reverb_delay] as f32) * reverb;
            }
        }

        let adjusted_sample = adjusted.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        let (left_gain, right_gain) = crate::core::audio::engine::helpers::pan_gains(pan);

        let left = ((adjusted_sample as f32) * left_gain) as i16;
        let right = ((adjusted_sample as f32) * right_gain) as i16;

        // For interleaved buffer with `channels` channels, each frame has `channels` samples.
        // left channel at frame i is at offset + i * channels, right at +1.
        let left_pos = offset + i * channels;
        let right_pos = left_pos + 1;

        if right_pos < engine.buffer.len() {
            engine.buffer[left_pos] = engine.buffer[left_pos].saturating_add(left);
            engine.buffer[right_pos] = engine.buffer[right_pos].saturating_add(right);
        }
    }
}
