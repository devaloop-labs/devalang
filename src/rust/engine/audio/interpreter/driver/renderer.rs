#![allow(unused_macros)]
use super::AudioInterpreter;
use crate::engine::audio::effects::chain::{EffectChain, build_effect_chain};
use crate::engine::audio::effects::normalize_effects;
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

    #[cfg(feature = "cli")]
    let logger = crate::tools::logger::Logger::new();
    #[cfg(not(feature = "cli"))]
    let _logger = ();

    // Check if we should use audio graph rendering (when routing is configured)
    if !interpreter.audio_graph.node_names().is_empty() 
        && interpreter.audio_graph.node_names().len() > 1 {
        log_info!(
            logger,
            "Using audio graph rendering with {} nodes",
            interpreter.audio_graph.node_names().len()
        );
        return super::renderer_graph::render_audio_graph(interpreter, total_samples)
            .map_err(|e| anyhow::anyhow!("Audio graph rendering failed: {}", e));
    }

    // Default: simple buffer rendering (no routing)
    let mut buffer = vec![0.0f32; total_samples * 2]; // stereo

    log_info!(
        logger,
        "Starting audio rendering: {} events, {} synths, duration {:.2}s",
        interpreter.events.events.len(),
        interpreter.events.synths.len(),
        total_duration
    );

    // No-op pre-scan: logs are stored separately in interpreter.events.logs and are ignored by renderer.

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
                synth_id,
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
                use_per_note_automation,
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
                    lfo: synth_def.lfo.clone(),
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

                let mut samples = if *use_per_note_automation {
                    // Generate per-note automation with segments
                    if let Some(automation_ctx) =
                        interpreter.note_automation_templates.get(synth_id.as_str())
                    {
                        use crate::engine::audio::automation::evaluate_template_at;

                        let mut all_samples = Vec::new();
                        let num_segments = 8; // Generate 8 segments per note for smooth automation
                        let segment_duration = duration / num_segments as f32;

                        for segment_idx in 0..num_segments {
                            // Calculate progress for this segment (0.0 to 1.0)
                            let segment_progress = (segment_idx as f32 + 0.5) / num_segments as f32;

                            // Evaluate templates for this progress point
                            let segment_pan = automation_ctx
                                .templates
                                .iter()
                                .find(|t| t.param_name == "pan")
                                .map(|t| evaluate_template_at(t, segment_progress))
                                .unwrap_or(*pan);

                            let segment_detune = automation_ctx
                                .templates
                                .iter()
                                .find(|t| t.param_name == "pitch" || t.param_name == "detune")
                                .map(|t| evaluate_template_at(t, segment_progress))
                                .unwrap_or(*detune);

                            let segment_gain = automation_ctx
                                .templates
                                .iter()
                                .find(|t| t.param_name == "volume" || t.param_name == "gain")
                                .map(|t| evaluate_template_at(t, segment_progress))
                                .unwrap_or(*gain);

                            // Clone and modify params for this segment
                            let mut segment_params = params.clone();

                            // Apply cutoff automation
                            for filter in &mut segment_params.filters {
                                let segment_cutoff = automation_ctx
                                    .templates
                                    .iter()
                                    .find(|t| t.param_name == "cutoff")
                                    .map(|t| evaluate_template_at(t, segment_progress))
                                    .unwrap_or(filter.cutoff);
                                filter.cutoff = segment_cutoff;

                                let segment_resonance = automation_ctx
                                    .templates
                                    .iter()
                                    .find(|t| t.param_name == "resonance")
                                    .map(|t| evaluate_template_at(t, segment_progress))
                                    .unwrap_or(filter.resonance);
                                filter.resonance = segment_resonance;
                            }

                            // Generate this segment with updated params
                            let segment_samples = generate_note_with_options(
                                *midi,
                                segment_duration * 1000.0,
                                (velocity * segment_gain).clamp(0.0, 1.0),
                                &segment_params,
                                interpreter.sample_rate,
                                segment_pan,
                                segment_detune,
                            )?;

                            all_samples.extend(segment_samples);
                        }

                        all_samples
                    } else {
                        // No templates found, generate normally
                        generate_note_with_options(
                            *midi,
                            duration * 1000.0,
                            velocity * gain,
                            &params,
                            interpreter.sample_rate,
                            *pan,
                            *detune,
                        )?
                    }
                } else {
                    // Generate note normally (global mode or no automation)
                    generate_note_with_options(
                        *midi,
                        duration * 1000.0,
                        velocity * gain,
                        &params,
                        interpreter.sample_rate,
                        *pan,
                        *detune,
                    )?
                };

                // If this event has an effects map/array, build an effect chain and apply it.
                // Also avoid double-applying drive/reverb/delay when those keys appear in the effects map.
                let mut skip_drive = false;
                let mut skip_reverb = false;
                let mut skip_delay = false;
                let mut effect_chain: Option<EffectChain> = None;

                // Try to extract per-event effects (if present) and build a chain
                if let crate::engine::audio::events::AudioEvent::Note { effects, .. } = event {
                    if let Some(eff_val) = effects {
                        match eff_val {
                            crate::language::syntax::ast::Value::Array(arr) => {
                                let chain = build_effect_chain(arr, true);
                                if !chain.is_empty() {
                                    effect_chain = Some(chain);
                                }
                            }
                            crate::language::syntax::ast::Value::Map(_) => {
                                let normalized = normalize_effects(&Some(eff_val.clone()));
                                if !normalized.is_empty() {
                                    let mut chain = EffectChain::new(true);
                                    for (k, v) in normalized.into_iter() {
                                        chain.add_effect(
                                            &k,
                                            Some(crate::language::syntax::ast::Value::Map(v)),
                                        );
                                        // mark skips for known audio processors
                                        match k.as_str() {
                                            "drive" => skip_drive = true,
                                            "reverb" => skip_reverb = true,
                                            "delay" => skip_delay = true,
                                            _ => {}
                                        }
                                    }
                                    effect_chain = Some(chain);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(chain) = effect_chain.as_mut() {
                    chain.process(&mut samples, interpreter.sample_rate);
                }

                // Apply legacy per-field processors only when not overridden by the effects map
                if let Some(amount) = drive_amount {
                    if !skip_drive {
                        let color = drive_color.unwrap_or(0.5);
                        let mix = 0.7;
                        // tone default to 0.5, color passed from drive_color
                        let mut processor = DriveProcessor::new(*amount, 0.5, color, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
                }
                if let Some(amount) = reverb_amount {
                    if !skip_reverb {
                        let room_size = *amount;
                        let damping = 0.5;
                        let decay = 0.5;
                        let mix = *amount * 0.5;
                        let mut processor = ReverbProcessor::new(room_size, damping, decay, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
                }
                if let Some(time) = delay_time {
                    if !skip_delay {
                        let feedback = delay_feedback.unwrap_or(0.3);
                        let mix = delay_mix.unwrap_or(0.5);
                        let mut processor = DelayProcessor::new(*time, feedback, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
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
                effects,
                use_per_note_automation: _,
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
                    lfo: synth_def.lfo.clone(),
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

                // Build and apply effect chain if per-event effects are present
                let mut skip_drive = false;
                let mut skip_reverb = false;
                let mut skip_delay = false;
                let mut effect_chain: Option<EffectChain> = None;
                if let Some(eff_val) = effects {
                    match eff_val {
                        crate::language::syntax::ast::Value::Array(arr) => {
                            let chain = build_effect_chain(arr, true);
                            if !chain.is_empty() {
                                effect_chain = Some(chain);
                            }
                        }
                        crate::language::syntax::ast::Value::Map(_) => {
                            let normalized = normalize_effects(&Some(eff_val.clone()));
                            if !normalized.is_empty() {
                                let mut chain = EffectChain::new(true);
                                for (k, v) in normalized.into_iter() {
                                    chain.add_effect(
                                        &k,
                                        Some(crate::language::syntax::ast::Value::Map(v)),
                                    );
                                    match k.as_str() {
                                        "drive" => skip_drive = true,
                                        "reverb" => skip_reverb = true,
                                        "delay" => skip_delay = true,
                                        _ => {}
                                    }
                                }
                                effect_chain = Some(chain);
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(chain) = effect_chain.as_mut() {
                    chain.process(&mut samples, interpreter.sample_rate);
                }

                if let Some(amount) = drive_amount {
                    if !skip_drive {
                        let color = drive_color.unwrap_or(0.5);
                        let mix = 0.7;
                        let mut processor = DriveProcessor::new(*amount, 0.5, color, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
                }
                if let Some(amount) = reverb_amount {
                    if !skip_reverb {
                        let room_size = *amount;
                        let damping = 0.5;
                        let decay = 0.5;
                        let mix = *amount * 0.5;
                        let mut processor = ReverbProcessor::new(room_size, damping, decay, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
                }
                if let Some(time) = delay_time {
                    if !skip_delay {
                        let feedback = delay_feedback.unwrap_or(0.3);
                        let mix = delay_mix.unwrap_or(0.5);
                        let mut processor = DelayProcessor::new(*time, feedback, mix);
                        processor.process(&mut samples, interpreter.sample_rate);
                    }
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
                effects,
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

                        // Make a mutable copy so we can run effects on it (effects expect f32 slices)
                        let mut proc_samples = sample_data.samples.clone();

                        // Build and apply effect chain for sample events (trigger context)
                        let mut sample_chain: Option<EffectChain> = None;
                        if let Some(eff_val) = effects {
                            match eff_val {
                                crate::language::syntax::ast::Value::Array(arr) => {
                                    let chain = build_effect_chain(arr, false);
                                    if !chain.is_empty() {
                                        sample_chain = Some(chain);
                                    }
                                }
                                crate::language::syntax::ast::Value::Map(_) => {
                                    let normalized = normalize_effects(&Some(eff_val.clone()));
                                    if !normalized.is_empty() {
                                        let mut chain = EffectChain::new(false);
                                        for (k, v) in normalized.into_iter() {
                                            chain.add_effect(
                                                &k,
                                                Some(crate::language::syntax::ast::Value::Map(v)),
                                            );
                                        }
                                        sample_chain = Some(chain);
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let Some(chain) = sample_chain.as_mut() {
                            chain.process(&mut proc_samples, interpreter.sample_rate);
                        }

                        for (i, &sample) in proc_samples.iter().enumerate() {
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
            } // Log events are stored separately and ignored by renderer
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
