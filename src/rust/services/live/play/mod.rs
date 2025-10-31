#![cfg(feature = "cli")]

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::select;

use crate::engine::audio::playback::live::{
    LiveAudioSource, LivePlaybackEngine, LivePlaybackOptions,
};
use crate::services::build::pipeline::{BuildArtifacts, BuildRequest, ProjectBuilder};
use crate::services::watch::file::{FileWatcher, WatchOptions};
use crate::tools::logger::Logger;

#[derive(Debug, Clone)]
pub struct LivePlayRequest {
    pub build: BuildRequest,
    pub live_mode: bool,
    pub crossfade_ms: u64,
    pub volume: f32,
}

pub struct LivePlayService {
    logger: Arc<Logger>,
    playback: LivePlaybackEngine,
    builder: ProjectBuilder,
    /// Guard clone to keep bg_rx alive for the duration of a live session
    bg_rx_guard: std::sync::Mutex<
        Option<
            std::sync::Arc<
                std::sync::Mutex<
                    std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>,
                >,
            >,
        >,
    >,
}

impl LivePlayService {
    pub fn new(logger: Arc<Logger>, builder: ProjectBuilder) -> Result<Self> {
        let playback = LivePlaybackEngine::new(logger.clone())
            .context("failed to initialise audio playback engine")?;
        Ok(Self {
            logger,
            playback,
            builder,
            bg_rx_guard: std::sync::Mutex::new(None),
        })
    }

    pub async fn run(&self, request: LivePlayRequest) -> Result<()> {
        if request.live_mode {
            self.run_live(request).await
        } else {
            self.run_offline(request).await
        }
    }

    async fn run_offline(&self, request: LivePlayRequest) -> Result<()> {
        let artifacts = self.builder.build(&request.build)?;
        self.logger
            .debug(format!("Build RMS: {:.4}", artifacts.rms));
        self.logger.watch(format!(
            "Audio regenerated in {} (total build {})",
            format_duration(artifacts.audio_render_time),
            format_duration(artifacts.total_duration)
        ));
        self.logger.info(format!(
            "Loop length ≈ {}",
            format_duration(artifacts.audio_length)
        ));
        self.logger.success(format!(
            "Artifacts written: AST={}, audio={}",
            artifacts.ast_path.display(),
            artifacts.primary_audio_path.display()
        ));

        let source = LiveAudioSource::from_artifacts(&artifacts);
        self.playback.play_once(source, request.volume).await?;
        self.logger.info("Playback finished.");
        Ok(())
    }

    async fn run_live(&self, request: LivePlayRequest) -> Result<()> {
        let mut artifacts = match self.builder.build(&request.build) {
            Ok(artifacts) => artifacts,
            Err(err) => {
                self.logger.error(format!("Initial build failed: {err}"));
                return Err(err);
            }
        };
        self.logger
            .debug(format!("Build RMS: {:.4}", artifacts.rms));
        self.logger.watch(format!(
            "Audio regenerated in {} (total build {})",
            format_duration(artifacts.audio_render_time),
            format_duration(artifacts.total_duration)
        ));
        self.logger.info(format!(
            "Loop length ≈ {}",
            format_duration(artifacts.audio_length)
        ));
        let poll = Duration::from_millis(request.crossfade_ms.max(10));
        let options = LivePlaybackOptions::new(poll).with_volume(request.volume);

        let initial_source = LiveAudioSource::from_artifacts(&artifacts);

        // Spawn a persistent interpreter thread to keep "loop pass" background workers
        // alive and to print realtime messages while the live session is active.
        use crate::engine::audio::interpreter::driver::AudioInterpreter;
        use std::sync::mpsc as std_mpsc;
        use std::sync::{Arc as StdArc, Mutex as StdMutex};

        // Create a single background channel for the entire live session and wrap the
        // receiver in an Arc<Mutex<...>> so the receiver's lifetime can be owned by the
        // LivePlaybackSession and shared (by reference) with the persistent thread.
        let (bg_tx, bg_rx_std) =
            std_mpsc::channel::<crate::engine::audio::events::AudioEventList>();
        let bg_rx = StdArc::new(StdMutex::new(bg_rx_std));
        // Also emit via the structured logger so it appears in normal log outputs
        self.logger.debug(format!(
            "[LIVE] created bg channel; bg_rx strong_count={}",
            StdArc::strong_count(&bg_rx)
        ));

        // Keep a guardian clone in the service to avoid accidental drop during live session
        if let Ok(mut g) = self.bg_rx_guard.lock() {
            *g = Some(bg_rx.clone());
        }

        let mut persistent_stop_tx: Option<std_mpsc::Sender<()>>;
        let mut persistent_handle: Option<std::thread::JoinHandle<()>>;

        // Helper closure to spawn a persistent interpreter that will run collect_events once
        // to register groups and spawn background workers, then wait until signalled to stop.
        // It accepts the bg_tx and an Arc<Mutex<Receiver>> owned by the session.
        let spawn_persistent = |stmts: Vec<crate::language::syntax::ast::Statement>,
                                sample_rate: u32,
                                bg_tx: std_mpsc::Sender<
            crate::engine::audio::events::AudioEventList,
        >,
                                bg_rx: StdArc<
            StdMutex<std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>>,
        >,
                                logger: Arc<Logger>| {
            // Stop signal channel for the persistent thread
            let (stop_tx, stop_rx) = std_mpsc::channel::<()>();

            let handle = std::thread::spawn(move || {
                use std::time::Duration as StdDuration;
                // Run the persistent interpreter inside a panic-catch so we can
                // log if it exits unexpectedly (panics will otherwise be silent
                // inside spawned threads and lead to the bg receiver being
                // dropped, causing worker sends to fail).
                let r = std::panic::catch_unwind(|| {
                    logger.debug(format!(
                        "[PERSISTENT] starting persistent interpreter (sample_rate={})",
                        sample_rate
                    ));
                    logger.debug(format!(
                        "[PERSISTENT] bg_rx strong_count at thread start={}",
                        StdArc::strong_count(&bg_rx)
                    ));
                    let mut interp = AudioInterpreter::new(sample_rate);
                    // Suppress immediate prints in the persistent interpreter so that
                    // the live playback engine (which replays the build's .printlog)
                    // is the single source of truth for timed PRINT output. This
                    // avoids duplicate prints (once from the interpreter and once
                    // from the playback sidecar) while keeping scheduled logs in
                    // the merged events for rendering.
                    interp.suppress_print = true;
                    interp.suppress_beat_emit = true;

                    // Install the sender so that loop pass workers will reuse it instead of
                    // creating their own channel and receiver owned by the interpreter.
                    interp.background_event_tx = Some(bg_tx.clone());

                    // Collect events once to register groups and spawn background 'pass' workers
                    let _ = interp.collect_events(&stmts);
                    logger.debug(format!(
                        "[PERSISTENT] collect_events() returned; background_event_tx present={}",
                        interp.background_event_tx.is_some()
                    ));

                    // Drain and merge background worker batches continuously until stop signal.
                    loop {
                        // Check for stop signal without blocking forever
                        match stop_rx.try_recv() {
                            Ok(_) | Err(std_mpsc::TryRecvError::Disconnected) => break,
                            Err(std_mpsc::TryRecvError::Empty) => {}
                        }

                        // Receive one batch from the bg_rx with timeout and merge
                        logger.debug(format!(
                            "[PERSISTENT] bg_rx strong_count before recv_timeout={}",
                            StdArc::strong_count(&bg_rx)
                        ));
                        match bg_rx
                            .lock()
                            .expect("bg_rx lock")
                            .recv_timeout(StdDuration::from_millis(200))
                        {
                            Ok(events) => {
                                // Debug: report merge and earliest time
                                let cnt = events.events.len();
                                let mut times: Vec<f32> = events
                                    .events
                                    .iter()
                                    .map(|e| match e {
                                        crate::engine::audio::events::AudioEvent::Note {
                                            start_time,
                                            ..
                                        } => *start_time,
                                        crate::engine::audio::events::AudioEvent::Chord {
                                            start_time,
                                            ..
                                        } => *start_time,
                                        crate::engine::audio::events::AudioEvent::Sample {
                                            start_time,
                                            ..
                                        } => *start_time,
                                    })
                                    .collect();
                                for (t, _m) in &events.logs {
                                    times.push(*t);
                                }
                                let earliest = times.into_iter().min_by(|a, b| {
                                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                                });
                                if let Some(t) = earliest {
                                    logger.debug(format!(
                                        "[PERSISTENT] merging {} bg events, earliest_start_time={}",
                                        cnt, t
                                    ));
                                } else {
                                    logger.debug(format!("[PERSISTENT] merging {} bg events", cnt));
                                }
                                // Merge produced events into persistent interpreter's event list
                                interp.events.merge(events);
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                // nothing to merge now, continue loop
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                                // workers are gone
                                break;
                            }
                        }
                    }
                    logger.debug(
                        "[PERSISTENT] persistent interpreter exiting (dropping interp)".to_string(),
                    );
                    // dropping interp will stop background workers
                });

                // If the persistent interpreter panicked, log the payload so we can
                // diagnose unexpected thread exits that would drop the bg receiver.
                if let Err(err) = r {
                    logger.error(format!(
                        "[PERSISTENT] persistent thread panicked: {:?}",
                        err
                    ));
                }
            });

            // Return the stop sender so callers can request the thread to stop, and the join handle
            (stop_tx, handle)
        };

        // Start persistent interpreter for initial artifacts
        {
            let (tx, handle) = spawn_persistent(
                artifacts.statements.clone(),
                artifacts.sample_rate,
                bg_tx.clone(),
                bg_rx.clone(),
                self.logger.clone(),
            );
            persistent_stop_tx = Some(tx);
            persistent_handle = Some(handle);
        }

        let session = self
            .playback
            .start_live_session(initial_source, options, Some(bg_rx.clone()))
            .await?;
        let mut best_audio_render_time = artifacts.audio_render_time;

        self.logger.watch(format!(
            "Live mode watching {}",
            request.build.entry_path.display()
        ));
        let watcher = FileWatcher::new(self.logger.clone());
        let mut stream = watcher
            .watch(request.build.entry_path.clone(), WatchOptions::default())
            .await
            .context("failed to initialise file watcher")?;

        loop {
            select! {
                change = stream.next_change() => {
                    match change {
                        Some(path) => {
                            self.logger.watch(format!("Rebuilding after change at {}", path.display()));
                            match self.builder.build(&request.build) {
                                Ok(new_artifacts) => {
                                    self.logger
                                        .debug(format!("Build RMS: {:.4}", new_artifacts.rms));
                                    self.logger.watch(format!(
                                        "Audio regenerated in {} (total build {})",
                                        format_duration(new_artifacts.audio_render_time),
                                        format_duration(new_artifacts.total_duration)
                                    ));
                                    self.logger.info(format!(
                                        "Loop length ≈ {}",
                                        format_duration(new_artifacts.audio_length)
                                    ));
                                    if new_artifacts.audio_render_time < best_audio_render_time {
                                        best_audio_render_time = new_artifacts.audio_render_time;
                                        self.logger.success(format!(
                                            "⏱️ New best audio regen time: {}",
                                            format_duration(best_audio_render_time)
                                        ));
                                    } else {
                                        self.logger.info(format!(
                                            "Best audio regen time so far: {}",
                                            format_duration(best_audio_render_time)
                                        ));
                                    }
                                    // Stop previous persistent interpreter (if any) and spawn a new one for updated statements
                                    if let Some(tx) = persistent_stop_tx.take() {
                                        let _ = tx.send(());
                                    }
                                    if let Some(handle) = persistent_handle.take() {
                                        let _ = handle.join();
                                    }

                                    artifacts = new_artifacts;
                                    let (tx, handle) = spawn_persistent(artifacts.statements.clone(), artifacts.sample_rate, bg_tx.clone(), bg_rx.clone(), self.logger.clone());
                                    persistent_stop_tx = Some(tx);
                                    persistent_handle = Some(handle);

                                    let next_source = LiveAudioSource::from_artifacts(&artifacts);
                                    if let Err(err) = session.queue_source(next_source) {
                                        self.logger.error(format!("Failed to queue live buffer: {err}"));
                                    }
                                }
                                Err(err) => {
                                    self.logger.error(format!("Build failed after change: {err}"));
                                }
                            }
                        }
                        None => {
                            self.logger.warn("Watch stream ended; shutting down live playback");
                            break;
                        }
                    }
                }
                _ = session.heartbeat() => {}
            }
        }

        // Wait for session completion, then clear our guardian clone so the receiver may be dropped
        let res = session.finish().await;
        // clear guard
        if let Ok(mut g) = self.bg_rx_guard.lock() {
            let _ = g.take();
        }
        res
    }
}

impl LiveAudioSource {
    fn from_artifacts(artifacts: &BuildArtifacts) -> Self {
        LiveAudioSource::with_path(
            artifacts.primary_audio_path.clone(),
            artifacts.primary_format,
            artifacts.bit_depth,
            artifacts.channels,
            artifacts.sample_rate,
            artifacts.resample_quality,
            artifacts.audio_length,
        )
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() >= 1 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        let ms = duration.as_secs_f64() * 1000.0;
        if ms >= 100.0 {
            format!("{:.0}ms", ms)
        } else {
            format!("{:.1}ms", ms)
        }
    }
}
