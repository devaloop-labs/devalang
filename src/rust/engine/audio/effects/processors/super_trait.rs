use std::fmt::Debug;

/// Trait for all effect processors
pub trait EffectProcessor: Send + Debug {
    /// Process audio samples (stereo interleaved)
    fn process(&mut self, samples: &mut [f32], sample_rate: u32);

    /// Reset effect state
    fn reset(&mut self);

    /// Get effect name
    fn name(&self) -> &str;
}
