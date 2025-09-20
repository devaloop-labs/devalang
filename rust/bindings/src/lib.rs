//! Minimal, focused bindings helpers for writing custom render functions in
//! plugin crates.
//!
//! The core runtime already handles registration and FFI bridging. This crate
//! only exports the types and function-signature aliases that plugin authors
//! should implement in safe Rust. The host will convert raw buffers/pointers
//! into these safe types before calling the plugin function.
//!
//! Guiding principle: plugin authors write purely safe Rust functions with
//! clear parameters (slices and small structs). No registration helpers or
//! global registries are provided here.

/// Lightweight representation of a musical note.
#[derive(Debug, Clone, Copy)]
pub struct Note {
	/// MIDI pitch 0..127
	pub pitch: u8,
	/// Velocity 0..127
	pub velocity: u8,
	/// Note duration in milliseconds (approximate)
	pub duration_ms: u32,
}

impl Default for Note {
	fn default() -> Self {
		Self { pitch: 60, velocity: 100, duration_ms: 500 }
	}
}

/// Parameters describing the output buffer and audio context.
#[derive(Debug, Clone, Copy)]
pub struct BufferParams {
	/// Sample rate in Hz
	pub sample_rate: u32,
	/// Number of channels (1 = mono, 2 = stereo, ...)
	pub channels: u32,
	/// Number of frames (samples per channel) available in the buffer
	pub frames: u32,
}

/// Helper to compute expected buffer length (frames * channels).
impl BufferParams {
	/// Returns the expected length of the interleaved buffer (frames * channels)
	/// as a host-friendly `usize`. The host is responsible for converting
	/// its platform-sized integers into these explicit-width fields.
	pub fn buffer_len(&self) -> usize {
		(self.frames as usize).saturating_mul(self.channels as usize)
	}

	/// Quick validation helper that plugins can call to ensure the provided
	/// output slice matches the declared `params` length.
	pub fn validate_buffer(out: &mut [f32], params: BufferParams) -> Result<(), &'static str> {
		let expected = params.buffer_len();
		if out.len() < expected { Err("output buffer too small") } else { Ok(()) }
	}
}

impl Default for BufferParams {
	fn default() -> Self {
		Self { sample_rate: 44100, channels: 1, frames: 0 }
	}
}

/// Common render function signature plugin authors should implement.
///
/// - `out` is a mutable slice of f32 samples (interleaved if channels > 1).
/// - `params` provides sample rate / channels / frames info.
/// - `note` is the Note descriptor.
/// - `freq`/`amp` give additional voice parameters the host may pass.
pub type RenderFn = fn(out: &mut [f32], params: BufferParams, note: Note, freq: f32, amp: f32);

/// A more complete signature used by synth-style plugins that also need
/// access to extra controls (e.g. voice index or time offset).
pub type RenderFnExt = fn(
	out: &mut [f32],
	params: BufferParams,
	note: Note,
	freq: f32,
	amp: f32,
	voice_index: u32,
	time_ms: u64,
);

/// Optional signature for control parameter setters. The host can call these
/// safely by converting raw values into primitives.
pub type SetParamFn = fn(param_name: &str, value: f32);