use super::AudioInterpreter;
use crate::engine::audio::effects::processors::{
    DelayProcessor, DriveProcessor, EffectProcessor, ReverbProcessor,
};
use crate::engine::audio::generator::{
    SynthParams, generate_chord_with_options, generate_note_with_options,
};
use anyhow::Result;

// Conditional logging macros for CLI feature
#[cfg(feature = "cli")]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_info {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

#[cfg(feature = "cli")]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_warn {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

#[cfg(feature = "cli")]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_error {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

pub fn render_audio(interpreter: &AudioInterpreter) -> Result<Vec<f32>> {
    let total_duration = interpreter.events.total_duration();
    if total_duration <= 0.0 {
        return Ok(Vec::new());
    }

    let total_samples = (total_duration * interpreter.sample_rate as f32).ceil() as usize;
    let mut buffer = vec![0.0f32; total_samples * 2]; // stereo

    #[cfg(feature = "cli")]
    let logger = crate::tools::logger::Logger::new();
    #[cfg(not(feature = "cli"))]
    let _logger = ();
    log_info!(
        logger,
        "Starting audio rendering: {} events, {} synths, duration {:.2}s",
        interpreter.events.events.len(),
        interpreter.events.synths.len(),
        total_duration
    );

    for event in &interpreter.events.events {
        match event {
            crate::engine::audio::events::AudioEvent::Note { .. } => {}
            _ => {}
        }
    }

    // Render each event (copied logic from driver)
    let mut note_count = 0;
    let mut sample_count = 0;
    for event in &interpreter.events.events {
        match event {
            crate::engine::audio::events::AudioEvent::Note {
                midi,
                start_time,
                duration,
                velocity,
                synth_def,
                pan,
                detune,
                gain,
                attack,
                release,
                delay_time,
                delay_feedback,
                delay_mix,
                reverb_amount,
                drive_amount,
                drive_color,
                ..
            } => {
                note_count += 1;
                // Log note rendering only if needed (debug mode)

                let mut params = SynthParams {
                    waveform: synth_def.waveform.clone(),
                    attack: synth_def.attack,
                    decay: synth_def.decay,
                    sustain: synth_def.sustain,
                    release: synth_def.release,
                    synth_type: synth_def.synth_type.clone(),
                    filters: synth_def.filters.clone(),
                    options: synth_def.options.clone(),
                    plugin_author: synth_def.plugin_author.clone(),
                    plugin_name: synth_def.plugin_name.clone(),
                    plugin_export: synth_def.plugin_export.clone(),
                };

                if let Some(a) = attack {
                    params.attack = a / 1000.0;
                }
                if let Some(r) = release {
                    params.release = r / 1000.0;
                }

                let mut samples = generate_note_with_options(
                    *midi,
                    duration * 1000.0,
                    velocity * gain,
                    &params,
                    interpreter.sample_rate,
                    *pan,
                    *detune,
                )?;

                if let Some(amount) = drive_amount {
                    let color = drive_color.unwrap_or(0.5);
                    let mix = 0.7;
                    let mut processor = DriveProcessor::new(*amount, color, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }
                if let Some(amount) = reverb_amount {
                    let room_size = *amount;
                    let damping = 0.5;
                    let mix = *amount * 0.5;
                    let mut processor = ReverbProcessor::new(room_size, damping, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }
                if let Some(time) = delay_time {
                    let feedback = delay_feedback.unwrap_or(0.3);
                    let mix = delay_mix.unwrap_or(0.5);
                    let mut processor = DelayProcessor::new(*time, feedback, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }

                let start_sample = (*start_time * interpreter.sample_rate as f32) as usize * 2;
                for (i, &sample) in samples.iter().enumerate() {
                    let buf_idx = start_sample + i;
                    if buf_idx < buffer.len() {
                        buffer[buf_idx] += sample;
                    }
                }
            }

            crate::engine::audio::events::AudioEvent::Chord {
                midis,
                start_time,
                duration,
                velocity,
                synth_id: _,
                synth_def,
                pan,
                detune,
                spread,
                gain,
                attack,
                release,
                delay_time,
                delay_feedback,
                delay_mix,
                reverb_amount,
                drive_amount,
                drive_color,
            } => {
                let mut params = SynthParams {
                    waveform: synth_def.waveform.clone(),
                    attack: synth_def.attack,
                    decay: synth_def.decay,
                    sustain: synth_def.sustain,
                    release: synth_def.release,
                    synth_type: synth_def.synth_type.clone(),
                    filters: synth_def.filters.clone(),
                    options: synth_def.options.clone(),
                    plugin_author: synth_def.plugin_author.clone(),
                    plugin_name: synth_def.plugin_name.clone(),
                    plugin_export: synth_def.plugin_export.clone(),
                };
                if let Some(a) = attack {
                    params.attack = a / 1000.0;
                }
                if let Some(r) = release {
                    params.release = r / 1000.0;
                }

                let mut samples = generate_chord_with_options(
                    midis,
                    duration * 1000.0,
                    velocity * gain,
                    &params,
                    interpreter.sample_rate,
                    *pan,
                    *detune,
                    *spread,
                )?;

                if let Some(amount) = drive_amount {
                    let color = drive_color.unwrap_or(0.5);
                    let mix = 0.7;
                    let mut processor = DriveProcessor::new(*amount, color, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }
                if let Some(amount) = reverb_amount {
                    let room_size = *amount;
                    let damping = 0.5;
                    let mix = *amount * 0.5;
                    let mut processor = ReverbProcessor::new(room_size, damping, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }
                if let Some(time) = delay_time {
                    let feedback = delay_feedback.unwrap_or(0.3);
                    let mix = delay_mix.unwrap_or(0.5);
                    let mut processor = DelayProcessor::new(*time, feedback, mix);
                    processor.process(&mut samples, interpreter.sample_rate);
                }

                let start_sample = (*start_time * interpreter.sample_rate as f32) as usize * 2;
                for (i, &sample) in samples.iter().enumerate() {
                    let buf_idx = start_sample + i;
                    if buf_idx < buffer.len() {
                        buffer[buf_idx] += sample;
                    }
                }
            }

            crate::engine::audio::events::AudioEvent::Sample {
                uri,
                start_time,
                velocity,
            } => {
                sample_count += 1;
                // Log sample rendering only if needed (debug mode)

                // WASM path: samples are provided by the web registry as i16 PCM
                #[cfg(feature = "wasm")]
                {
                    use crate::web::registry::samples::get_sample;
                    if let Some(pcm_data) = get_sample(uri) {
                        let start_sample_idx =
                            (*start_time * interpreter.sample_rate as f32) as usize;
                        for (i, &pcm_value) in pcm_data.iter().enumerate() {
                            let sample = (pcm_value as f32 / 32768.0) * velocity;
                            let stereo_pos = (start_sample_idx + i) * 2;
                            let buf_idx_l = stereo_pos;
                            let buf_idx_r = stereo_pos + 1;
                            if buf_idx_l < buffer.len() {
                                buffer[buf_idx_l] += sample;
                            }
                            if buf_idx_r < buffer.len() {
                                buffer[buf_idx_r] += sample;
                            }
                        }
                    } else {
                        log_warn!(logger, "Sample not found in registry: {}", uri);
                    }
                }

                // CLI/native path: use SampleData (mono f32) and resample/scale into stereo buffer
                #[cfg(feature = "cli")]
                {
                    use crate::engine::audio::samples;
                    if let Some(sample_data) = samples::get_sample(uri) {
                        let start_sample_idx =
                            (*start_time * interpreter.sample_rate as f32) as usize;
                        // velocity is in 0.0..1.0 range for sample events
                        let velocity_scale = velocity;
                        let resample_ratio =
                            interpreter.sample_rate as f32 / sample_data.sample_rate as f32;
                        for (i, &sample) in sample_data.samples.iter().enumerate() {
                            let output_idx =
                                start_sample_idx + (i as f32 * resample_ratio) as usize;
                            let stereo_pos = output_idx * 2;
                            let buf_idx_l = stereo_pos;
                            let buf_idx_r = stereo_pos + 1;
                            let scaled_sample = sample * velocity_scale;
                            if buf_idx_l < buffer.len() {
                                buffer[buf_idx_l] += scaled_sample;
                            }
                            if buf_idx_r < buffer.len() {
                                buffer[buf_idx_r] += scaled_sample;
                            }
                        }
                    } else {
                        log_error!(logger, "Bank sample not found: {}", uri);
                    }
                }
            }
        }
    }

    log_info!(
        logger,
        "Rendered {} notes + {} samples",
        note_count,
        sample_count
    );
    let max_amplitude = buffer.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    log_info!(
        logger,
        "Max amplitude before normalization: {:.4}",
        max_amplitude
    );

    if max_amplitude > 1.0 {
        for sample in buffer.iter_mut() {
            *sample /= max_amplitude;
        }
    }

    Ok(buffer)
}

pub fn render_audio_wrapper(interpreter: &mut AudioInterpreter) -> Result<Vec<f32>> {
    render_audio(interpreter)
}
