/// Debugger utilities - logging and introspection
use std::time::Instant;

pub struct DebugTimer {
    start: Instant,
    label: String,
}

impl DebugTimer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    pub fn log_elapsed(&self) {}
}

impl Drop for DebugTimer {
    fn drop(&mut self) {
        self.log_elapsed();
    }
}

#[cfg(test)]
#[path = "test_shared_debugger.rs"]
mod tests;
