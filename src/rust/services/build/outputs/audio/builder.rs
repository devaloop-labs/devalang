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
        bpm: f32,
    ) -> Result<MultiFormatRenderSummary> {
        let start = Instant::now();

        let formats = if requested_formats.is_empty() {
            vec![AudioFormat::Wav]
        } else {
            requested_formats.to_vec()
        };

        let mut summaries = Vec::new();
        for format in formats {
            summaries.push(self.render(
                statements,
                _entry_path,
                output_root,
                module_name,
                format,
                requested_bit_depth,
                channels,
                sample_rate,
                ResampleQuality::Sinc24,
                bpm,
            )?);
        }

        let total_time = start.elapsed();
        let primary = summaries
            .first()
            .cloned()
            .context("no audio formats were rendered")?;
        let exported = summaries
            .iter()
            .map(|summary| (summary.format, summary.path.clone()))
            .collect();

        Ok(MultiFormatRenderSummary {
            primary_path: primary.path,
            primary_format: primary.format,
            exported_formats: exported,
            bit_depth: primary.bit_depth,
            rms: primary.rms,
            render_time: total_time,
            audio_length: primary.audio_length,
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
        bpm: f32,
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

        // module_name may be a relative path like "demos/hello-sound/index"
        // Split it into directory part and file part
        let module_path = Path::new(module_name);
        let (output_subdir, file_name) = if let Some(parent) = module_path.parent() {
            let parent_str = parent.to_string_lossy();
            if parent_str.is_empty() || parent_str == "." {
                (String::new(), module_name.to_string())
            } else {
                let file_name = module_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| module_name.to_string());
                (parent_str.to_string(), file_name)
            }
        } else {
            (String::new(), module_name.to_string())
        };

        let full_audio_dir = if output_subdir.is_empty() {
            audio_dir.clone()
        } else {
            audio_dir.join(&output_subdir)
        };

        std::fs::create_dir_all(&full_audio_dir).with_context(|| {
            format!(
                "failed to create audio output directory: {}",
                full_audio_dir.display()
            )
        })?;

        let output_path = full_audio_dir.join(format!(
            "{}.{}",
            file_name,
            requested_format.file_extension()
        ));

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

        let applied = match requested_format {
            AudioFormat::Wav => {
                if !buffer.is_empty() {
                    write_wav(
                        &output_path,
                        &buffer,
                        sample_rate,
                        requested_bit_depth,
                        channels,
                    )?
                } else {
                    // Even if buffer is empty, create an empty WAV file
                    write_wav(
                        &output_path,
                        &[],
                        sample_rate,
                        requested_bit_depth,
                        channels,
                    )?
                }
            }
            AudioFormat::Mid => {
                let midi_bpm = if interpreter.bpm.is_finite() && interpreter.bpm > 0.0 {
                    interpreter.bpm
                } else {
                    bpm
                };
                crate::engine::audio::midi::export_midi_file(
                    &interpreter.events.events,
                    &output_path,
                    midi_bpm,
                )?;
                requested_bit_depth
            }
            AudioFormat::Mp3 | AudioFormat::Flac => {
                anyhow::bail!(
                    "{:?} export is not implemented in the CLI build pipeline yet",
                    requested_format
                );
            }
        };

        // Write scheduled print events sidecar for live playback to consume.
        let log_path = output_path.with_file_name(format!("{}.printlog", file_name));
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
    }
}
