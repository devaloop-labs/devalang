use super::EffectAvailability;
use crate::engine::audio::effects::processors::EffectProcessor;
use crate::engine::audio::effects::processors::{
    BandpassProcessor, BitcrushProcessor, FreezeProcessor, HighpassProcessor, LfoProcessor,
    LowpassProcessor, MonoizerProcessor, ReverseProcessor, RollProcessor, SliceProcessor,
    SpeedProcessor, StereoProcessor, StretchProcessor, TremoloProcessor, VibratoProcessor,
};
use crate::engine::audio::effects::processors::{
    ChorusProcessor, CompressorProcessor, DelayProcessor, DistortionProcessor, DriveProcessor,
    FlangerProcessor, PhaserProcessor, ReverbProcessor,
};
use std::collections::HashMap;

/// Effect registry - stores available effects and their processors
#[derive(Debug)]
pub struct EffectRegistry {
    effects: HashMap<&'static str, (EffectAvailability, Box<dyn CloneableEffect>)>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            effects: HashMap::new(),
        };

        // Common effects
        registry.register_effect(
            "gain",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "pan",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "fadeIn",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "fadeOut",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "pitch",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "chorus",
            EffectAvailability::Both,
            Box::new(ChorusProcessor::default()),
        );
        registry.register_effect(
            "flanger",
            EffectAvailability::Both,
            Box::new(FlangerProcessor::default()),
        );
        registry.register_effect(
            "phaser",
            EffectAvailability::Both,
            Box::new(PhaserProcessor::default()),
        );
        registry.register_effect(
            "compressor",
            EffectAvailability::Both,
            Box::new(CompressorProcessor::default()),
        );
        registry.register_effect(
            "drive",
            EffectAvailability::Both,
            Box::new(DriveProcessor::default()),
        );
        registry.register_effect(
            "reverb",
            EffectAvailability::Both,
            Box::new(ReverbProcessor::default()),
        );
        registry.register_effect(
            "delay",
            EffectAvailability::Both,
            Box::new(DelayProcessor::default()),
        );
        registry.register_effect(
            "bitcrush",
            EffectAvailability::Both,
            Box::new(BitcrushProcessor::default()),
        );
        registry.register_effect(
            "lowpass",
            EffectAvailability::Both,
            Box::new(LowpassProcessor::default()),
        );
        registry.register_effect(
            "highpass",
            EffectAvailability::Both,
            Box::new(HighpassProcessor::default()),
        );
        registry.register_effect(
            "bandpass",
            EffectAvailability::Both,
            Box::new(BandpassProcessor::default()),
        );
        registry.register_effect(
            "tremolo",
            EffectAvailability::Both,
            Box::new(TremoloProcessor::default()),
        );
        registry.register_effect(
            "vibrato",
            EffectAvailability::Both,
            Box::new(VibratoProcessor::default()),
        );
        registry.register_effect(
            "mono",
            EffectAvailability::Both,
            Box::new(MonoizerProcessor::default()),
        );
        registry.register_effect(
            "monoizer",
            EffectAvailability::Both,
            Box::new(MonoizerProcessor::default()),
        );
        registry.register_effect(
            "stereo",
            EffectAvailability::Both,
            Box::new(StereoProcessor::default()),
        );
        registry.register_effect(
            "freeze",
            EffectAvailability::Both,
            Box::new(FreezeProcessor::default()),
        );
        registry.register_effect(
            "distortion",
            EffectAvailability::Both,
            Box::new(DistortionProcessor::default()),
        );
        registry.register_effect(
            "dist",
            EffectAvailability::Both,
            Box::new(DistortionProcessor::default()),
        );
        registry.register_effect(
            "lfo",
            EffectAvailability::Both,
            Box::new(LfoProcessor::default()),
        );

        // Trigger-only effects
        registry.register_effect(
            "reverse",
            EffectAvailability::TriggerOnly,
            Box::new(ReverseProcessor::default()),
        );
        registry.register_effect(
            "speed",
            EffectAvailability::TriggerOnly,
            Box::new(SpeedProcessor::default()),
        );
        registry.register_effect(
            "slice",
            EffectAvailability::TriggerOnly,
            Box::new(SliceProcessor::default()),
        );
        registry.register_effect(
            "stretch",
            EffectAvailability::TriggerOnly,
            Box::new(StretchProcessor::default()),
        );
        registry.register_effect(
            "roll",
            EffectAvailability::TriggerOnly,
            Box::new(RollProcessor::default()),
        );

        // Aliases
        registry.register_effect(
            "dist",
            EffectAvailability::Both,
            Box::new(DistortionProcessor::default()),
        );
        registry.register_effect(
            "comp",
            EffectAvailability::Both,
            Box::new(CompressorProcessor::default()),
        );
        registry.register_effect(
            "lpf",
            EffectAvailability::Both,
            Box::new(LowpassProcessor::default()),
        );
        registry.register_effect(
            "hpf",
            EffectAvailability::Both,
            Box::new(HighpassProcessor::default()),
        );
        registry.register_effect(
            "bpf",
            EffectAvailability::Both,
            Box::new(BandpassProcessor::default()),
        );

        // Synth-only effects (could add more specific synth effects here)

        registry
    }

    /// Register a new effect with its availability and processor
    pub fn register_effect(
        &mut self,
        name: &'static str,
        availability: EffectAvailability,
        processor: Box<dyn CloneableEffect>,
    ) {
        self.effects.insert(name, (availability, processor));
    }

    /// Get an effect processor by name if it's available for the given context
    pub fn get_effect(&self, name: &str, synth_context: bool) -> Option<Box<dyn CloneableEffect>> {
        self.effects
            .get(name)
            .and_then(|(availability, processor)| {
                match (availability, synth_context) {
                    // In synth context, allow SynthOnly and Both
                    (EffectAvailability::SynthOnly, true) | (EffectAvailability::Both, true) => {
                        Some(processor.clone_box())
                    }
                    // In trigger context, allow TriggerOnly and Both
                    (EffectAvailability::TriggerOnly, false)
                    | (EffectAvailability::Both, false) => Some(processor.clone_box()),
                    _ => None, // Effect not available in this context
                }
            })
    }

    /// Check if an effect exists and is available in the given context
    pub fn is_effect_available(&self, name: &str, synth_context: bool) -> bool {
        self.effects.get(name).map_or(false, |(availability, _)| {
            match (availability, synth_context) {
                (EffectAvailability::SynthOnly, true)
                | (EffectAvailability::Both, true)
                | (EffectAvailability::TriggerOnly, false)
                | (EffectAvailability::Both, false) => true,
                _ => false,
            }
        })
    }

    /// List all available effects for a given context
    pub fn list_available_effects(&self, synth_context: bool) -> Vec<&'static str> {
        self.effects
            .iter()
            .filter_map(
                |(&name, (availability, _))| match (availability, synth_context) {
                    (EffectAvailability::SynthOnly, true)
                    | (EffectAvailability::Both, true)
                    | (EffectAvailability::TriggerOnly, false)
                    | (EffectAvailability::Both, false) => Some(name),
                    _ => None,
                },
            )
            .collect()
    }
}

/// Trait object that is cloneable: used to store prototype processors in the registry.
/// This is a super-trait combining `EffectProcessor` behaviour and a `clone_box` method
/// returning a boxed clone of the same dyn trait.
pub trait CloneableEffect: EffectProcessor {
    fn clone_box(&self) -> Box<dyn CloneableEffect>;
}

impl<T> CloneableEffect for T
where
    T: EffectProcessor + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneableEffect> {
        Box::new(self.clone())
    }
}

impl dyn EffectProcessor {
    /// Get description of effect parameters
    pub fn get_parameters_description(&self) -> HashMap<&'static str, String> {
        let mut params = HashMap::new();
        match self.name() {
            "Chorus" => {
                params.insert("depth", "Modulation depth (0.0 to 1.0)".to_string());
                params.insert("rate", "Modulation rate (0.1 to 10.0 Hz)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Flanger" => {
                params.insert("depth", "Modulation depth (0.0 to 1.0)".to_string());
                params.insert("rate", "Modulation rate (0.1 to 10.0 Hz)".to_string());
                params.insert("feedback", "Feedback amount (0.0 to 0.95)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Phaser" => {
                params.insert("stages", "Number of allpass stages (2 to 12)".to_string());
                params.insert("rate", "Modulation rate (0.1 to 10.0 Hz)".to_string());
                params.insert("depth", "Modulation depth (0.0 to 1.0)".to_string());
                params.insert("feedback", "Feedback amount (0.0 to 0.95)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Compressor" => {
                params.insert(
                    "threshold",
                    "Threshold level in dB (-60.0 to 0.0)".to_string(),
                );
                params.insert("ratio", "Compression ratio (1.0 to 20.0)".to_string());
                params.insert(
                    "attack",
                    "Attack time in seconds (0.001 to 1.0)".to_string(),
                );
                params.insert(
                    "release",
                    "Release time in seconds (0.001 to 2.0)".to_string(),
                );
            }
            "Drive" => {
                params.insert("amount", "Drive amount (0.0 to 1.0)".to_string());
                params.insert("tone", "Tone control (0.0 to 1.0)".to_string());
                params.insert(
                    "color",
                    "Color/timbre control (0.0 bright to 1.0 dark)".to_string(),
                );
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Distortion" => {
                params.insert("amount", "Distortion amount (0.0 to 1.0)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Bitcrush" => {
                params.insert("depth", "Bit depth (1..16)".to_string());
                params.insert(
                    "sample_rate",
                    "Target sample rate for downsampling (Hz)".to_string(),
                );
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Lowpass" => {
                params.insert("cutoff", "Cutoff frequency (20.0 to 20000.0)".to_string());
                params.insert("resonance", "Resonance/Q (0.0 to 1.0)".to_string());
            }
            "Highpass" => {
                params.insert("cutoff", "Cutoff frequency (20.0 to 20000.0)".to_string());
                params.insert("resonance", "Resonance/Q (0.0 to 1.0)".to_string());
            }
            "Bandpass" => {
                params.insert("cutoff", "Center frequency (20.0 to 20000.0)".to_string());
                params.insert("resonance", "Bandwidth/Resonance (0.0 to 1.0)".to_string());
            }
            "Tremolo" => {
                params.insert("rate", "LFO rate (0.1 to 20.0 Hz)".to_string());
                params.insert("depth", "Depth (0.0 to 1.0)".to_string());
                params.insert("sync", "Sync to tempo (true/false)".to_string());
            }
            "Vibrato" => {
                params.insert("rate", "LFO rate (0.1 to 10.0 Hz)".to_string());
                params.insert(
                    "depth",
                    "Delay depth in seconds (small, e.g. 0.003)".to_string(),
                );
                params.insert("sync", "Sync to tempo (true/false)".to_string());
            }
            "Monoizer" => {
                params.insert("enabled", "Enable monoizer (true/false)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Stereo" => {
                params.insert("width", "Stereo width (0.0 to 2.0)".to_string());
            }
            "Freeze" => {
                params.insert("enabled", "Enable freeze (true/false)".to_string());
                params.insert(
                    "fade",
                    "Fade-in amount when freezing (0.0 to 1.0)".to_string(),
                );
                params.insert("hold", "Hold time in seconds (0.05 to 5.0)".to_string());
            }
            "Slice" => {
                params.insert("segments", "Number of segments (1 to 16)".to_string());
                params.insert("mode", "Mode: sequential | random".to_string());
                params.insert(
                    "crossfade",
                    "Crossfade between slices (0.0 to 1.0)".to_string(),
                );
            }
            "Stretch" => {
                params.insert("factor", "Time stretch factor (0.25 to 4.0)".to_string());
                params.insert(
                    "pitch",
                    "Pitch shift in semitones (-48.0 to 48.0)".to_string(),
                );
                params.insert("formant", "Preserve formants (true/false)".to_string());
            }
            "Roll" => {
                params.insert("duration_ms", "Roll segment duration in ms".to_string());
                params.insert("sync", "Sync to tempo (true/false)".to_string());
                params.insert("repeats", "Number of repeats (1 to 16)".to_string());
                params.insert("fade", "Crossfade between repeats (0.0 to 1.0)".to_string());
            }
            "Reverb" => {
                params.insert("size", "Room size (0.0 to 1.0)".to_string());
                params.insert(
                    "decay",
                    "Decay/time multiplier (0.0 short to 2.0 long)".to_string(),
                );
                params.insert("damping", "High frequency damping (0.0 to 1.0)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Delay" => {
                params.insert(
                    "time",
                    "Delay time in milliseconds (1.0 to 2000.0)".to_string(),
                );
                params.insert("feedback", "Feedback amount (0.0 to 0.95)".to_string());
                params.insert("mix", "Wet/dry mix (0.0 to 1.0)".to_string());
            }
            "Reverse" => {
                params.insert(
                    "enabled",
                    "Enable/disable reverse effect (true/false)".to_string(),
                );
            }
            "Speed" => {
                params.insert(
                    "speed",
                    "Playback speed multiplier (0.1 to 4.0)".to_string(),
                );
            }
            _ => {}
        }
        params
    }
}

// trigger-specific processors moved to effects::processors module

#[cfg(test)]
#[path = "test_registry.rs"]
mod tests;
