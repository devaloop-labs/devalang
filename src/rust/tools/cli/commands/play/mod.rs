#![cfg(feature = "cli")]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};
use crate::platform::config::AppConfig;
use crate::services::build::pipeline::{BuildRequest, ProjectBuilder};
use crate::services::live::play::{LivePlayRequest, LivePlayService};
use crate::tools::cli::state::CliContext;

#[derive(Debug, Clone, Args)]
pub struct PlayCommand {
    /// Path to the entry track or project file (overrides config)
    #[arg(long = "input")]
    pub input: Option<PathBuf>,

    /// Output directory for generated assets (overrides config)
    #[arg(long = "output")]
    pub output: Option<PathBuf>,

    /// Preferred output format
    #[arg(long, value_enum)]
    pub format: Option<AudioFormat>,

    /// PCM bit depth
    #[arg(long = "bit-depth", value_enum)]
    pub bit_depth: Option<AudioBitDepth>,

    /// Channel count (mono or stereo)
    #[arg(long, value_enum)]
    pub channels: Option<AudioChannels>,

    /// Override sample rate
    #[arg(long = "sample-rate")]
    pub sample_rate: Option<u32>,

    /// Resampling quality strategy
    #[arg(long = "resample-quality", value_enum)]
    pub resample_quality: Option<ResampleQuality>,

    /// Enable live mode with watch + crossfade
    #[arg(long)]
    pub live: bool,

    /// Crossfade duration in milliseconds
    #[arg(long = "crossfade-ms")]
    pub crossfade_ms: Option<u64>,

    /// Mute the audio output
    #[arg(long)]
    pub quiet: bool,

    /// Volume level (0.0 to 1.0)
    #[arg(long)]
    pub volume: Option<f32>,
}

pub async fn execute(command: PlayCommand, ctx: &CliContext) -> Result<()> {
    let logger = ctx.logger();
    let cwd = std::env::current_dir()?;
    let config = AppConfig::load(&cwd)?;

    // Auto-load sample banks for native builds
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::engine::audio::samples;
        logger.info("Loading sample banks...");
        if let Err(e) = samples::auto_load_banks() {
            logger.warn(format!("Failed to auto-load banks: {}", e));
        }
    }

    let entry_path = command
        .input
        .clone()
        .unwrap_or_else(|| config.entry_path(&cwd));
    let output_root = command
        .output
        .clone()
        .unwrap_or_else(|| config.output_path(&cwd));

    let audio_format = command.format.unwrap_or_else(|| config.audio_format());
    let bit_depth = command
        .bit_depth
        .unwrap_or_else(|| config.audio_bit_depth());
    let channels = command.channels.unwrap_or_else(|| config.audio_channels());
    let sample_rate = command.sample_rate.unwrap_or_else(|| config.sample_rate());
    let resample_quality = command
        .resample_quality
        .unwrap_or_else(|| config.resample_quality());
    let crossfade_ms = command
        .crossfade_ms
        .unwrap_or_else(|| config.crossfade_ms());
    let live_mode = command.live;

    logger.info(format!(
        "Using entry={} output={} format={:?} bit_depth={} channels={} sample_rate={} resample={}",
        entry_path.display(),
        output_root.display(),
        audio_format,
        bit_depth.bits(),
        channels.count(),
        sample_rate,
        resample_quality
    ));

    fs::create_dir_all(&output_root)?;

    let build_request = BuildRequest {
        entry_path: entry_path.clone(),
        output_root: output_root.clone(),
        audio_formats: vec![audio_format],
        bit_depth,
        channels,
        resample_quality,
        sample_rate,
        bpm: config.audio.bpm,
    };

    let builder = ProjectBuilder::new(logger.clone());
    let service = LivePlayService::new(logger.clone(), builder)?;

    let volume = if command.quiet {
        0.0
    } else {
        let vol = command.volume.unwrap_or(1.0);
        if vol < 0.0 || vol > 1.0 {
            logger.error("Volume must be between 0.0 and 1.0");
            return Err(anyhow::anyhow!("Invalid volume value: {}", vol));
        }
        vol
    };

    let request = LivePlayRequest {
        build: build_request,
        live_mode,
        crossfade_ms,
        volume,
    };

    service.run(request).await
}
