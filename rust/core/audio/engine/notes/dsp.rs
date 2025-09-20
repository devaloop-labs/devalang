use crate::core::audio::engine::notes::params::NoteSetup;
use devalang_types::Value;
use std::collections::HashMap;

pub fn render_notes_into_buffer(
    engine: &mut crate::core::audio::engine::AudioEngine,
    _waveform: &str,
    _freq: f32,
    _amp: f32,
    _start_time_ms: f32,
    _duration_ms: f32,
    _synth_params: HashMap<String, Value>,
    _note_params: HashMap<String, Value>,
    _automation: Option<HashMap<String, Value>>,
    setup: NoteSetup,
) -> Vec<(usize, usize)> {
    use crate::core::audio::engine::helpers;

    let sample_rate = setup.sample_rate;
    let channels = setup.channels;
    let total_samples = setup.total_samples;
    let start_sample = setup.start_sample;

    let mut stereo_samples: Vec<i16> = Vec::with_capacity(total_samples * channels);
    let fade_len = (sample_rate * 0.01) as usize; // 10ms fade

    for i in 0..total_samples {
        let t = ((start_sample + i) as f32) / sample_rate;

        // simplified voice/unison and oscillator sampling
        let mut value = helpers::oscillator_sample(_waveform, _freq, t);

        // apply ADSR envelope
        let envelope = helpers::adsr_envelope_value(
            i,
            setup.attack_samples,
            setup.decay_samples,
            if total_samples > setup.attack_samples + setup.decay_samples + setup.release_samples {
                total_samples - setup.attack_samples - setup.decay_samples - setup.release_samples
            } else {
                0
            },
            setup.release_samples,
            setup.sustain_level,
        );

        value *= envelope * (i16::MAX as f32) * _amp;

        if fade_len > 0 && i < fade_len {
            if fade_len == 1 {
                value *= 0.0;
            } else {
                value *= (i as f32) / (fade_len as f32);
            }
        } else if fade_len > 0 && i >= total_samples.saturating_sub(fade_len) {
            if fade_len == 1 {
                value *= 0.0;
            } else {
                value *= ((total_samples - 1 - i) as f32) / ((fade_len - 1) as f32);
            }
        }

        let (left_gain, right_gain) = helpers::pan_gains(0.0);
        let left = (value * left_gain)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        let right = (value * right_gain)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        // push samples interleaved for channels=2, fallback zeros for other channel counts
        if channels >= 2 {
            stereo_samples.push(left);
            stereo_samples.push(right);
        } else if channels == 1 {
            stereo_samples.push(((left as i32 + right as i32) / 2) as i16);
        } else {
            stereo_samples.push(left);
            stereo_samples.push(right);
        }
    }

    engine.note_count = engine.note_count.saturating_add(1);
    helpers::mix_stereo_samples_into_buffer(engine, start_sample, channels, &stereo_samples);

    // Return the inserted sample range for this note (start_sample, total_samples)
    vec![(start_sample, stereo_samples.len())]
}
