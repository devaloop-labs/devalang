//! Declarative macros for exporting plugin functions safely.
//!
//! These macros generate the necessary FFI wrappers without requiring
//! procedural macros, keeping everything simple and safe.

/// Export a safe function as a plugin with automatic FFI wrapper generation.
///
/// This macro eliminates the need for `unsafe` code in plugin implementations.
/// It generates a `#[no_mangle] extern "C"` wrapper that handles all FFI concerns.
///
/// # Usage
///
/// ```rust,ignore
/// use devalang_bindings::*;
///
/// export_plugin!(my_synth, |out, params, note, freq, amp| {
///     // Your safe synthesis code here
///     for frame in 0..params.frames {
///         let sample = (freq * frame as f32).sin() * amp;
///         for ch in 0..params.channels {
///             let idx = (frame * params.channels + ch) as usize;
///             out[idx] = sample;
///         }
///     }
/// });
/// ```
///
/// # Parameters
///
/// The closure receives:
/// - `out: &mut [f32]` - Output buffer (interleaved)
/// - `params: BufferParams` - Sample rate, channels, frames
/// - `note: Note` - MIDI note information
/// - `freq: f32` - Frequency in Hz
/// - `amp: f32` - Amplitude (0.0 to 1.0)
#[macro_export]
macro_rules! export_plugin {
    ($name:ident, $impl_fn:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            out_ptr: *mut f32,
            out_len: i32,
            freq: f32,
            amp: f32,
            duration_ms: i32,
            sample_rate: i32,
            channels: i32,
        ) {
            // Validate pointer
            if out_ptr.is_null() {
                return;
            }

            // Validate and convert parameters
            let out_len_usize = out_len.max(0) as usize;
            if out_len_usize == 0 {
                return;
            }

            let channels_val = channels.max(1) as u32;
            let sample_rate_val = sample_rate.max(1) as u32;
            let frames = (out_len_usize / channels_val as usize) as u32;

            // Construct parameters
            let params = $crate::engine::plugin::bindings::types::BufferParams {
                sample_rate: sample_rate_val,
                channels: channels_val,
                frames,
            };

            let note = $crate::engine::plugin::bindings::types::Note {
                pitch: 60,
                velocity: 100,
                duration_ms: duration_ms.max(1) as u32,
            };

            // SAFETY: Host guarantees valid pointer and length
            unsafe {
                let out = core::slice::from_raw_parts_mut(out_ptr, out_len_usize);
                
                // Call the implementation function
                let implementation: fn(&mut [f32], $crate::engine::plugin::bindings::types::BufferParams, $crate::engine::plugin::bindings::types::Note, f32, f32) = $impl_fn;
                implementation(out, params, note, freq, amp);
            }
        }
    };
}

/// Export a plugin with parameter setters.
///
/// This macro creates both a render function and parameter setter functions.
///
/// # Usage
///
/// ```rust,ignore
/// export_plugin_with_state!(
///     my_synth,
///     // State struct
///     struct SynthState {
///         cutoff: f32,
///         resonance: f32,
///     },
///     // Default state
///     SynthState {
///         cutoff: 1000.0,
///         resonance: 0.5,
///     },
///     // Render function
///     |state, out, params, note, freq, amp| {
///         // Use state.cutoff, state.resonance, etc.
///     },
///     // Parameter setters
///     {
///         cutoff: |state, value| state.cutoff = value,
///         resonance: |state, value| state.resonance = value,
///     }
/// );
/// ```
#[macro_export]
macro_rules! export_plugin_with_state {
    (
        $name:ident,
        $state_struct:item,
        $default_state:expr,
        $render:expr,
        { $($param:ident: $setter:expr),* $(,)? }
    ) => {
        use std::sync::Mutex;
        use once_cell::sync::Lazy;
        
        $state_struct
        
        static STATE: Lazy<Mutex<std::collections::HashMap<String, Box<dyn std::any::Any + Send>>>> = 
            Lazy::new(|| Mutex::new(std::collections::HashMap::new()));
        
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            out_ptr: *mut f32,
            out_len: i32,
            freq: f32,
            amp: f32,
            duration_ms: i32,
            sample_rate: i32,
            channels: i32,
        ) {
            if out_ptr.is_null() {
                return;
            }

            let out_len_usize = out_len.max(0) as usize;
            if out_len_usize == 0 {
                return;
            }

            let channels_val = channels.max(1) as u32;
            let sample_rate_val = sample_rate.max(1) as u32;
            let frames = (out_len_usize / channels_val as usize) as u32;

            let params = $crate::engine::plugin::bindings::types::BufferParams {
                sample_rate: sample_rate_val,
                channels: channels_val,
                frames,
            };

            let note = $crate::engine::plugin::bindings::types::Note {
                pitch: 60,
                velocity: 100,
                duration_ms: duration_ms.max(1) as u32,
            };

            unsafe {
                let out = core::slice::from_raw_parts_mut(out_ptr, out_len_usize);
                
                // Get or create state for this instance
                let instance_key = "default".to_string(); // TODO: pass instance ID
                let mut states = STATE.lock().unwrap();
                let state = states.entry(instance_key)
                    .or_insert_with(|| Box::new($default_state))
                    .downcast_mut()
                    .unwrap();
                
                let render_fn: fn(&mut _, &mut [f32], $crate::engine::plugin::bindings::types::BufferParams, $crate::engine::plugin::bindings::types::Note, f32, f32) = $render;
                render_fn(state, out, params, note, freq, amp);
            }
        }
        
        $(
            paste::paste! {
                #[unsafe(no_mangle)]
                pub extern "C" fn [<set $param:camel>](value: f32) {
                    let instance_key = "default".to_string();
                    let mut states = STATE.lock().unwrap();
                    if let Some(state_any) = states.get_mut(&instance_key) {
                        if let Some(state) = state_any.downcast_mut() {
                            let setter: fn(&mut _, f32) = $setter;
                            setter(state, value);
                        }
                    }
                }
            }
        )*
    };
}

/// Helper macro to create a simple oscillator plugin.
///
/// # Usage
///
/// ```rust,ignore
/// simple_oscillator_plugin!(my_sine, Waveform::Sine);
/// ```
#[macro_export]
macro_rules! simple_oscillator_plugin {
    ($name:ident, $waveform:expr) => {
        $crate::export_plugin!($name, |out, params, _note, freq, amp| {
            use $crate::oscillators::Oscillator;
            let mut osc = Oscillator::new($waveform);
            osc.set_frequency(freq);
            osc.set_amplitude(amp);
            osc.render(out, params);
        });
    };
}
