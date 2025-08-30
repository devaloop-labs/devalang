use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;

use std::time::{Duration, Instant};

pub fn watch_directory<F>(entry: String, callback: F) -> notify::Result<()>
where
    F: Fn() + Send + 'static,
{
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(&entry.as_ref(), RecursiveMode::Recursive)?;

    let mut last_trigger = Instant::now();

    loop {
        match rx.recv() {
            Ok(_) => {
                let now = Instant::now();
                if now.duration_since(last_trigger) > Duration::from_millis(200) {
                    callback();
                    last_trigger = now;
                }
            }
            Err(e) => {
                // Prefer structured logging when available
                #[cfg(feature = "cli")]
                {
                    let logger = crate::logger::Logger::new();
                    logger.log_message(
                        crate::logger::LogLevel::Error,
                        &format!("Channel error: {:?}", e),
                    );
                }
                #[cfg(not(feature = "cli"))]
                {
                    eprintln!("Channel error: {:?}", e);
                }
                break;
            }
        }
    }

    Ok(())
}
