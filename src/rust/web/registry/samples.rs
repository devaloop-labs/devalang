//! Sample registry for WASM
//!
//! Manages loaded audio samples (PCM data) and their metadata.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Registry of loaded samples: URI -> PCM data (i16)
    pub static REGISTERED_SAMPLES: RefCell<HashMap<String, Vec<i16>>> = RefCell::new(HashMap::new());

    /// Map URI to origin URL (for debugging)
    pub static SAMPLE_ORIGIN_URLS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());

    /// Map origin URL back to devalang:// URI
    pub static ORIGIN_TO_URI: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Register a sample with PCM data
pub fn register_sample(uri: String, pcm: Vec<i16>) {
    REGISTERED_SAMPLES.with(|samples| {
        samples.borrow_mut().insert(uri, pcm);
    });
}

/// Register origin URL for a sample
pub fn register_sample_origin(uri: String, origin_url: String) {
    SAMPLE_ORIGIN_URLS.with(|origins| {
        origins.borrow_mut().insert(uri.clone(), origin_url.clone());
    });
    ORIGIN_TO_URI.with(|mapping| {
        mapping.borrow_mut().insert(origin_url, uri);
    });
}

/// Get sample PCM data by URI
pub fn get_sample(uri: &str) -> Option<Vec<i16>> {
    REGISTERED_SAMPLES.with(|samples| samples.borrow().get(uri).cloned())
}

/// Get origin URL for a URI
pub fn get_origin_url(uri: &str) -> Option<String> {
    SAMPLE_ORIGIN_URLS.with(|origins| origins.borrow().get(uri).cloned())
}

/// Get URI from origin URL
pub fn get_uri_from_origin(origin_url: &str) -> Option<String> {
    ORIGIN_TO_URI.with(|mapping| mapping.borrow().get(origin_url).cloned())
}

/// Clear all registered samples
pub fn clear_samples() {
    REGISTERED_SAMPLES.with(|samples| samples.borrow_mut().clear());
    SAMPLE_ORIGIN_URLS.with(|origins| origins.borrow_mut().clear());
    ORIGIN_TO_URI.with(|mapping| mapping.borrow_mut().clear());
}

/// Get list of all registered sample URIs
pub fn list_sample_uris() -> Vec<String> {
    REGISTERED_SAMPLES.with(|samples| samples.borrow().keys().cloned().collect())
}

/// Get count of registered samples
pub fn sample_count() -> usize {
    REGISTERED_SAMPLES.with(|samples| samples.borrow().len())
}

/// Get sample metadata (URI, size, origin)
pub fn get_sample_metadata(uri: &str) -> Option<SampleMetadata> {
    REGISTERED_SAMPLES.with(|samples| {
        samples.borrow().get(uri).map(|pcm| {
            let origin = get_origin_url(uri);
            SampleMetadata {
                uri: uri.to_string(),
                sample_count: pcm.len(),
                size_bytes: pcm.len() * 2, // i16 = 2 bytes
                origin_url: origin,
            }
        })
    })
}

#[derive(Clone, Debug)]
pub struct SampleMetadata {
    pub uri: String,
    pub sample_count: usize,
    pub size_bytes: usize,
    pub origin_url: Option<String>,
}
