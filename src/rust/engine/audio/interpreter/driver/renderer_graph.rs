/// Audio graph rendering - implements proper routing, node effects, and ducking
use super::AudioInterpreter;
use crate::engine::audio::interpreter::audio_graph::Connection;
use std::collections::HashMap;

/// Buffers for each node in the audio graph (stereo: left + right samples interleaved)
type NodeBuffers = HashMap<String, Vec<f32>>;

/// Process audio through the routing graph
pub fn render_audio_graph(
    interpreter: &AudioInterpreter,
    total_samples: usize,
) -> anyhow::Result<Vec<f32>> {
    let total_duration = total_samples as f32 / interpreter.sample_rate as f32;

    // Create buffers for each node in the graph
    let mut node_buffers: NodeBuffers = HashMap::new();
    for node_name in interpreter.audio_graph.node_names() {
        node_buffers.insert(node_name, vec![0.0f32; total_samples * 2]);
    }

    // Phase 1: Render audio events into their respective nodes
    render_events_into_nodes(interpreter, &mut node_buffers, total_duration)?;

    // Phase 2: Apply effects to each node
    apply_node_effects(interpreter, &mut node_buffers)?;

    // Phase 3: Apply ducks and route audio between nodes
    apply_routing_and_ducking(interpreter, &mut node_buffers)?;

    // Phase 4: Mix all nodes into master buffer
    let master_buffer = mix_to_master(interpreter, &node_buffers)?;

    Ok(master_buffer)
}

/// Determine which node an event belongs to based on its content
/// Returns the node name where this event should be rendered
fn get_event_target_node(
    event: &crate::engine::audio::events::AudioEvent,
    interpreter: &AudioInterpreter,
) -> String {
    use crate::engine::audio::events::AudioEvent;

    match event {
        AudioEvent::Note { synth_id, .. } | AudioEvent::Chord { synth_id, .. } => {
            // Check if there's a routing node matching the synth ID
            if let Some(_) = interpreter.routing.nodes.get(synth_id) {
                synth_id.clone()
            } else {
                // Auto-route to $master if not explicitly defined in routing
                "$master".to_string()
            }
        }
        AudioEvent::Sample { source, .. } => {
            // First, check if source (pattern name) matches a routing node
            if let Some(src) = source {
                if interpreter.routing.nodes.contains_key(src) {
                    return src.clone();
                }
            }

            // Auto-route to $master if pattern not explicitly defined in routing
            "$master".to_string()
        }
        AudioEvent::Stop { .. } => {
            // Stop events don't route to nodes
            "$master".to_string()
        }
    }
}

/// Render audio events into their assigned nodes
fn render_events_into_nodes(
    interpreter: &AudioInterpreter,
    node_buffers: &mut NodeBuffers,
    total_duration: f32,
) -> anyhow::Result<()> {
    use crate::engine::audio::events::AudioEvent;
    use crate::engine::audio::generator::{SynthParams, generate_note_with_options};

    let total_samples = (total_duration * interpreter.sample_rate as f32).ceil() as usize;

    for event in &interpreter.events.events {
        // Determine target node for this event
        let mut target_node = get_event_target_node(event, interpreter);

        // If target node doesn't exist in buffers and it's not $master, route to $master instead
        if !node_buffers.contains_key(&target_node) && target_node != "$master" {
            target_node = "$master".to_string();
        }

        // Ensure the target buffer exists (create if needed)
        if !node_buffers.contains_key(&target_node) {
            node_buffers.insert(target_node.clone(), vec![0.0f32; total_samples * 2]);
        }

        // Get the target buffer
        let target_buffer = node_buffers.get_mut(&target_node);
        if target_buffer.is_none() {
            continue;
        }
        let target_buffer = target_buffer.unwrap();

        match event {
            AudioEvent::Note {
                midi,
                start_time,
                duration,
                synth_def,
                pan,
                detune,
                gain,
                velocity,
                attack,
                release,
                ..
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

                let samples = generate_note_with_options(
                    *midi,
                    *duration * 1000.0, // Convert to milliseconds
                    velocity * gain,    // Combined velocity and gain
                    &params,
                    interpreter.sample_rate,
                    *pan,
                    *detune,
                )?;

                let start_sample = (*start_time * interpreter.sample_rate as f32).ceil() as usize;
                let start_idx = start_sample * 2; // Convert to sample index (stereo)

                // Only write if start is within bounds
                if start_idx < total_samples * 2 {
                    let end_idx = (start_idx + samples.len()).min(total_samples * 2);
                    let write_len = end_idx.saturating_sub(start_idx);

                    if write_len > 0 {
                        target_buffer[start_idx..end_idx]
                            .iter_mut()
                            .zip(samples[0..write_len].iter())
                            .for_each(|(dst, src)| *dst += src);
                    }
                }
            }
            AudioEvent::Sample {
                uri: _uri,
                start_time: _start_time,
                velocity: _velocity,
                ..
            } => {
                // Load sample from bank (synthetic drums for CLI)
                #[cfg(feature = "cli")]
                {
                    use crate::engine::audio::samples;

                    if let Some(sample_data) = samples::get_sample(_uri) {
                        let start_sample =
                            (*_start_time * interpreter.sample_rate as f32).ceil() as usize;
                        let start_idx = start_sample * 2; // Convert to stereo sample index

                        // Only write if sample starts before buffer ends
                        if start_idx < total_samples * 2 {
                            let end_idx =
                                (start_idx + sample_data.samples.len()).min(total_samples * 2);
                            let write_len = end_idx - start_idx;

                            if write_len > 0 {
                                // Scale sample with velocity
                                let velocity_scale = _velocity;
                                target_buffer[start_idx..end_idx]
                                    .iter_mut()
                                    .zip(sample_data.samples[0..write_len].iter())
                                    .for_each(|(dst, src)| *dst += src * velocity_scale);
                            }
                        }
                    }
                }
            }
            _ => {
                // Chord and Stop events are not rendered directly in this graph rendering mode
            }
        }
    }

    Ok(())
}

/// Apply effects chains to each node
fn apply_node_effects(
    interpreter: &AudioInterpreter,
    node_buffers: &mut NodeBuffers,
) -> anyhow::Result<()> {
    use crate::engine::audio::effects::chain::build_effect_chain;

    for (node_name, node_config) in &interpreter.audio_graph.nodes {
        if let Some(effects_value) = &node_config.effects {
            // Build effect chain - need to convert single Value to array
            let effects_array = match effects_value {
                crate::language::syntax::ast::Value::Array(arr) => arr.clone(),
                _ => vec![effects_value.clone()],
            };

            let mut effect_chain = build_effect_chain(&effects_array, false);

            if let Some(buffer) = node_buffers.get_mut(node_name) {
                // Apply effects to this node's buffer
                effect_chain.process(buffer, interpreter.sample_rate);
            }
        }
    }

    Ok(())
}

/// Apply routing connections and duck effects
fn apply_routing_and_ducking(
    interpreter: &AudioInterpreter,
    node_buffers: &mut NodeBuffers,
) -> anyhow::Result<()> {
    // Phase 1: Apply all ducks and sidechains first (these modify source buffers)
    for connection in interpreter.audio_graph.connections.iter() {
        match connection {
            Connection::Duck {
                source,
                destination,
                effect_params: _,
            } => {
                apply_duck(source, destination, node_buffers, interpreter.sample_rate)?;
            }
            Connection::Sidechain {
                source,
                destination,
                effect_params: _,
            } => {
                apply_sidechain(source, destination, node_buffers, interpreter.sample_rate)?;
            }
            _ => {}
        }
    }

    // Phase 2: Apply all routes (these mix audio between nodes)
    for connection in interpreter.audio_graph.connections.iter() {
        match connection {
            Connection::Route {
                source,
                destination,
                gain,
            } => {
                // Mix source buffer into destination buffer with gain
                if let (Some(src_buf), Some(dst_buf)) = (
                    node_buffers.get(source).cloned(),
                    node_buffers.get_mut(destination),
                ) {
                    for j in 0..src_buf.len() {
                        dst_buf[j] += src_buf[j] * gain;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Apply duck effect - compress destination based on source envelope
fn apply_duck(
    source_name: &str,
    destination_name: &str,
    node_buffers: &mut NodeBuffers,
    sample_rate: u32,
) -> anyhow::Result<()> {
    // Get source buffer (the trigger - kick) and destination buffer (the target - bassline)
    let source_buf = if let Some(buf) = node_buffers.get(source_name) {
        buf.clone()
    } else {
        return Ok(());
    };

    let dest_opt = node_buffers.get_mut(destination_name);
    if dest_opt.is_none() {
        return Ok(());
    }

    let dest_buf = dest_opt.unwrap();

    // Simple duck: when source has significant amplitude, reduce destination
    // Use RMS (root mean square) to detect activity in source
    let window_size = (sample_rate as usize / 100) * 2; // 10ms windows in stereo samples

    for frame_start in (0..dest_buf.len()).step_by(window_size.max(2)) {
        let frame_end = (frame_start + window_size).min(dest_buf.len());

        // Calculate RMS of source in this window
        let source_rms = if frame_start < source_buf.len() && frame_end <= source_buf.len() {
            let sum_sq: f32 = source_buf[frame_start..frame_end]
                .iter()
                .map(|&s| s * s)
                .sum();
            (sum_sq / (frame_end - frame_start) as f32).sqrt()
        } else {
            0.0
        };

        // Apply duck: if source is active (RMS > threshold), reduce destination
        let threshold = 0.01; // Amplitude threshold for detecting kick
        let duck_reduction = if source_rms > threshold {
            // Smooth reduction: 1.0 (no reduction) when silent, 0.3 (70% reduction) when active
            0.3 + (1.0 - 0.3) * (-source_rms * 100.0).exp()
        } else {
            1.0 // No reduction when source is silent
        };

        // Apply the reduction to destination buffer in this window
        for i in frame_start..frame_end {
            dest_buf[i] *= duck_reduction;
        }
    }

    Ok(())
}

/// Apply sidechain effect - gate modulation between nodes
fn apply_sidechain(
    source_name: &str,
    destination_name: &str,
    node_buffers: &mut NodeBuffers,
    sample_rate: u32,
) -> anyhow::Result<()> {
    // Get current volumes in destination buffer (envelope)
    let dest_envelope = if let Some(dest_buf) = node_buffers.get(destination_name) {
        compute_envelope(dest_buf, sample_rate)
    } else {
        return Ok(());
    };

    // Apply sidechain modulation based on destination envelope
    if let Some(src_buf) = node_buffers.get_mut(source_name) {
        // Map buffer indices to envelope indices
        let frame_rate = 100;
        let samples_per_frame = (sample_rate / frame_rate) as usize * 2;

        for frame_idx in (0..src_buf.len()).step_by(2) {
            let current_envelope_idx = frame_idx / samples_per_frame;

            if current_envelope_idx < dest_envelope.len() {
                let dest_level = dest_envelope[current_envelope_idx];

                // Sidechain gate: proportional to destination level
                let normalized_linear = (dest_level * 10.0).min(1.0);
                let gate_open = 1.0 - (normalized_linear * 0.5); // Range [1.0, 0.5]

                src_buf[frame_idx] *= gate_open;
                if frame_idx + 1 < src_buf.len() {
                    src_buf[frame_idx + 1] *= gate_open;
                }
            }
        }
    }

    Ok(())
}

/// Compute RMS envelope of a buffer (stereo, 2 samples per frame)
fn compute_envelope(buffer: &[f32], sample_rate: u32) -> Vec<f32> {
    let frame_rate = 100; // 100 Hz envelope resolution
    let samples_per_frame = sample_rate / frame_rate;
    let mut envelope = Vec::new();

    for chunk in buffer.chunks(samples_per_frame as usize * 2) {
        let rms: f32 = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        envelope.push(rms.min(1.0).max(0.0));
    }

    envelope
}

/// Mix all node buffers down to master
fn mix_to_master(
    _interpreter: &AudioInterpreter,
    node_buffers: &NodeBuffers,
) -> anyhow::Result<Vec<f32>> {
    let master_buf = node_buffers
        .get("$master")
        .ok_or_else(|| anyhow::anyhow!("Master node not found"))?
        .clone();

    let mut result = master_buf;

    // Mix all other nodes into master (except master itself)
    for (node_name, buffer) in node_buffers {
        if node_name != "$master" {
            for i in 0..buffer.len() {
                result[i] += buffer[i];
            }
        }
    }

    Ok(result)
}
