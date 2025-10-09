use super::synth::types::{SynthType, get_synth_type};
use super::synth::{adsr_envelope, midi_to_frequency, oscillator_sample, time_to_samples};
/// Note generator - creates audio samples for synthesized notes
use anyhow::Result;
use std::collections::HashMap;

#[cfg(feature = "cli")]
use crate::engine::plugin::{loader::load_plugin, runner::WasmPluginRunner};

/// Filter definition
#[derive(Debug, Clone)]
pub struct FilterDef {
    pub filter_type: String, // lowpass, highpass, bandpass
    pub cutoff: f32,         // Hz
    pub resonance: f32,      // 0.0 - 1.0
}

/// Parameters for synthesizing a note
#[derive(Debug, Clone)]
pub struct SynthParams {
    pub waveform: String,
    pub attack: f32,                   // seconds
    pub decay: f32,                    // seconds
    pub sustain: f32,                  // level (0.0 - 1.0)
    pub release: f32,                  // seconds
    pub synth_type: Option<String>,    // pluck, arp, pad, bass, lead, keys
    pub filters: Vec<FilterDef>,       // Chain of filters
    pub options: HashMap<String, f32>, // Configurable options for synth types
    // Plugin support
    pub plugin_author: Option<String>,
    pub plugin_name: Option<String>,
    pub plugin_export: Option<String>,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: "sine".to_string(),
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            synth_type: None,
            filters: Vec::new(),
            options: HashMap::new(),
            plugin_author: None,
            plugin_name: None,
            plugin_export: None,
        }
    }
}

/// Generate stereo audio samples for a single note
pub fn generate_note(
    midi_note: u8,
    duration_ms: f32,
    velocity: f32,
    params: &SynthParams,
    sample_rate: u32,
) -> Result<Vec<f32>> {
    generate_note_with_options(
        midi_note,
        duration_ms,
        velocity,
        params,
        sample_rate,
        0.0,
        0.0,
    )
}

/// Generate stereo audio samples for a single note with pan and detune options
///
/// If params contains plugin information, it will use the WASM plugin instead of built-in synth
pub fn generate_note_with_options(
    midi_note: u8,
    duration_ms: f32,
    velocity: f32,
    params: &SynthParams,
    sample_rate: u32,
    pan: f32,    // -1.0 (left) to 1.0 (right), 0.0 = center
    detune: f32, // cents, -100 to 100
) -> Result<Vec<f32>> {
    // Generating note (diagnostics suppressed)

    // Check if we should use a WASM plugin
    #[cfg(feature = "cli")]
    {
        if let Some(ref author) = params.plugin_author {
            if let Some(ref name) = params.plugin_name {
                // Using plugin path
                return generate_note_with_plugin(
                    midi_note,
                    duration_ms,
                    velocity,
                    params,
                    sample_rate,
                    pan,
                    detune,
                    author,
                    name,
                    params.plugin_export.as_deref(),
                );
            }
        }
    }

    // Using classic synth path

    let base_frequency = midi_to_frequency(midi_note);

    // Apply detune (cents to frequency ratio: 2^(cents/1200))
    let frequency = if detune.abs() > 0.01 {
        base_frequency * 2.0_f32.powf(detune / 1200.0)
    } else {
        base_frequency
    };

    let velocity = velocity.clamp(0.0, 1.0);

    // Clone params to allow modification by synth type
    let mut modified_params = params.clone();

    // Apply synth type modifications
    let synth_type: Option<Box<dyn SynthType>> = if let Some(ref type_name) = params.synth_type {
        get_synth_type(type_name)
    } else {
        None
    };

    if let Some(ref stype) = synth_type {
        stype.modify_params(&mut modified_params);
    }

    // Convert ADSR times to samples
    let attack_samples = time_to_samples(modified_params.attack, sample_rate);
    let decay_samples = time_to_samples(modified_params.decay, sample_rate);
    let release_samples = time_to_samples(modified_params.release, sample_rate);

    // Calculate sustain duration
    let duration_seconds = duration_ms / 1000.0;
    let total_samples = time_to_samples(duration_seconds, sample_rate);
    let envelope_samples = attack_samples + decay_samples + release_samples;
    let sustain_samples = total_samples.saturating_sub(envelope_samples);

    let mut samples = Vec::with_capacity(total_samples * 2); // stereo

    // Calculate pan gains (constant power panning)
    let pan = pan.clamp(-1.0, 1.0);
    let pan_angle = (pan + 1.0) * 0.25 * std::f32::consts::PI; // 0 to PI/2
    let left_gain = pan_angle.cos();
    let right_gain = pan_angle.sin();

    for i in 0..total_samples {
        let time = i as f32 / sample_rate as f32;

        // Generate oscillator sample
        let osc_sample = oscillator_sample(&modified_params.waveform, frequency, time);

        // Apply ADSR envelope
        let envelope = adsr_envelope(
            i,
            attack_samples,
            decay_samples,
            sustain_samples,
            release_samples,
            modified_params.sustain,
        );

        // Apply velocity and envelope
        let amplitude = osc_sample * envelope * velocity * 0.3; // 0.3 for headroom

        // Stereo output with panning
        samples.push(amplitude * left_gain);
        samples.push(amplitude * right_gain);
    }

    // Apply filters if any
    for filter in &modified_params.filters {
        apply_filter(&mut samples, filter, sample_rate)?;
    }

    // Apply synth type post-processing
    if let Some(stype) = synth_type {
        stype.post_process(&mut samples, sample_rate, &modified_params.options)?;
    }

    Ok(samples)
}

/// Apply a filter to audio samples
fn apply_filter(samples: &mut [f32], filter: &FilterDef, sample_rate: u32) -> Result<()> {
    match filter.filter_type.to_lowercase().as_str() {
        "lowpass" => apply_lowpass(samples, filter.cutoff, sample_rate),
        "highpass" => apply_highpass(samples, filter.cutoff, sample_rate),
        "bandpass" => apply_bandpass(samples, filter.cutoff, sample_rate),
        _ => Ok(()),
    }
}

/// Simple one-pole lowpass filter
fn apply_lowpass(samples: &mut [f32], cutoff: f32, sample_rate: u32) -> Result<()> {
    let dt = 1.0 / sample_rate as f32;
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    let alpha = dt / (rc + dt);

    let mut prev = 0.0f32;
    for i in (0..samples.len()).step_by(2) {
        // Process left channel
        let filtered = prev + alpha * (samples[i] - prev);
        prev = filtered;
        samples[i] = filtered;

        // Copy to right channel
        if i + 1 < samples.len() {
            samples[i + 1] = filtered;
        }
    }

    Ok(())
}

/// Simple one-pole highpass filter
fn apply_highpass(samples: &mut [f32], cutoff: f32, sample_rate: u32) -> Result<()> {
    let dt = 1.0 / sample_rate as f32;
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    let alpha = rc / (rc + dt);

    let mut prev_input = 0.0f32;
    let mut prev_output = 0.0f32;

    for i in (0..samples.len()).step_by(2) {
        let current = samples[i];
        let filtered = alpha * (prev_output + current - prev_input);

        prev_input = current;
        prev_output = filtered;

        samples[i] = filtered;
        if i + 1 < samples.len() {
            samples[i + 1] = filtered;
        }
    }

    Ok(())
}

/// Simple bandpass filter (combination of lowpass and highpass)
fn apply_bandpass(samples: &mut [f32], center: f32, sample_rate: u32) -> Result<()> {
    // Bandpass = highpass below center, then lowpass above center
    let bandwidth = center * 0.5; // 50% bandwidth

    apply_highpass(samples, center - bandwidth, sample_rate)?;
    apply_lowpass(samples, center + bandwidth, sample_rate)?;

    Ok(())
}

/// Generate stereo audio samples for a chord (multiple notes)
pub fn generate_chord(
    midi_notes: &[u8],
    duration_ms: f32,
    velocity: f32,
    params: &SynthParams,
    sample_rate: u32,
) -> Result<Vec<f32>> {
    generate_chord_with_options(
        midi_notes,
        duration_ms,
        velocity,
        params,
        sample_rate,
        0.0,
        0.0,
        0.0,
    )
}

/// Generate stereo audio samples for a chord with pan, detune and spread options
pub fn generate_chord_with_options(
    midi_notes: &[u8],
    duration_ms: f32,
    velocity: f32,
    params: &SynthParams,
    sample_rate: u32,
    pan: f32,    // -1.0 (left) to 1.0 (right), 0.0 = center
    detune: f32, // cents, -100 to 100
    spread: f32, // stereo spread 0.0-1.0 for chord notes
) -> Result<Vec<f32>> {
    if midi_notes.is_empty() {
        return Ok(Vec::new());
    }

    let num_notes = midi_notes.len();
    let spread = spread.clamp(0.0, 1.0);

    // Calculate pan position for each note if spread is enabled
    let mut result: Option<Vec<f32>> = None;

    for (i, &midi_note) in midi_notes.iter().enumerate() {
        // Calculate individual pan for each note based on spread
        let note_pan = if num_notes > 1 && spread > 0.0 {
            // Distribute notes across stereo field
            let position = i as f32 / (num_notes - 1) as f32; // 0.0 to 1.0
            let spread_amount = (position - 0.5) * 2.0 * spread; // -spread to +spread
            (pan + spread_amount).clamp(-1.0, 1.0)
        } else {
            pan
        };

        // Generate note with individual pan
        let note_samples = generate_note_with_options(
            midi_note,
            duration_ms,
            velocity,
            params,
            sample_rate,
            note_pan,
            detune,
        )?;

        // Mix notes together
        match result {
            None => {
                result = Some(note_samples);
            }
            Some(ref mut buffer) => {
                // Mix by averaging (to avoid clipping)
                for (j, sample) in note_samples.iter().enumerate() {
                    if j < buffer.len() {
                        buffer[j] = (buffer[j] + sample) / 2.0;
                    }
                }
            }
        }
    }

    Ok(result.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_note() {
        let params = SynthParams::default();
        let samples = generate_note(60, 500.0, 0.8, &params, 44100).unwrap();

        // Should have samples (stereo)
        assert!(samples.len() > 0);
        assert_eq!(samples.len() % 2, 0); // Must be even (stereo)

        // Check that some samples are non-zero
        let has_audio = samples.iter().any(|&s| s.abs() > 0.001);
        assert!(has_audio);
    }

    #[test]
    fn test_generate_chord() {
        let params = SynthParams::default();
        let samples = generate_chord(&[60, 64, 67], 500.0, 0.8, &params, 44100).unwrap();

        // Should have samples
        assert!(samples.len() > 0);

        // Check that some samples are non-zero
        let has_audio = samples.iter().any(|&s| s.abs() > 0.001);
        assert!(has_audio);
    }
}

/// Generate audio using a WASM plugin
#[cfg(feature = "cli")]
fn generate_note_with_plugin(
    midi_note: u8,
    duration_ms: f32,
    velocity: f32,
    params: &SynthParams,
    sample_rate: u32,
    pan: f32,
    detune: f32,
    plugin_author: &str,
    plugin_name: &str,
    plugin_export: Option<&str>,
) -> Result<Vec<f32>> {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    println!(
        "🎸 [PLUGIN_GEN] Generating note with plugin: {}.{} (export: {:?})",
        plugin_author, plugin_name, plugin_export
    );

    // Global plugin runner (cached)
    static PLUGIN_RUNNER: Lazy<Mutex<WasmPluginRunner>> =
        Lazy::new(|| Mutex::new(WasmPluginRunner::new()));

    // Global plugin cache
    static PLUGIN_CACHE: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Get or load plugin
    let plugin_key = format!("{}.{}", plugin_author, plugin_name);
    let mut cache = PLUGIN_CACHE.lock().unwrap();

    let wasm_bytes = if let Some(bytes) = cache.get(&plugin_key) {
        println!("   Using cached plugin ({}  bytes)", bytes.len());
        bytes.clone()
    } else {
        println!("   Loading plugin from disk...");
        // Load plugin
        let (info, bytes) = load_plugin(plugin_author, plugin_name)
            .map_err(|e| anyhow::anyhow!("Failed to load plugin: {}", e))?;

        eprintln!(
            "✅ Loaded plugin: {}.{} (v{})",
            info.author,
            info.name,
            info.version.as_deref().unwrap_or("unknown")
        );

        cache.insert(plugin_key.clone(), bytes.clone());
        bytes
    };
    drop(cache);

    // Calculate buffer size
    let base_frequency = midi_to_frequency(midi_note);
    let frequency = if detune.abs() > 0.01 {
        base_frequency * 2.0_f32.powf(detune / 1200.0)
    } else {
        base_frequency
    };

    let duration_seconds = duration_ms / 1000.0;
    let total_samples = (duration_seconds * sample_rate as f32) as usize;
    let mut buffer = vec![0.0f32; total_samples * 2]; // stereo

    // Prepare options for plugin (merge waveform + custom options)
    let mut plugin_options = params.options.clone();

    // Add waveform if not already in options
    if !plugin_options.contains_key("waveform") {
        // Try to convert waveform string to numeric value if possible
        let waveform_value = match params.waveform.as_str() {
            "sine" => 0.0,
            "saw" => 1.0,
            "square" => 2.0,
            "triangle" => 3.0,
            _ => 0.0, // default to sine
        };
        plugin_options.insert("waveform".to_string(), waveform_value);
    }

    // Call plugin
    let runner = PLUGIN_RUNNER.lock().unwrap();
    let synth_id = format!("{}_{}", plugin_key, plugin_export.unwrap_or("default"));

    println!(
        "   📋 Buffer before plugin: first 10 samples: {:?}",
        &buffer[0..10.min(buffer.len())]
    );

    runner
        .render_note_in_place(
            &wasm_bytes,
            &mut buffer,
            Some(&synth_id),
            plugin_export,
            frequency,
            velocity,
            duration_ms as i32,
            sample_rate as i32,
            2, // stereo
            Some(&plugin_options),
        )
        .map_err(|e| anyhow::anyhow!("Plugin render error: {}", e))?;

    println!(
        "   📋 Buffer after plugin: first 10 samples: {:?}",
        &buffer[0..10.min(buffer.len())]
    );
    println!(
        "   📋 Buffer stats: len={}, max={:.4}, rms={:.4}",
        buffer.len(),
        buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max),
        (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt()
    );

    // Apply panning if needed
    if pan.abs() > 0.01 {
        let pan = pan.clamp(-1.0, 1.0);
        let pan_angle = (pan + 1.0) * 0.25 * std::f32::consts::PI;
        let left_gain = pan_angle.cos();
        let right_gain = pan_angle.sin();

        for i in (0..buffer.len()).step_by(2) {
            if i + 1 < buffer.len() {
                buffer[i] *= left_gain;
                buffer[i + 1] *= right_gain;
            }
        }
    }

    // Apply filters if any
    for filter in &params.filters {
        apply_filter(&mut buffer, filter, sample_rate)?;
    }

    Ok(buffer)
}
