use super::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_debug_timer() {
    let timer = DebugTimer::new("test");
    thread::sleep(Duration::from_millis(10));
    assert!(timer.elapsed_ms() >= 10.0);
}
