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
}

impl LivePlayService {
    pub fn new(logger: Arc<Logger>, builder: ProjectBuilder) -> Result<Self> {
        let playback = LivePlaybackEngine::new(logger.clone())
            .context("failed to initialise audio playback engine")?;
        Ok(Self {
            logger,
            playback,
            builder,
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
        let session = self
            .playback
            .start_live_session(initial_source, options)
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
                                    artifacts = new_artifacts;
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

        session.finish().await
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
