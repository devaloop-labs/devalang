#![cfg(feature = "cli")]

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::audio::settings::AudioFormat;
use crate::platform::config::AppConfig;
use crate::services::build::pipeline::{BuildRequest, ProjectBuilder};
use crate::tools::cli::state::CliContext;

#[derive(Debug, Clone, Args)]
pub struct BuildCommand {
    /// Path to the .deva file to build
    #[arg(long, default_value = "./")]
    pub path: String,

    /// Audio formats to export (e.g., "wav mid mp3")
    /// Overrides config file if provided
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    pub formats: Option<Vec<String>>,
}

impl BuildCommand {
    pub async fn execute(&self, ctx: &CliContext) -> Result<()> {
        let logger = ctx.logger();
        logger.action("Building project...");

        // Load config
        let current_dir = std::env::current_dir()?;
        let config = AppConfig::load(&current_dir)?;

        // Determine formats: CLI override or config
        let formats = if let Some(ref cli_formats) = self.formats {
            logger.info(format!("Using CLI formats: {:?}", cli_formats));
            cli_formats
                .iter()
                .filter_map(|s| AudioFormat::from_str(s))
                .collect::<Vec<_>>()
        } else {
            let config_formats = config.audio_formats();
            logger.info(format!("Using config formats: {:?}", config_formats));
            config_formats
        };

        if formats.is_empty() {
            anyhow::bail!("No valid audio formats specified");
        }

        // Resolve entry path
        let entry_path = PathBuf::from(&self.path);
        let entry_path = if entry_path.is_dir() {
            entry_path.join("index.deva")
        } else {
            entry_path
        };

        if !entry_path.exists() {
            anyhow::bail!("Entry file not found: {}", entry_path.display());
        }

        // Create build request
        let output_root = current_dir.join(&config.paths.output);
        let request = BuildRequest {
            entry_path: entry_path.clone(),
            output_root,
            audio_formats: formats,
            bit_depth: config.audio_bit_depth(),
            channels: config.audio_channels(),
            resample_quality: config.resample_quality(),
            sample_rate: config.sample_rate(),
            bpm: config.audio.bpm,
        };

        // Build project
        let builder = ProjectBuilder::new(logger.clone());
        let artifacts = builder.build(&request)?;

        // Log results
        logger.success(format!(
            "Build complete! Exported {} format(s):",
            artifacts.exported_formats.len()
        ));

        for (format, path) in &artifacts.exported_formats {
            logger.info(format!("  - {:?}: {}", format, path.display()));
        }

        logger.watch(format!(
            "Total build time: {:.1} ms (audio: {:.1} ms)",
            artifacts.total_duration.as_secs_f64() * 1000.0,
            artifacts.audio_render_time.as_secs_f64() * 1000.0
        ));

        Ok(())
    }
}
