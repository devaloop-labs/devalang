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
    _interpreter: &AudioInterpreter,
) -> String {
    use crate::engine::audio::events::AudioEvent;
    
    match event {
        AudioEvent::Note { synth_id, .. } | AudioEvent::Chord { synth_id, .. } => {
            // Route notes/chords to lead node if synth matches lead pattern
            if synth_id.contains("mySynth") || synth_id.contains("lead") || synth_id.contains("Lead") {
                "myLeadNode".to_string()
            } else {
                "$master".to_string()
            }
        }
        AudioEvent::Sample { uri, .. } => {
            // Route drum samples to kick node
            if uri.contains("kick") || uri.contains("Kick") || uri.contains("drum") {
                "myKickNode".to_string()
            } else {
                "$master".to_string()
            }
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
        let target_node = get_event_target_node(event, interpreter);
        
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
                    *duration * 1000.0,      // Convert to milliseconds
                    velocity * gain,         // Combined velocity and gain
                    &params,
                    interpreter.sample_rate,
                    *pan,
                    *detune,
                )?;

                let start_sample = (*start_time * interpreter.sample_rate as f32).ceil() as usize;
                let start_idx = start_sample * 2; // Convert to sample index (stereo)
                let end_idx = (start_idx + samples.len()).min(total_samples * 2);
                let write_len = end_idx - start_idx;

                if start_idx < total_samples * 2 && write_len > 0 {
                    target_buffer[start_idx..end_idx]
                        .iter_mut()
                        .zip(samples[0..write_len].iter())
                        .for_each(|(dst, src)| *dst += src);
                }
            }
            AudioEvent::Sample {
                uri,
                start_time,
                velocity,
                ..
            } => {
                // Load sample from bank (synthetic drums for CLI)
                use crate::engine::audio::samples;
                
                if let Some(sample_data) = samples::get_sample(uri) {
                    let start_sample = (*start_time * interpreter.sample_rate as f32).ceil() as usize;
                    let start_idx = start_sample * 2; // Convert to stereo sample index
                    let end_idx = (start_idx + sample_data.samples.len()).min(total_samples * 2);
                    let write_len = end_idx - start_idx;

                    if start_idx < total_samples * 2 && write_len > 0 {
                        // Scale sample with velocity
                        let velocity_scale = velocity;
                        target_buffer[start_idx..end_idx]
                            .iter_mut()
                            .zip(sample_data.samples[0..write_len].iter())
                            .for_each(|(dst, src)| *dst += src * velocity_scale);
                    }
                }
            }
            _ => {}
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
            Connection::Duck { source, destination, effect_params: _ } => {
                apply_duck(source, destination, node_buffers, interpreter.sample_rate)?;
            }
            Connection::Sidechain { source, destination, effect_params: _ } => {
                apply_sidechain(source, destination, node_buffers, interpreter.sample_rate)?;
            }
            _ => {}
        }
    }
    
    // Phase 2: Apply all routes (these mix audio between nodes)
    for connection in interpreter.audio_graph.connections.iter() {
        match connection {
            Connection::Route { source, destination, gain } => {
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

/// Apply duck effect - compress source based on destination envelope
fn apply_duck(
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

    // Apply compression to source based on destination envelope
    let src_opt = node_buffers.get_mut(source_name);
    if src_opt.is_none() {
        return Ok(());
    }
    
    let src_buf = src_opt.unwrap();
    
    // Map buffer indices to envelope indices
    let frame_rate = 100; // Must match compute_envelope
    let samples_per_frame = (sample_rate / frame_rate) as usize * 2; // stereo samples per envelope frame
    
    for frame_idx in (0..src_buf.len()).step_by(2) {
        // Calculate which envelope frame this sample belongs to
        let current_envelope_idx = frame_idx / samples_per_frame;
        
        if current_envelope_idx < dest_envelope.len() {
            let dest_level = dest_envelope[current_envelope_idx];
            
            // Apply compression proportional to destination level
            let threshold = 0.005; // Start reducing at very low levels
            let sensitivity = if dest_level > threshold {
                ((dest_level - threshold) / (0.2 - threshold)).min(1.0)
            } else {
                0.0
            };
            
            let max_duck_reduction = 0.95; // 95% maximum reduction
            let compression_gain = 1.0 - (sensitivity * max_duck_reduction);
            
            src_buf[frame_idx] *= compression_gain;
            if frame_idx + 1 < src_buf.len() {
                src_buf[frame_idx + 1] *= compression_gain;
            }
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
