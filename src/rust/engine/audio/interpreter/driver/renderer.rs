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
use std::collections::HashMap;

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

fn event_voice_window(
    event: &crate::engine::audio::events::AudioEvent,
) -> Option<(String, f32, f32)> {
    match event {
        crate::engine::audio::events::AudioEvent::Note {
            synth_id,
            start_time,
            duration,
            ..
        }
        | crate::engine::audio::events::AudioEvent::Chord {
            synth_id,
            start_time,
            duration,
            ..
        } => Some((synth_id.clone(), *start_time, *start_time + *duration)),
        _ => None,
    }
}

fn event_voice_limit(event: &crate::engine::audio::events::AudioEvent) -> Option<usize> {
    let value = match event {
        crate::engine::audio::events::AudioEvent::Note { synth_def, .. }
        | crate::engine::audio::events::AudioEvent::Chord { synth_def, .. } => {
            synth_def.options.get("voices").copied()
        }
        _ => None,
    }?;

    if value >= 1.0 {
        Some(value.round() as usize)
    } else {
        None
    }
}

pub(crate) fn renderable_event_windows(
    events: &[crate::engine::audio::events::AudioEvent],
) -> Vec<Option<f32>> {
    let mut audible_until: Vec<Option<f32>> = events
        .iter()
        .map(|event| {
            event_voice_window(event)
                .map(|(_, _, end)| end)
                .or(Some(f32::INFINITY))
        })
        .collect();
    let mut active_by_synth: HashMap<String, Vec<(f32, f32, usize)>> = HashMap::new();
    let mut ordered_events: Vec<(usize, String, f32, f32, usize)> = events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            let (synth_id, start, end) = event_voice_window(event)?;
            let limit = event_voice_limit(event)?;
            Some((idx, synth_id, start, end, limit))
        })
        .collect();

    ordered_events.sort_by(|a, b| {
        a.2.partial_cmp(&b.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    for (idx, synth_id, start, end, limit) in ordered_events {
        let active = active_by_synth.entry(synth_id).or_default();
        active.retain(|(_, active_end, active_idx)| {
            *active_end > start && audible_until[*active_idx].is_some()
        });
        active.push((start, end, idx));

        while active.len() > limit {
            if let Some((oldest_pos, _)) = active
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            {
                let (removed_start, _, removed_idx) = active.remove(oldest_pos);
                audible_until[removed_idx] = if removed_start < start {
                    Some(start)
                } else {
                    None
                };
            } else {
                break;
            }
        }
    }

    audible_until
}

pub(crate) fn renderable_event_mask(
    events: &[crate::engine::audio::events::AudioEvent],
) -> Vec<bool> {
    renderable_event_windows(events)
        .into_iter()
        .map(|window| window.is_some())
        .collect()
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
        && interpreter.audio_graph.node_names().len() > 1
    {
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
    let render_windows = renderable_event_windows(&interpreter.events.events);
    for (event_idx, event) in interpreter.events.events.iter().enumerate() {
        if render_windows[event_idx].is_none() {
            continue;
        }

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
                let render_duration = render_windows[event_idx]
                    .map(|end| (end - *start_time).min(*duration).max(0.0))
                    .unwrap_or(*duration);
                if render_duration <= 0.0 {
                    continue;
                }

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
                    bpm: synth_def.bpm,
                };

                if let Some(a) = attack {
                    params.attack = a / 1000.0;
                }
                if let Some(r) = release {
                    params.release = r / 1000.0;
                }

                // Handle decay("auto") mode - calculate decay based on note duration
                if let Some(&decay_mode) = params.options.get("decay_mode") {
                    if decay_mode > 0.5 {
                        // Check if decay_mode is set to "auto" (1.0)
                        // Calculate automatic decay: use most of duration minus attack and a small tail
                        let attack_ms = params.attack * 1000.0;
                        let tail_ms = 50.0; // Small tail at end
                        let available_ms = (render_duration * 1000.0).max(attack_ms + tail_ms);
                        let auto_decay = available_ms - attack_ms - tail_ms;
                        params.decay = (auto_decay / 1000.0).max(0.01); // Convert to seconds, minimum 0.01s
                    }
                }

                let mut samples = if *use_per_note_automation {
                    // Generate per-note automation with segments
                    if let Some(automation_ctx) =
                        interpreter.note_automation_templates.get(synth_id.as_str())
                    {
                        use crate::engine::audio::automation::evaluate_template_at;

                        let mut all_samples = Vec::new();
                        let num_segments = 8; // Generate 8 segments per note for smooth automation
                        let segment_duration = render_duration / num_segments as f32;

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
                            render_duration * 1000.0,
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
                        render_duration * 1000.0,
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
                ..
            } => {
                let render_duration = render_windows[event_idx]
                    .map(|end| (end - *start_time).min(*duration).max(0.0))
                    .unwrap_or(*duration);
                if render_duration <= 0.0 {
                    continue;
                }

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
                    bpm: synth_def.bpm,
                };
                if let Some(a) = attack {
                    params.attack = a / 1000.0;
                }
                if let Some(r) = release {
                    params.release = r / 1000.0;
                }

                let mut samples = generate_chord_with_options(
                    midis,
                    render_duration * 1000.0,
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
                effects: _effects,
                source: _,
                ..
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
                        if let Some(eff_val) = _effects {
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
            }
            crate::engine::audio::events::AudioEvent::Stop { target, time: _ } => {
                // Stop events: these are control events that signal to stop playback
                // They don't contribute to audio rendering directly
                // Target is optional: None = stop all, Some = stop specific entity
                // For now, we just log it. The actual stopping would be handled by the playback engine
                if let Some(entity) = target {
                    log_info!(logger, "Stop event for entity: {}", entity);
                } else {
                    log_info!(logger, "Stop all event");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::audio::events::{AudioEvent, SynthDefinition};

    fn note_event(start_time: f32, duration: f32, voices: Option<f32>) -> AudioEvent {
        let mut synth_def = SynthDefinition::default();
        if let Some(voices) = voices {
            synth_def.options.insert("voices".to_string(), voices);
        }

        AudioEvent::Note {
            node_id: Some("master".to_string()),
            midi: 60,
            start_time,
            duration,
            velocity: 0.8,
            synth_id: "lead".to_string(),
            synth_def,
            pan: 0.0,
            detune: 0.0,
            gain: 1.0,
            attack: None,
            release: None,
            delay_time: None,
            delay_feedback: None,
            delay_mix: None,
            reverb_amount: None,
            drive_amount: None,
            drive_color: None,
            effects: None,
            use_per_note_automation: false,
        }
    }

    #[test]
    fn test_voice_limit_clips_stolen_voice_at_overlap() {
        let events = vec![
            note_event(0.0, 1.0, Some(1.0)),
            note_event(0.25, 1.0, Some(1.0)),
        ];

        assert_eq!(
            renderable_event_windows(&events),
            vec![Some(0.25), Some(1.25)]
        );
        assert_eq!(renderable_event_mask(&events), vec![true, true]);
    }

    #[test]
    fn test_voice_limit_mutes_same_start_oldest_event() {
        let events = vec![
            note_event(0.0, 1.0, Some(1.0)),
            note_event(0.0, 1.0, Some(1.0)),
        ];

        assert_eq!(renderable_event_windows(&events), vec![None, Some(1.0)]);
    }

    #[test]
    fn test_voice_limit_keeps_two_overlapping_events_when_allowed() {
        let events = vec![
            note_event(0.0, 1.0, Some(2.0)),
            note_event(0.25, 1.0, Some(2.0)),
            note_event(0.5, 1.0, Some(2.0)),
        ];

        assert_eq!(
            renderable_event_windows(&events),
            vec![Some(0.5), Some(1.25), Some(1.5)]
        );
    }

    #[test]
    fn test_voice_limit_renders_stolen_voice_before_overlap() {
        let mut interpreter = AudioInterpreter::new(44_100);
        interpreter.events.events = vec![
            note_event(0.0, 1.0, Some(1.0)),
            note_event(0.25, 1.0, Some(1.0)),
        ];

        let samples = render_audio(&interpreter).expect("render should succeed");
        let before_overlap_end = (0.2 * interpreter.sample_rate as f32) as usize * 2;

        assert!(
            samples[..before_overlap_end]
                .iter()
                .any(|sample| sample.abs() > 0.0001),
            "stolen voices should still render before the newer note starts"
        );
    }
}
