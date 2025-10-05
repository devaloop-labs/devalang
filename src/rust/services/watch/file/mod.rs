#![cfg(feature = "cli")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::{self, Receiver};

use crate::tools::logger::Logger;

#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub debounce: Duration,
    pub poll_interval: Duration,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(150),
            poll_interval: Duration::from_millis(500),
        }
    }
}

pub struct FileWatcher {
    logger: Arc<Logger>,
}

impl FileWatcher {
    pub fn new(logger: Arc<Logger>) -> Self {
        Self { logger }
    }

    #[allow(clippy::unused_async)]
    pub async fn watch(&self, path: PathBuf, options: WatchOptions) -> Result<FileWatchStream> {
        let (tx, rx) = mpsc::channel(32);
        let config = Config::default().with_poll_interval(options.poll_interval);
        let fallback_path = path.clone();
        let logger = self.logger.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if !is_relevant(&event.kind) {
                        return;
                    }
                    let target = event
                        .paths
                        .first()
                        .cloned()
                        .unwrap_or_else(|| fallback_path.clone());
                    let send_result = tx.blocking_send(target);
                    if let Err(err) = send_result {
                        logger.warn(format!(
                            "Watch channel dropped before event could be delivered: {err}"
                        ));
                    }
                }
                Err(err) => {
                    logger.error(format!("Watch error: {err}"));
                }
            },
            config,
        )?;

        watcher
            .watch(Path::new(&path), RecursiveMode::NonRecursive)
            .with_context(|| format!("failed to watch {}", path.display()))?;

        Ok(FileWatchStream::new(
            path,
            rx,
            watcher,
            self.logger.clone(),
            options.debounce,
        ))
    }
}

fn is_relevant(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
    )
}

pub struct FileWatchStream {
    #[allow(dead_code)]
    path: PathBuf,
    receiver: Receiver<PathBuf>,
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    logger: Arc<Logger>,
    debounce: Duration,
    last_emit: Option<Instant>,
}

impl FileWatchStream {
    fn new(
        path: PathBuf,
        receiver: Receiver<PathBuf>,
        watcher: RecommendedWatcher,
        logger: Arc<Logger>,
        debounce: Duration,
    ) -> Self {
        Self {
            path,
            receiver,
            watcher,
            logger,
            debounce,
            last_emit: None,
        }
    }

    pub async fn next_change(&mut self) -> Option<PathBuf> {
        while let Some(path) = self.receiver.recv().await {
            let now = Instant::now();
            if let Some(last) = self.last_emit {
                if now.duration_since(last) < self.debounce {
                    continue;
                }
            }
            self.last_emit = Some(now);
            self.logger
                .watch(format!("Change detected: {}", path.display()));
            return Some(path);
        }
        None
    }
}
