use crate::engine::audio::generator::FilterDef;
/// Audio events system - stores note/chord events to be rendered
use crate::language::syntax::ast::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AudioEvent {
    Note {
        midi: u8,
        start_time: f32,
        duration: f32,
        velocity: f32,
        synth_id: String,
        synth_def: SynthDefinition, // Snapshot of synth at event creation time
        // Audio options
        pan: f32,             // -1.0 to 1.0
        detune: f32,          // cents
        gain: f32,            // multiplier
        attack: Option<f32>,  // ms override
        release: Option<f32>, // ms override
        // Effects
        delay_time: Option<f32>,     // ms
        delay_feedback: Option<f32>, // 0.0-1.0
        delay_mix: Option<f32>,      // 0.0-1.0
        reverb_amount: Option<f32>,  // 0.0-1.0
        drive_amount: Option<f32>,   // 0.0-1.0
        drive_color: Option<f32>,    // 0.0-1.0
    },
    Chord {
        midis: Vec<u8>,
        start_time: f32,
        duration: f32,
        velocity: f32,
        synth_id: String,
        synth_def: SynthDefinition, // Snapshot of synth at event creation time
        // Audio options
        pan: f32,             // -1.0 to 1.0
        detune: f32,          // cents
        spread: f32,          // 0.0 to 1.0
        gain: f32,            // multiplier
        attack: Option<f32>,  // ms override
        release: Option<f32>, // ms override
        // Effects
        delay_time: Option<f32>,     // ms
        delay_feedback: Option<f32>, // 0.0-1.0
        delay_mix: Option<f32>,      // 0.0-1.0
        reverb_amount: Option<f32>,  // 0.0-1.0
        drive_amount: Option<f32>,   // 0.0-1.0
        drive_color: Option<f32>,    // 0.0-1.0
    },
    Sample {
        uri: String,
        start_time: f32,
        velocity: f32,
    },
}

/// Audio events collector
#[derive(Debug, Default)]
pub struct AudioEventList {
    pub events: Vec<AudioEvent>,
    pub synths: HashMap<String, SynthDefinition>,
}

#[derive(Debug, Clone)]
pub struct SynthDefinition {
    pub waveform: String,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub synth_type: Option<String>,
    pub filters: Vec<FilterDef>,
    pub options: HashMap<String, f32>, // Configurable synth type options
    // Plugin support
    pub plugin_author: Option<String>,
    pub plugin_name: Option<String>,
    pub plugin_export: Option<String>,
}

impl Default for SynthDefinition {
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

impl AudioEventList {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            synths: HashMap::new(),
        }
    }

    pub fn add_synth(&mut self, name: String, definition: SynthDefinition) {
        self.synths.insert(name, definition);
    }

    pub fn add_note_event(
        &mut self,
        synth_id: &str,
        midi: u8,
        start_time: f32,
        duration: f32,
        velocity: f32,
        pan: f32,
        detune: f32,
        gain: f32,
        attack: Option<f32>,
        release: Option<f32>,
        delay_time: Option<f32>,
        delay_feedback: Option<f32>,
        delay_mix: Option<f32>,
        reverb_amount: Option<f32>,
        drive_amount: Option<f32>,
        drive_color: Option<f32>,
    ) {
        // Capture synth definition snapshot at event creation time
        let synth_def = self.get_synth(synth_id).cloned().unwrap_or_else(|| {
            println!("⚠️  Warning: Synth '{}' not found when creating note event. Available synths: {:?}", 
                     synth_id, self.synths.keys().collect::<Vec<_>>());
            SynthDefinition::default()
        });

        self.events.push(AudioEvent::Note {
            midi,
            start_time,
            duration,
            velocity,
            synth_id: synth_id.to_string(),
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
        });
    }

    pub fn add_chord_event(
        &mut self,
        synth_id: &str,
        midis: Vec<u8>,
        start_time: f32,
        duration: f32,
        velocity: f32,
        pan: f32,
        detune: f32,
        spread: f32,
        gain: f32,
        attack: Option<f32>,
        release: Option<f32>,
        delay_time: Option<f32>,
        delay_feedback: Option<f32>,
        delay_mix: Option<f32>,
        reverb_amount: Option<f32>,
        drive_amount: Option<f32>,
        drive_color: Option<f32>,
    ) {
        // Capture synth definition snapshot at event creation time
        let synth_def = self.get_synth(synth_id).cloned().unwrap_or_default();

        self.events.push(AudioEvent::Chord {
            midis,
            start_time,
            duration,
            velocity,
            synth_id: synth_id.to_string(),
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
        });
    }

    pub fn add_sample_event(&mut self, uri: &str, start_time: f32, velocity: f32) {
        self.events.push(AudioEvent::Sample {
            uri: uri.to_string(),
            start_time,
            velocity,
        });
    }

    pub fn get_synth(&self, name: &str) -> Option<&SynthDefinition> {
        self.synths.get(name)
    }

    pub fn total_duration(&self) -> f32 {
        self.events
            .iter()
            .map(|event| match event {
                AudioEvent::Note {
                    start_time,
                    duration,
                    ..
                } => start_time + duration,
                AudioEvent::Chord {
                    start_time,
                    duration,
                    ..
                } => start_time + duration,
                AudioEvent::Sample {
                    start_time, uri, ..
                } => {
                    // Get actual sample duration from registry
                    #[cfg(target_arch = "wasm32")]
                    {
                        use crate::web::registry::samples::get_sample;
                        if let Some(pcm) = get_sample(uri) {
                            // Assume 44.1kHz sample rate
                            let duration = pcm.len() as f32 / 44100.0;
                            start_time + duration
                        } else {
                            // Fallback: estimate 2 seconds
                            start_time + 2.0
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Fallback for native: estimate 2 seconds
                        let _ = uri; // Silence unused warning on non-WASM targets
                        start_time + 2.0
                    }
                }
            })
            .fold(0.0, f32::max)
    }

    /// Merge another AudioEventList into this one
    /// This is used for parallel spawn execution
    pub fn merge(&mut self, other: AudioEventList) {
        // Merge synth definitions FIRST (prefer existing definitions on conflict)
        for (name, def) in other.synths {
            if !self.synths.contains_key(&name) {
                self.synths.insert(name, def);
            }
        }

        // Merge events and update their synth_def snapshots if needed
        for mut event in other.events {
            // Update synth_def snapshot for Note and Chord events
            match &mut event {
                AudioEvent::Note {
                    synth_id,
                    synth_def,
                    ..
                } => {
                    // If this event's synth now exists in merged synths, update the snapshot
                    if let Some(updated_def) = self.synths.get(synth_id) {
                        *synth_def = updated_def.clone();
                    }
                }
                AudioEvent::Chord {
                    synth_id,
                    synth_def,
                    ..
                } => {
                    // If this event's synth now exists in merged synths, update the snapshot
                    if let Some(updated_def) = self.synths.get(synth_id) {
                        *synth_def = updated_def.clone();
                    }
                }
                _ => {}
            }
            self.events.push(event);
        }
    }
}

/// Helper to extract values from Value::Map
pub fn extract_number(map: &HashMap<String, Value>, key: &str, default: f32) -> f32 {
    map.get(key)
        .and_then(|v| {
            if let Value::Number(n) = v {
                Some(*n)
            } else {
                None
            }
        })
        .unwrap_or(default)
}

pub fn extract_string(map: &HashMap<String, Value>, key: &str, default: &str) -> String {
    map.get(key)
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| default.to_string())
}

pub fn extract_filters(filters_arr: &[Value]) -> Vec<FilterDef> {
    filters_arr
        .iter()
        .filter_map(|v| {
            if let Value::Map(filter_map) = v {
                let filter_type = extract_string(filter_map, "type", "lowpass");
                let cutoff = filter_map
                    .get("cutoff")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .unwrap_or(1000.0);
                let resonance = filter_map
                    .get("resonance")
                    .and_then(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .unwrap_or(1.0);

                Some(FilterDef {
                    filter_type,
                    cutoff,
                    resonance,
                })
            } else {
                None
            }
        })
        .collect()
}
