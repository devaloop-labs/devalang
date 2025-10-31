#![cfg(feature = "cli")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};
use crate::language::syntax::ast::Statement;
use crate::language::syntax::parser::driver::SimpleParser;
use crate::tools::logger::Logger;

use super::outputs::ast::AstBuilder;
use super::outputs::audio::builder::AudioBuilder;
use super::outputs::logs::LogWriter;

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub entry_path: PathBuf,
    pub output_root: PathBuf,
    pub audio_formats: Vec<AudioFormat>,
    pub bit_depth: AudioBitDepth,
    pub channels: AudioChannels,
    pub resample_quality: ResampleQuality,
    pub sample_rate: u32,
    pub bpm: f32,
}

#[derive(Debug, Clone)]
pub struct BuildArtifacts {
    pub primary_format: AudioFormat,
    pub exported_formats: Vec<(AudioFormat, PathBuf)>,
    pub bit_depth: AudioBitDepth,
    pub channels: AudioChannels,
    pub resample_quality: ResampleQuality,
    pub sample_rate: u32,
    pub module_name: String,
    pub statements: Vec<Statement>,
    pub ast_path: PathBuf,
    pub primary_audio_path: PathBuf,
    pub rms: f32,
    pub audio_render_time: Duration,
    pub audio_length: Duration,
    pub total_duration: Duration,
}

#[derive(Clone)]
pub struct ProjectBuilder {
    logger: Arc<Logger>,
    ast_builder: AstBuilder,
    audio_builder: AudioBuilder,
    log_writer: LogWriter,
}

impl ProjectBuilder {
    pub fn new(logger: Arc<Logger>) -> Self {
        let log_writer = LogWriter::new();
        let audio_logger = logger.clone();
        Self {
            logger,
            ast_builder: AstBuilder::new(),
            audio_builder: AudioBuilder::new(log_writer, audio_logger),
            log_writer,
        }
    }

    pub fn build(&self, request: &BuildRequest) -> Result<BuildArtifacts> {
        let build_start = Instant::now();
        self.logger.action(format!(
            "Building module from {}",
            request.entry_path.display()
        ));

        let statements = self.parse(&request.entry_path)?;
        let module_name = module_name_from_path(&request.entry_path);

        let ast_path = self
            .ast_builder
            .write(&statements, &request.output_root, &module_name)?;

        // Render audio output in all requested formats
        use super::outputs::audio::builder::MultiFormatRenderSummary;
        let MultiFormatRenderSummary {
            primary_path,
            primary_format,
            exported_formats,
            bit_depth,
            rms,
            render_time: audio_render_time,
            audio_length,
        } = self.audio_builder.render_all_formats(
            &statements,
            &request.entry_path,
            &request.output_root,
            &module_name,
            &request.audio_formats,
            request.bit_depth,
            request.channels,
            request.sample_rate,
            request.resample_quality,
            request.bpm,
        )?;

        // Clear logs before writing new entries
        self.log_writer.clear(&request.output_root)?;

        // Append build summary to logs
        let formats_str = exported_formats
            .iter()
            .map(|(fmt, _)| format!("{:?}", fmt))
            .collect::<Vec<_>>()
            .join(", ");

        self.log_writer.append(
            &request.output_root,
            &format!(
                "Module '{}' built with {} statement(s); exported formats: [{}] ({} bits, {} ch, {:?})",
                module_name,
                statements.len(),
                formats_str,
                bit_depth.bits(),
                request.channels.count(),
                request.resample_quality
            ),
        )?;

        let total_duration = build_start.elapsed();
        self.logger.watch(format!(
            "Build complete in {:.1} ms (audio regen {:.1} ms)",
            total_duration.as_secs_f64() * 1000.0,
            audio_render_time.as_secs_f64() * 1000.0
        ));

        Ok(BuildArtifacts {
            primary_format,
            exported_formats,
            bit_depth,
            channels: request.channels,
            resample_quality: request.resample_quality,
            sample_rate: request.sample_rate,
            module_name,
            statements,
            ast_path,
            primary_audio_path: primary_path,
            rms,
            audio_render_time,
            audio_length,
            total_duration,
        })
    }

    fn parse(&self, entry: impl AsRef<Path>) -> Result<Vec<Statement>> {
        SimpleParser::parse_file(entry)
    }
}

fn module_name_from_path(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "module".to_string())
}
