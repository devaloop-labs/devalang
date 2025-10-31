use crate::engine::audio::events::AudioEvent;
use crate::language::syntax::ast::Value;
/// MIDI file loading and parsing
use anyhow::{Result, anyhow};
use std::path::Path;

#[cfg(any(feature = "cli", feature = "wasm"))]
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
};

#[cfg(all(target_arch = "wasm32", not(any(feature = "cli", feature = "wasm"))))]
use crate::midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
};

#[cfg(feature = "cli")]
use std::collections::HashMap;

/// Load a MIDI file and return a Value::Map representing the MIDI data
#[cfg(feature = "cli")]
pub fn load_midi_file(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path)
        .map_err(|e| anyhow!("Failed to read MIDI file {}: {}", path.display(), e))?;

    let smf = Smf::parse(&bytes)
        .map_err(|e| anyhow!("Failed to parse MIDI file {}: {}", path.display(), e))?;

    // Convert MIDI data to a map
    let mut midi_map = HashMap::new();

    // Defaults
    let mut bpm = 120.0f32;
    let mut tempo_us_per_quarter: u32 = 500_000; // default 120 BPM
    let mut notes: Vec<Value> = Vec::new();

    // Determine ticks per beat from header timing
    let ticks_per_beat: u32 = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        _ => 480u32,
    };

    // Active note-on map to pair note-offs: key = (track, channel, key) -> Vec of indices in notes
    let mut active: std::collections::HashMap<(usize, u8, u8), Vec<usize>> =
        std::collections::HashMap::new();

    // Process all tracks
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut current_ticks: u32 = 0;

        for event in track {
            current_ticks = current_ticks.wrapping_add(event.delta.as_int() as u32);

            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    let chan = channel.as_int();
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                // compute time in ms from ticks using current tempo
                                let time_ms = (current_ticks as f32)
                                    * (tempo_us_per_quarter as f32)
                                    / (ticks_per_beat as f32)
                                    / 1000.0;

                                let mut note_map = HashMap::new();
                                note_map.insert(
                                    "tick".to_string(),
                                    Value::Number(current_ticks as f32),
                                );
                                note_map.insert("time".to_string(), Value::Number(time_ms));
                                // store beat position (useful to rescale when interpreter BPM changes)
                                let beats = time_ms * (bpm as f32) / 60000.0; // time_ms / (60000/midi_bpm)
                                note_map.insert("beat".to_string(), Value::Number(beats));
                                note_map
                                    .insert("note".to_string(), Value::Number(key.as_int() as f32));
                                note_map.insert(
                                    "velocity".to_string(),
                                    Value::Number(vel.as_int() as f32),
                                );
                                note_map
                                    .insert("track".to_string(), Value::Number(track_idx as f32));
                                note_map.insert("channel".to_string(), Value::Number(chan as f32));

                                notes.push(Value::Map(note_map));

                                // record active index for pairing
                                let idx = notes.len() - 1;
                                active
                                    .entry((track_idx, chan as u8, key.as_int() as u8))
                                    .or_default()
                                    .push(idx);
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            // pair with most recent active note-on for same track/channel/key
                            let key_tuple = (track_idx, channel.as_int() as u8, key.as_int() as u8);
                            if let Some(vec_idxs) = active.get_mut(&key_tuple) {
                                if let Some(on_idx) = vec_idxs.pop() {
                                    // compute duration from ticks
                                    // find onset tick stored in notes[on_idx]
                                    if let Some(Value::Map(on_map)) = notes.get_mut(on_idx) {
                                        if let Some(Value::Number(on_tick)) = on_map.get("tick") {
                                            let onset_ticks = *on_tick as u32;
                                            let dur_ticks =
                                                current_ticks.saturating_sub(onset_ticks);
                                            let duration_ms = (dur_ticks as f32)
                                                * (tempo_us_per_quarter as f32)
                                                / (ticks_per_beat as f32)
                                                / 1000.0;
                                            on_map.insert(
                                                "duration".to_string(),
                                                Value::Number(duration_ms),
                                            );
                                            // also store duration in beats for easier rescaling
                                            let duration_beats =
                                                duration_ms * (bpm as f32) / 60000.0;
                                            on_map.insert(
                                                "duration_beats".to_string(),
                                                Value::Number(duration_beats),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Meta(meta) => {
                    // Extract tempo if present
                    if let MetaMessage::Tempo(t) = meta {
                        tempo_us_per_quarter = t.as_int();
                        bpm = 60_000_000.0f32 / tempo_us_per_quarter as f32;
                    }
                }
                _ => {}
            }
        }
    }

    // For any lingering active notes without note-off, set a default duration (500 ms)
    for (_key, vec_idxs) in active.iter() {
        for &idx in vec_idxs.iter() {
            if let Some(Value::Map(m)) = notes.get_mut(idx) {
                if !m.contains_key("duration") {
                    m.insert("duration".to_string(), Value::Number(500.0));
                    // default duration beats using current bpm
                    let default_beats = 500.0 * (bpm as f32) / 60000.0;
                    m.insert("duration_beats".to_string(), Value::Number(default_beats));
                }
            }
        }
    }

    // Store in map
    midi_map.insert("bpm".to_string(), Value::Number(bpm));
    midi_map.insert(
        "ticks_per_beat".to_string(),
        Value::Number(ticks_per_beat as f32),
    );
    midi_map.insert("notes".to_string(), Value::Array(notes));
    midi_map.insert("type".to_string(), Value::String("midi".to_string()));

    Ok(Value::Map(midi_map))
}

#[cfg(not(feature = "cli"))]
pub fn load_midi_file(_path: &Path) -> Result<Value> {
    Err(anyhow!("MIDI loading not available without 'cli' feature"))
}

// ============================================================================
// MIDI EXPORT
// ============================================================================

/// Export AudioEvents to MIDI bytes (for WASM)
pub fn events_to_midi_bytes(events: &[AudioEvent], bpm: f32) -> Result<Vec<u8>> {
    if events.is_empty() {
        return Err(anyhow!("No events to export"));
    }

    // Create MIDI header (single track format)
    let ticks_per_beat = 480; // Standard MIDI resolution
    let header = Header::new(Format::SingleTrack, Timing::Metrical(ticks_per_beat.into()));

    // Create track events list
    let mut track_events = Vec::new();

    // Add tempo meta event at start
    let tempo_us_per_quarter = (60_000_000.0 / bpm) as u32;
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(tempo_us_per_quarter.into())),
    });

    // Collect all note events (expand chords to individual notes)
    let mut midi_notes = Vec::new();

    for event in events {
        match event {
            AudioEvent::Note {
                midi,
                start_time,
                duration,
                velocity,
                ..
            } => {
                midi_notes.push(MidiNote {
                    note: *midi,
                    start: *start_time,
                    duration: *duration,
                    velocity: *velocity,
                });
            }
            AudioEvent::Chord {
                midis,
                start_time,
                duration,
                velocity,
                ..
            } => {
                // Expand chord into individual notes
                for &note in midis {
                    midi_notes.push(MidiNote {
                        note,
                        start: *start_time,
                        duration: *duration,
                        velocity: *velocity,
                    });
                }
            }
            AudioEvent::Sample { .. } => {
                // Samples are not exported to MIDI
            } // Log messages are stored separately in AudioEventList.logs and are not part of AudioEvent
        }
    }

    // Sort by start time
    midi_notes.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    // Convert events to MIDI messages with proper delta timing
    let mut midi_messages: Vec<MidiEventTimed> = Vec::new();

    for note in &midi_notes {
        let start_ticks = time_to_ticks(note.start, bpm, ticks_per_beat);
        let end_ticks = time_to_ticks(note.start + note.duration, bpm, ticks_per_beat);

        // Note On
        midi_messages.push(MidiEventTimed {
            ticks: start_ticks,
            message: MidiMessage::NoteOn {
                key: note.note.into(),
                vel: ((note.velocity * 127.0) as u8).into(),
            },
        });

        // Note Off
        midi_messages.push(MidiEventTimed {
            ticks: end_ticks,
            message: MidiMessage::NoteOff {
                key: note.note.into(),
                vel: 0.into(),
            },
        });
    }

    // Sort all messages by time
    midi_messages.sort_by_key(|msg| msg.ticks);

    // Convert to delta times and create TrackEvents
    let mut last_ticks = 0u32;

    for msg in midi_messages {
        let delta = msg.ticks.saturating_sub(last_ticks);

        track_events.push(TrackEvent {
            delta: delta.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: msg.message,
            },
        });

        last_ticks = msg.ticks;
    }

    // End of track marker
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    // Create SMF and write to memory buffer
    let track = Track::from(track_events);
    let mut smf = Smf::new(header);
    smf.tracks.push(track);

    // Write to memory buffer
    let mut buffer = Vec::new();
    smf.write(&mut buffer)
        .map_err(|e| anyhow!("Failed to write MIDI bytes: {}", e))?;

    Ok(buffer)
}

/// Export AudioEvents to a standard MIDI file
#[cfg(feature = "cli")]
pub fn export_midi_file(events: &[AudioEvent], output_path: &Path, bpm: f32) -> Result<()> {
    if events.is_empty() {
        return Err(anyhow!("No events to export"));
    }

    // Create MIDI header (single track format)
    let ticks_per_beat = 480; // Standard MIDI resolution
    let header = Header::new(Format::SingleTrack, Timing::Metrical(ticks_per_beat.into()));

    // Create track events list
    let mut track_events = Vec::new();

    // Add tempo meta event at start
    let tempo_us_per_quarter = (60_000_000.0 / bpm) as u32;
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(tempo_us_per_quarter.into())),
    });

    // Collect all note events (expand chords to individual notes)
    let mut midi_notes = Vec::new();

    for event in events {
        match event {
            AudioEvent::Note {
                midi,
                start_time,
                duration,
                velocity,
                ..
            } => {
                midi_notes.push(MidiNote {
                    note: *midi,
                    start: *start_time,
                    duration: *duration,
                    velocity: *velocity,
                });
            }
            AudioEvent::Chord {
                midis,
                start_time,
                duration,
                velocity,
                ..
            } => {
                // Expand chord into individual notes
                for &note in midis {
                    midi_notes.push(MidiNote {
                        note,
                        start: *start_time,
                        duration: *duration,
                        velocity: *velocity,
                    });
                }
            }
            AudioEvent::Sample { .. } => {
                // Samples are not exported to MIDI
            } // Note: Log messages are stored separately in AudioEventList.logs and are
              // not part of the AudioEvent enum; they are ignored for MIDI export.
        }
    }

    // Sort by start time
    midi_notes.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    // Convert events to MIDI messages with proper delta timing
    let mut midi_messages: Vec<MidiEventTimed> = Vec::new();

    for note in &midi_notes {
        let start_ticks = time_to_ticks(note.start, bpm, ticks_per_beat);
        let end_ticks = time_to_ticks(note.start + note.duration, bpm, ticks_per_beat);

        // Note On
        midi_messages.push(MidiEventTimed {
            ticks: start_ticks,
            message: MidiMessage::NoteOn {
                key: note.note.into(),
                vel: ((note.velocity * 127.0) as u8).into(),
            },
        });

        // Note Off
        midi_messages.push(MidiEventTimed {
            ticks: end_ticks,
            message: MidiMessage::NoteOff {
                key: note.note.into(),
                vel: 0.into(),
            },
        });
    }

    // Sort all messages by time
    midi_messages.sort_by_key(|msg| msg.ticks);

    // Convert to delta times and create TrackEvents
    let mut last_ticks = 0u32;

    for msg in midi_messages {
        let delta = msg.ticks.saturating_sub(last_ticks);

        track_events.push(TrackEvent {
            delta: delta.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: msg.message,
            },
        });

        last_ticks = msg.ticks;
    }

    // End of track marker
    track_events.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    // Create SMF and write to file
    let track = Track::from(track_events);
    let mut smf = Smf::new(header);
    smf.tracks.push(track);

    // Write to file directly (midly 0.5 API)
    smf.save(output_path)
        .map_err(|e| anyhow!("Failed to write MIDI file {}: {}", output_path.display(), e))?;

    println!(
        "✅ MIDI exported: {} ({} events in, {} notes written)",
        output_path.display(),
        events.len(),
        midi_notes.len()
    );
    Ok(())
}

#[cfg(not(feature = "cli"))]
pub fn export_midi_file(_events: &[AudioEvent], _output_path: &Path, _bpm: f32) -> Result<()> {
    Err(anyhow!("MIDI export not available without 'cli' feature"))
}

// Helper structures for MIDI export
#[derive(Debug, Clone)]
struct MidiNote {
    note: u8,
    start: f32,
    duration: f32,
    velocity: f32,
}

#[derive(Debug, Clone)]
struct MidiEventTimed {
    ticks: u32,
    message: MidiMessage,
}

/// Convert time in seconds to MIDI ticks
fn time_to_ticks(time_seconds: f32, bpm: f32, ticks_per_beat: u16) -> u32 {
    let beats = time_seconds * (bpm / 60.0);
    (beats * ticks_per_beat as f32) as u32
}
