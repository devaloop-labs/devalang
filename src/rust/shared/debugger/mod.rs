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

    pub fn log_elapsed(&self) {
        // Debug timing removed - use profiler if needed
    }
}

impl Drop for DebugTimer {
    fn drop(&mut self) {
        self.log_elapsed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_debug_timer() {
        let timer = DebugTimer::new("test");
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10.0);
    }
}
