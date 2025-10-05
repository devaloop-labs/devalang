//! Playhead event tracking for WASM
//!
//! This module manages playhead events that can be collected by the JavaScript
//! side to display visual feedback during audio playback.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// Playhead event - represents a note or audio event at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayheadEvent {
    /// Event type: "note_on", "note_off", "chord_on", "chord_off"
    pub event_type: String,

    /// MIDI note number(s)
    pub midi: Vec<u8>,

    /// Time in seconds when the event occurs
    pub time: f32,

    /// Velocity (0.0 to 1.0)
    pub velocity: f32,

    /// Synth ID
    pub synth_id: String,
}

/// Callback function type for playhead events
type PlayheadCallback = Box<dyn Fn(PlayheadEvent)>;

thread_local! {
    /// Collected playhead events
    static PLAYHEAD_EVENTS: RefCell<Vec<PlayheadEvent>> = RefCell::new(Vec::new());

    /// Registered playhead callback
    static PLAYHEAD_CALLBACK: RefCell<Option<js_sys::Function>> = RefCell::new(None);
}

/// Add a playhead event
pub fn push_event(event: PlayheadEvent) {
    // Store event
    PLAYHEAD_EVENTS.with(|events| {
        events.borrow_mut().push(event.clone());
    });

    // Call callback if registered
    PLAYHEAD_CALLBACK.with(|callback| {
        if let Some(cb) = callback.borrow().as_ref() {
            // Convert event to JsValue and call callback
            if let Ok(js_event) = serde_wasm_bindgen::to_value(&event) {
                let _ = cb.call1(&JsValue::NULL, &js_event);
            }
        }
    });
}

/// Register a JavaScript callback for playhead events
pub fn register_callback(callback: js_sys::Function) {
    PLAYHEAD_CALLBACK.with(|cb| {
        *cb.borrow_mut() = Some(callback);
    });
}

/// Unregister the playhead callback
pub fn unregister_callback() {
    PLAYHEAD_CALLBACK.with(|cb| {
        *cb.borrow_mut() = None;
    });
}

/// Get collected playhead events
pub fn get_events() -> Vec<PlayheadEvent> {
    PLAYHEAD_EVENTS.with(|events| events.borrow().clone())
}

/// Clear all playhead events
pub fn clear_events() {
    PLAYHEAD_EVENTS.with(|events| {
        events.borrow_mut().clear();
    });
}

/// Get event count
pub fn event_count() -> usize {
    PLAYHEAD_EVENTS.with(|events| events.borrow().len())
}
