#![cfg(feature = "cli")]

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};
use crate::language::syntax::ast::Statement;
use crate::tools::logger::Logger;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::services::build::outputs::audio::helpers::calculate_rms;
use crate::services::build::outputs::audio::writer::write_wav;

#[derive(Debug, Clone)]
pub struct AudioRenderSummary {
    pub path: PathBuf,
    pub format: AudioFormat,
    pub bit_depth: AudioBitDepth,
    pub rms: f32,
    pub render_time: Duration,
    pub audio_length: Duration,
}

#[derive(Debug, Clone)]
pub struct MultiFormatRenderSummary {
    pub primary_path: PathBuf,
    pub primary_format: AudioFormat,
    pub exported_formats: Vec<(AudioFormat, PathBuf)>,
    pub bit_depth: AudioBitDepth,
    pub rms: f32,
    pub render_time: Duration,
    pub audio_length: Duration,
}

#[derive(Clone)]
pub struct AudioBuilder {
    _logger: Arc<Logger>,
}

impl AudioBuilder {
    pub fn new(
        _log_writer: crate::services::build::outputs::logs::LogWriter,
        logger: Arc<Logger>,
    ) -> Self {
        Self { _logger: logger }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_all_formats(
        &self,
        statements: &[Statement],
        _entry_path: &Path,
        output_root: &Path,
        module_name: &str,
        requested_formats: &[AudioFormat],
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
        sample_rate: u32,
        _resample: ResampleQuality,
        _bpm: f32,
    ) -> Result<MultiFormatRenderSummary> {
        let start = Instant::now();

        // Pick primary format
        let primary_fmt = if requested_formats.is_empty() {
            AudioFormat::Wav
        } else {
            requested_formats[0]
        };

        let audio_summary = self.render(
            statements,
            _entry_path,
            output_root,
            module_name,
            primary_fmt,
            requested_bit_depth,
            channels,
            sample_rate,
            ResampleQuality::Sinc24,
        )?;

        let exported = vec![(audio_summary.format, audio_summary.path.clone())];

        let total_time = start.elapsed();

        Ok(MultiFormatRenderSummary {
            primary_path: audio_summary.path,
            primary_format: audio_summary.format,
            exported_formats: exported,
            bit_depth: audio_summary.bit_depth,
            rms: audio_summary.rms,
            render_time: total_time,
            audio_length: audio_summary.audio_length,
        })
    }

    pub fn render(
        &self,
        statements: &[Statement],
        _entry_path: &Path,
        output_root: impl AsRef<Path>,
        module_name: &str,
        requested_format: AudioFormat,
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
        sample_rate: u32,
        _resample: ResampleQuality,
    ) -> Result<AudioRenderSummary> {
        use crate::engine::audio::interpreter::driver::AudioInterpreter;

        let mut interpreter = AudioInterpreter::new(sample_rate);
        // During offline rendering we must not emit prints to stdout/stderr immediately.
        // Schedule prints into the interpreter event list and (optionally) replay them
        // in realtime during the render so the user can see PRINT messages as if
        // the audio was playing.
        interpreter.suppress_print = true;

        // During offline rendering we schedule prints into the interpreter.events.logs
        // and write them to a sidecar `.printlog` file for later replay by the
        // live playback engine. We do NOT replay prints in real-time during the
        // build here to avoid duplicate prints when the live player replays the
        // same scheduled logs.

        let buffer = interpreter.interpret(statements)?;

        let output_root = output_root.as_ref();
        let audio_dir = output_root.join("audio");
        std::fs::create_dir_all(&audio_dir).with_context(|| {
            format!(
                "failed to create audio output directory: {}",
                audio_dir.display()
            )
        })?;

        let output_path = audio_dir.join(format!("{}.wav", module_name));

        let mut rms = 0.0f32;
        let audio_length = if buffer.is_empty() {
            Duration::from_secs(0)
        } else {
            rms = calculate_rms(&buffer);
            let channel_count = channels.count() as usize;
            let frames = if channel_count == 0 {
                0
            } else {
                buffer.len() / channel_count
            };
            if sample_rate == 0 {
                Duration::from_secs(0)
            } else {
                Duration::from_secs_f64(frames as f64 / sample_rate as f64)
            }
        };

        if !buffer.is_empty() {
            let applied = write_wav(
                &output_path,
                &buffer,
                sample_rate,
                requested_bit_depth,
                channels,
            )?;

            // Write scheduled print events sidecar for live playback to consume.
            let log_path = output_path.with_file_name(format!("{}.printlog", module_name));
            if !interpreter.events.logs.is_empty() {
                if let Ok(mut f) = std::fs::File::create(&log_path) {
                    use std::io::Write;
                    for (t, msg) in &interpreter.events.logs {
                        // time in seconds (float) TAB message NEWLINE
                        let _ = writeln!(f, "{:.6}\t{}", t, msg);
                    }
                }
            }

            Ok(AudioRenderSummary {
                path: output_path,
                format: requested_format,
                bit_depth: applied,
                rms,
                render_time: Duration::from_secs(0),
                audio_length,
            })
        } else {
            Ok(AudioRenderSummary {
                path: output_path,
                format: requested_format,
                bit_depth: requested_bit_depth,
                rms: 0.0,
                render_time: Duration::from_secs(0),
                audio_length,
            })
        }
    }
}
