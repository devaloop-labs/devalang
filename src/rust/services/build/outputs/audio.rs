#![cfg(feature = "cli")]
use super::logs::LogWriter;
use crate::engine::audio::mixer::{AudioMixer, MASTER_INSERT, SampleBuffer};
use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};
use crate::language::addons::registry::BankRegistry;
use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use crate::tools::logger::Logger;
use anyhow::{Context, Result, anyhow};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
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
#[derive(Debug, Clone)]
enum RenderJob {
    Sample {
        start: f32,
        duration: f32,
        sample: SampleBuffer,
        insert: String,
    },
}

#[derive(Debug)]
struct RenderPlan {
    jobs: Vec<RenderJob>,
    inserts: HashMap<String, Option<String>>,
    total_duration: f32,
}

#[derive(Debug)]
struct SpawnPlan {
    jobs: Vec<RenderJob>,
    cursor: f32,
    max_time: f32,
}
#[derive(Debug, Clone)]
pub struct AudioBuilder {
    log_writer: LogWriter,
    logger: Arc<Logger>,
    resample_warning_emitted: Arc<std::sync::atomic::AtomicBool>,
}
impl AudioBuilder {
    pub fn new(log_writer: LogWriter, logger: Arc<Logger>) -> Self {
        Self {
            log_writer,
            logger,
            resample_warning_emitted: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        statements: &[Statement],
        entry_path: &Path,
        output_root: impl AsRef<Path>,
        module_name: &str,
        requested_format: AudioFormat,
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
        sample_rate: u32,
        resample: ResampleQuality,
    ) -> Result<AudioRenderSummary> {
        let render_start = Instant::now();
        let output_root = output_root.as_ref();
        let audio_dir = output_root.join("audio");
        std::fs::create_dir_all(&audio_dir).with_context(|| {
            format!(
                "failed to create audio output directory: {}",
                audio_dir.display()
            )
        })?;

        // USE NEW AUDIO INTERPRETER FOR SYNTH RENDERING
        use crate::engine::audio::interpreter::driver::AudioInterpreter;
        let mut interpreter = AudioInterpreter::new(sample_rate);
        let buffer = interpreter.interpret(statements)?;

        // If interpreter produced audio, use it
        if !buffer.is_empty() {
            let output_path = audio_dir.join(format!("{}.wav", module_name));
            let rms = calculate_rms(&buffer);
            self.logger.debug(format!("Audio RMS: {:.4}", rms));

            let applied_bit_depth = self.write_wav(
                &output_path,
                &buffer,
                sample_rate,
                requested_bit_depth,
                channels,
            )?;

            let render_time = render_start.elapsed();
            let channel_count = channels.count() as usize;
            let frames = if channel_count == 0 {
                0
            } else {
                buffer.len() / channel_count
            };
            let audio_length = if sample_rate == 0 {
                Duration::from_secs(0)
            } else {
                Duration::from_secs_f64(frames as f64 / sample_rate as f64)
            };

            self.logger.success(format!(
                "Rendered audio output -> {} ({:.1} ms)",
                output_path.display(),
                render_time.as_secs_f64() * 1000.0
            ));

            return Ok(AudioRenderSummary {
                path: output_path,
                format: AudioFormat::Wav,
                bit_depth: applied_bit_depth,
                rms,
                render_time,
                audio_length,
            });
        }

        // FALLBACK TO OLD SYSTEM IF NO AUDIO GENERATED
        if !matches!(resample, ResampleQuality::Sinc24)
            && !self
                .resample_warning_emitted
                .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            self.logger.debug(format!(
                "Resampling quality '{}' is noted but not yet applied (procedural synthesis).",
                resample
            ));
        }
        let (target_format, extension) = match requested_format {
            AudioFormat::Wav => (AudioFormat::Wav, "wav"),
            other => {
                self.logger.warn(format!(
                    "Audio export {:?} is not implemented yet. Falling back to WAV",
                    other
                ));
                (AudioFormat::Wav, "wav")
            }
        };
        let output_path = audio_dir.join(format!("{}.{}", module_name, extension));
        let channel_count = channels.count() as usize;
        let base_dir = entry_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let project_root = find_project_root(entry_path);
        let RenderPlan {
            jobs,
            inserts,
            total_duration,
        } = self.plan_jobs(statements, &base_dir, &project_root)?;
        let total_samples = (total_duration * sample_rate as f32).ceil() as usize;
        let mut mixer = AudioMixer::new(sample_rate, channel_count);
        for (insert, parent) in inserts.iter() {
            if insert != MASTER_INSERT {
                mixer.register_insert(insert.clone(), parent.as_deref());
            }
        }
        mixer.ensure_master_frames(total_samples);
        for job in jobs {
            match job {
                RenderJob::Sample {
                    start,
                    duration,
                    sample,
                    insert,
                } => {
                    if sample_rate == 0 {
                        continue;
                    }
                    let start_frame = if start <= 0.0 {
                        0
                    } else {
                        (start * sample_rate as f32).floor() as usize
                    };
                    mixer.mix_sample(&insert, start_frame, duration, &sample);
                }
            }
        }
        let mut buffer = mixer.into_master_buffer(total_samples);
        normalize(&mut buffer);
        let rms = calculate_rms(&buffer);
        self.logger.debug(format!("Audio RMS: {:.4}", rms));
        let applied_bit_depth = self.write_wav(
            &output_path,
            &buffer,
            sample_rate,
            requested_bit_depth,
            channels,
        )?;
        self.log_writer.append(
            output_root,
            &format!(
                "Rendered module '{}' to {} ({:?}, {} bits, {} ch, {:?})",
                module_name,
                output_path.display(),
                target_format,
                applied_bit_depth.bits(),
                channels.count(),
                resample
            ),
        )?;
        let render_time = render_start.elapsed();
        let frames = if channel_count == 0 {
            0
        } else {
            buffer.len() / channel_count
        };
        let audio_length = if sample_rate == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(frames as f64 / sample_rate as f64)
        };
        self.logger.success(format!(
            "Rendered audio output -> {} ({:.1} ms)",
            output_path.display(),
            render_time.as_secs_f64() * 1000.0
        ));
        Ok(AudioRenderSummary {
            path: output_path,
            format: target_format,
            bit_depth: applied_bit_depth,
            rms,
            render_time,
            audio_length,
        })
    }

    /// Render audio and export to multiple formats
    #[allow(clippy::too_many_arguments)]
    pub fn render_all_formats(
        &self,
        statements: &[Statement],
        _entry_path: &Path,
        output_root: impl AsRef<Path>,
        module_name: &str,
        requested_formats: &[AudioFormat],
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
        sample_rate: u32,
        _resample: ResampleQuality,
        bpm: f32,
    ) -> Result<MultiFormatRenderSummary> {
        let render_start = Instant::now();
        let output_root = output_root.as_ref();
        let audio_dir = output_root.join("audio");
        std::fs::create_dir_all(&audio_dir).with_context(|| {
            format!(
                "failed to create audio output directory: {}",
                audio_dir.display()
            )
        })?;

        // USE NEW AUDIO INTERPRETER FOR SYNTH RENDERING
        use crate::engine::audio::interpreter::driver::AudioInterpreter;
        let mut interpreter = AudioInterpreter::new(sample_rate);
        let buffer = interpreter.interpret(statements)?;

        let rms = calculate_rms(&buffer);
        self.logger.debug(format!("Audio RMS: {:.4}", rms));

        let channel_count = channels.count() as usize;
        let frames = if channel_count == 0 {
            0
        } else {
            buffer.len() / channel_count
        };
        let audio_length = if sample_rate == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(frames as f64 / sample_rate as f64)
        };

        // Export to all requested formats
        let mut exported_formats = Vec::new();
        let mut primary_path = PathBuf::new();
        let mut primary_format = AudioFormat::Wav;
        let mut applied_bit_depth = requested_bit_depth;

        for (idx, &format) in requested_formats.iter().enumerate() {
            match format {
                AudioFormat::Wav => {
                    let output_path = audio_dir.join(format!("{}.wav", module_name));
                    applied_bit_depth = self.write_wav(
                        &output_path,
                        &buffer,
                        sample_rate,
                        requested_bit_depth,
                        channels,
                    )?;
                    self.logger.success(format!(
                        "✅ Exported WAV: {} ({} bits)",
                        output_path.display(),
                        applied_bit_depth.bits()
                    ));
                    exported_formats.push((AudioFormat::Wav, output_path.clone()));
                    if idx == 0 {
                        primary_path = output_path;
                        primary_format = AudioFormat::Wav;
                    }
                }
                AudioFormat::Mp3 => {
                    let output_path = audio_dir.join(format!("{}.mp3", module_name));
                    applied_bit_depth = self.write_mp3(
                        &output_path,
                        &buffer,
                        sample_rate,
                        requested_bit_depth,
                        channels,
                    )?;
                    self.logger.success(format!(
                        "🎵 Exported MP3: {} ({} bits equivalent)",
                        output_path.display(),
                        applied_bit_depth.bits()
                    ));
                    exported_formats.push((AudioFormat::Mp3, output_path.clone()));
                    if idx == 0 {
                        primary_path = output_path;
                        primary_format = AudioFormat::Mp3;
                    }
                }
                AudioFormat::Flac => {
                    let output_path = audio_dir.join(format!("{}.flac", module_name));
                    applied_bit_depth = self.write_flac(
                        &output_path,
                        &buffer,
                        sample_rate,
                        requested_bit_depth,
                        channels,
                    )?;
                    self.logger.success(format!(
                        "🎼 Exported FLAC: {} ({} bits)",
                        output_path.display(),
                        applied_bit_depth.bits()
                    ));
                    exported_formats.push((AudioFormat::Flac, output_path.clone()));
                    if idx == 0 {
                        primary_path = output_path;
                        primary_format = AudioFormat::Flac;
                    }
                }
                AudioFormat::Mid => {
                    // Export MIDI if we have audio events
                    let output_path = audio_dir.join(format!("{}.mid", module_name));

                    // Get events from interpreter
                    if !interpreter.events().events.is_empty() {
                        use crate::engine::audio::midi::export_midi_file;
                        if let Err(e) =
                            export_midi_file(&interpreter.events().events, &output_path, bpm)
                        {
                            self.logger.warn(format!("Failed to export MIDI: {}", e));
                        } else {
                            self.logger
                                .success(format!("🎹 Exported MIDI: {}", output_path.display()));
                            exported_formats.push((AudioFormat::Mid, output_path));
                        }
                    } else {
                        self.logger.warn(
                            "No MIDI events to export (no synth play/chord statements)".to_string(),
                        );
                    }
                }
            }
        }

        let render_time = render_start.elapsed();

        self.logger.success(format!(
            "Rendered {} format(s) in {:.1} ms",
            exported_formats.len(),
            render_time.as_secs_f64() * 1000.0
        ));

        Ok(MultiFormatRenderSummary {
            primary_path,
            primary_format,
            exported_formats,
            bit_depth: applied_bit_depth,
            rms,
            render_time,
            audio_length,
        })
    }

    fn plan_jobs(
        &self,
        statements: &[Statement],
        base_dir: &Path,
        project_root: &Path,
    ) -> Result<RenderPlan> {
        let mut samples = HashMap::<String, SampleBuffer>::new();
        let mut sample_cache = HashMap::<PathBuf, SampleBuffer>::new();
        let mut banks = BankRegistry::new();
        let mut inserts = HashMap::<String, Option<String>>::new();
        inserts.insert(MASTER_INSERT.to_string(), None);
        let mut alias_to_insert = HashMap::<String, String>::new();
        alias_to_insert.insert(MASTER_INSERT.to_string(), MASTER_INSERT.to_string());

        // Store groups for spawn execution
        let mut groups = HashMap::<String, Vec<Statement>>::new();

        // Store patterns for spawn execution
        let mut patterns = HashMap::<String, (String, String)>::new(); // name -> (target, pattern_string)

        // Variable table with scope management
        use crate::language::scope::{BindingType, VariableTable};
        let mut variables = VariableTable::new();

        let mut jobs = Vec::new();
        let mut tempo = 120.0_f32;
        let mut cursor = 0.0_f32;
        let mut max_time = 0.0_f32;

        // First pass: register groups and patterns
        for statement in statements {
            match &statement.kind {
                StatementKind::Group { name, body } => {
                    groups.insert(name.clone(), body.clone());
                }
                StatementKind::Pattern { name, target } => {
                    if let Value::String(pattern_str) = &statement.value {
                        if let Some(target_entity) = target {
                            patterns
                                .insert(name.clone(), (target_entity.clone(), pattern_str.clone()));
                        } else {
                            self.logger
                                .warn(format!("Pattern '{}' has no target specified", name));
                        }
                    }
                }
                _ => {}
            }
        }

        // Second pass: process statements
        for statement in statements {
            match &statement.kind {
                StatementKind::Tempo => {
                    if let Value::Number(value) = statement.value {
                        if value > 0.0 {
                            tempo = value;
                        }
                    }
                }
                StatementKind::Let { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Let);
                        self.logger.debug(format!("let {} = {:?}", name, val));
                    }
                }
                StatementKind::Var { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Var);
                        self.logger.debug(format!("var {} = {:?}", name, val));
                    }
                }
                StatementKind::Const { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Const);
                        self.logger.debug(format!("const {} = {:?}", name, val));
                    }
                }
                StatementKind::Load { source, alias } => {
                    let resolved_path = resolve_sample_reference(base_dir, source);
                    match self.load_sample_cached(&mut sample_cache, resolved_path.clone()) {
                        Ok(sample) => {
                            let insert_name = register_route_for_alias(
                                &mut inserts,
                                &mut alias_to_insert,
                                alias,
                                "sample",
                                Some(MASTER_INSERT),
                            );
                            samples.insert(alias.to_string(), sample.clone());
                            self.logger.info(format!(
                                "Loaded sample '{}' into '{}' from {}",
                                alias,
                                insert_name,
                                resolved_path.display()
                            ));
                        }
                        Err(err) => {
                            self.logger.error(format!(
                                "Failed to load sample '{}' (alias '{}'): {}",
                                resolved_path.display(),
                                alias,
                                err
                            ));
                        }
                    }
                }
                StatementKind::Bank { name, alias } => {
                    let alias_name = alias.clone().unwrap_or_else(|| default_bank_alias(name));
                    let insert_name = register_route_for_alias(
                        &mut inserts,
                        &mut alias_to_insert,
                        &alias_name,
                        "bank",
                        Some(MASTER_INSERT),
                    );
                    alias_to_insert
                        .entry(name.clone())
                        .or_insert_with(|| insert_name.clone());
                    match banks.register_bank(alias_name.clone(), name, project_root, base_dir) {
                        Ok(bank) => {
                            let count = bank.trigger_count();
                            self.logger.info(format!(
                                "Registered bank '{}' as '{}' -> insert '{}' ({} trigger{})",
                                name,
                                alias_name,
                                insert_name,
                                count,
                                if count == 1 { "" } else { "s" }
                            ));
                        }
                        Err(err) => {
                            self.logger
                                .error(format!("Failed to register bank '{}': {}", name, err));
                        }
                    }
                }
                StatementKind::Trigger {
                    entity,
                    duration,
                    effects,
                } => {
                    // Resolve entity through variables (e.g., if entity is "kick" and kick = drums.kick)
                    let resolved_entity = if let Some(var_value) = variables.get(entity) {
                        match self.resolve_value(&var_value, &variables) {
                            Value::Identifier(id) => id,
                            Value::String(s) => s,
                            _ => entity.clone(),
                        }
                    } else {
                        entity.clone()
                    };

                    let (target_alias, trigger_name) = split_trigger_entity(&resolved_entity);
                    let insert_name = alias_to_insert
                        .get(target_alias)
                        .cloned()
                        .unwrap_or_else(|| MASTER_INSERT.to_string());
                    let duration_seconds = duration_in_seconds(duration, tempo);
                    let is_auto = matches!(duration, DurationValue::Auto);
                    let start = cursor;
                    let mut resolved_sample = samples.get(target_alias).cloned();
                    if resolved_sample.is_none() {
                        if let Some(trigger) = trigger_name {
                            if let Some(path) = banks.resolve_trigger(target_alias, trigger) {
                                match self.load_sample_cached(&mut sample_cache, path.clone()) {
                                    Ok(sample) => {
                                        self.logger.debug(format!(
                                            "Trigger '{}.{}' resolved to {}",
                                            target_alias,
                                            trigger,
                                            path.display()
                                        ));
                                        resolved_sample = Some(sample);
                                    }
                                    Err(err) => {
                                        self.logger.error(format!(
                                            "Failed to load trigger '{}.{}' from {}: {}",
                                            target_alias,
                                            trigger,
                                            path.display(),
                                            err
                                        ));
                                    }
                                }
                            } else if banks.has_bank(target_alias) {
                                self.logger.error(format!(
                                    "Bank '{}' does not define trigger '{}'; rendering silence",
                                    target_alias, trigger
                                ));
                            } else {
                                self.logger.warn(format!(
                                    "Bank alias '{}' not registered; rendering silence",
                                    target_alias
                                ));
                            }
                        } else if !samples.contains_key(target_alias) {
                            self.logger.warn(format!(
                                "Unknown sample alias '{}'; rendering silence",
                                target_alias
                            ));
                        }
                    }
                    if let Some(mut sample) = resolved_sample {
                        // Apply effects if present
                        if let Some(fx) = effects {
                            let effect_count = if let Value::Map(m) = fx { m.len() } else { 0 };
                            if effect_count > 0 {
                                self.logger.debug(format!(
                                    "  🎛️  Applying {} effect(s) to trigger '{}'",
                                    effect_count, entity
                                ));

                                let mut buffer = sample.data_clone();
                                if let Err(e) = apply_trigger_effects(
                                    &mut buffer,
                                    fx,
                                    sample.sample_rate(),
                                    sample.channels(),
                                ) {
                                    self.logger.warn(format!(
                                        "Failed to apply effects to trigger '{}': {}",
                                        entity, e
                                    ));
                                } else {
                                    sample = sample.with_modified_data(buffer, None);
                                }
                            }
                        }

                        let sample_len = if sample.sample_rate() == 0 {
                            0.0
                        } else {
                            sample.frames() as f32 / sample.sample_rate() as f32
                        };
                        let (advance, playback_duration) = if let Some(value) = duration_seconds {
                            if is_auto {
                                (sample_len, sample_len)
                            } else {
                                (value, value.max(sample_len))
                            }
                        } else {
                            (sample_len, sample_len)
                        };
                        jobs.push(RenderJob::Sample {
                            start,
                            duration: playback_duration,
                            sample,
                            insert: insert_name.clone(),
                        });
                        cursor += advance;
                        max_time = max_time.max(start + playback_duration);
                    } else {
                        let duration_value =
                            duration_seconds.unwrap_or(beats_to_seconds(0.25, tempo));
                        cursor += duration_value;
                        max_time = max_time.max(start + duration_value);
                    }
                }
                StatementKind::Routing => {
                    if let Value::Map(route) = &statement.value {
                        if let Some(source_alias) = route.get("source").and_then(value_as_string) {
                            let target_alias = route.get("target").and_then(value_as_string);
                            let parent_insert = target_alias.as_ref().and_then(|alias| {
                                resolve_parent_for_alias(&mut inserts, &mut alias_to_insert, alias)
                            });
                            let parent_ref = parent_insert.as_ref().map(|s| s.as_str());
                            let source_insert = alias_to_insert
                                .get(&source_alias)
                                .cloned()
                                .unwrap_or_else(|| {
                                    register_route_for_alias(
                                        &mut inserts,
                                        &mut alias_to_insert,
                                        &source_alias,
                                        "bus",
                                        parent_ref,
                                    )
                                });
                            if source_insert == MASTER_INSERT {
                                self.logger.warn(format!(
                                    "Ignoring routing directive attempting to modify master: '{}'",
                                    source_alias
                                ));
                                continue;
                            }
                            inserts
                                .entry(source_insert.clone())
                                .and_modify(|entry| *entry = parent_insert.clone())
                                .or_insert(parent_insert.clone());
                            alias_to_insert.entry(source_alias).or_insert(source_insert);
                        }
                    }
                }
                StatementKind::Sleep => {
                    // Resolve value through variables first
                    let resolved = self.resolve_value(&statement.value, &variables);
                    let duration = match resolved {
                        Value::Duration(d) => {
                            duration_in_seconds(&d, tempo).unwrap_or(beats_to_seconds(0.25, tempo))
                        }
                        Value::Number(n) => n / 1_000.0, // Convert ms to seconds
                        _ => beats_to_seconds(0.25, tempo),
                    };
                    cursor += duration;
                    max_time = max_time.max(cursor);
                }
                StatementKind::Group { .. } => {
                    // Groups are registered in first pass, skip execution here
                }
                StatementKind::Spawn { name, .. } => {
                    // Try to execute group first
                    if let Some(body) = groups.get(name) {
                        self.logger
                            .debug(format!("Spawning group '{}' at {}s", name, cursor));
                        let spawn_start = cursor;

                        // Create child scope for group
                        use crate::language::scope::VariableTable;
                        let child_vars = VariableTable::with_parent(variables.clone());

                        // Process group statements recursively
                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            &mut samples,
                            &mut sample_cache,
                            &mut banks,
                            &mut inserts,
                            &mut alias_to_insert,
                            &groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;

                        // Merge jobs from spawn
                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);

                        self.logger.debug(format!(
                            "Group '{}' spawned from {}s to {}s (duration: {}s)",
                            name,
                            spawn_start,
                            cursor,
                            cursor - spawn_start
                        ));
                    } else if let Some((target, pattern_str)) = patterns.get(name) {
                        // Execute pattern
                        self.logger
                            .debug(format!("Spawning pattern '{}' at {}s", name, cursor));
                        let spawn_start = cursor;

                        // Parse pattern string (e.g. "x--- x--- x--- x---")
                        let pattern_chars: Vec<char> =
                            pattern_str.chars().filter(|c| !c.is_whitespace()).collect();
                        let step_count = pattern_chars.len() as f32;

                        if step_count > 0.0 {
                            // Calculate step duration: 4 beats divided by number of steps
                            let bar_duration = beats_to_seconds(4.0, tempo);
                            let step_duration = bar_duration / step_count;

                            for (i, ch) in pattern_chars.iter().enumerate() {
                                if *ch == 'x' || *ch == 'X' {
                                    let trigger_time = spawn_start + (i as f32 * step_duration);

                                    // Resolve the trigger
                                    let (target_alias, trigger_name) = split_trigger_entity(target);
                                    let insert_name = alias_to_insert
                                        .get(target_alias)
                                        .cloned()
                                        .unwrap_or_else(|| MASTER_INSERT.to_string());

                                    let mut resolved_sample = None;
                                    if let Some(trigger) = trigger_name {
                                        if let Some(path) =
                                            banks.resolve_trigger(target_alias, trigger)
                                        {
                                            match self
                                                .load_sample_cached(&mut sample_cache, path.clone())
                                            {
                                                Ok(sample) => {
                                                    resolved_sample = Some(sample);
                                                }
                                                Err(err) => {
                                                    self.logger.error(format!(
                                                        "Failed to load pattern trigger '{}.{}': {}",
                                                        target_alias, trigger, err
                                                    ));
                                                }
                                            }
                                        }
                                    }

                                    if let Some(sample) = resolved_sample {
                                        let sample_len = if sample.sample_rate() == 0 {
                                            0.0
                                        } else {
                                            sample.frames() as f32 / sample.sample_rate() as f32
                                        };

                                        jobs.push(RenderJob::Sample {
                                            start: trigger_time,
                                            duration: sample_len,
                                            sample,
                                            insert: insert_name,
                                        });

                                        max_time = max_time.max(trigger_time + sample_len);
                                    }
                                }
                            }

                            cursor = spawn_start + bar_duration;
                            max_time = max_time.max(cursor);

                            self.logger.debug(format!(
                                "Pattern '{}' spawned from {}s to {}s ({} steps)",
                                name, spawn_start, cursor, step_count
                            ));
                        }
                    } else {
                        self.logger.warn(format!(
                            "Unknown group or pattern '{}' in spawn statement",
                            name
                        ));
                    }
                }
                StatementKind::For {
                    variable,
                    iterable,
                    body,
                } => {
                    // Execute for loop: for i in [1, 2, 3]:
                    let loop_start = cursor;

                    // Resolve iterable
                    let resolved_iterable = self.resolve_value(iterable, &variables);
                    let items = match resolved_iterable {
                        Value::Array(arr) => arr,
                        Value::Number(n) => {
                            // If number, create range [0, 1, 2, ..., n-1]
                            (0..(n as i32)).map(|i| Value::Number(i as f32)).collect()
                        }
                        _ => {
                            self.logger
                                .warn(format!("For loop iterable must be array or number"));
                            continue;
                        }
                    };

                    self.logger.debug(format!(
                        "For loop with {} iterations at {}s",
                        items.len(),
                        cursor
                    ));

                    for item in items {
                        // Create child scope with iterator variable
                        use crate::language::scope::{BindingType, VariableTable};
                        let mut child_vars = VariableTable::with_parent(variables.clone());
                        child_vars.set_with_type(variable.clone(), item, BindingType::Let);

                        // Execute body
                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            &mut samples,
                            &mut sample_cache,
                            &mut banks,
                            &mut inserts,
                            &mut alias_to_insert,
                            &groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;

                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);
                    }

                    self.logger.debug(format!(
                        "For loop completed from {}s to {}s",
                        loop_start, cursor
                    ));
                }
                StatementKind::Loop { count, body } => {
                    // Execute loop: loop 3:
                    let loop_start = cursor;

                    // Resolve count
                    let resolved_count = self.resolve_value(count, &variables);
                    let iterations = match resolved_count {
                        Value::Number(n) => n as usize,
                        _ => {
                            self.logger.warn(format!("Loop count must be a number"));
                            continue;
                        }
                    };

                    self.logger
                        .debug(format!("Loop {} times at {}s", iterations, cursor));

                    for _ in 0..iterations {
                        // Create child scope
                        use crate::language::scope::VariableTable;
                        let child_vars = VariableTable::with_parent(variables.clone());

                        // Execute body
                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            &mut samples,
                            &mut sample_cache,
                            &mut banks,
                            &mut inserts,
                            &mut alias_to_insert,
                            &groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;

                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);
                    }

                    self.logger.debug(format!(
                        "Loop completed from {}s to {}s",
                        loop_start, cursor
                    ));
                }
                StatementKind::If {
                    condition,
                    body,
                    else_body,
                } => {
                    // Evaluate condition
                    let condition_result = self.evaluate_condition(condition, &variables);

                    let branch_to_execute = if condition_result {
                        body
                    } else if let Some(else_branch) = else_body {
                        else_branch
                    } else {
                        continue;
                    };

                    self.logger.debug(format!(
                        "If condition evaluated to {}, executing branch at {}s",
                        condition_result, cursor
                    ));

                    // Create child scope
                    use crate::language::scope::VariableTable;
                    let child_vars = VariableTable::with_parent(variables.clone());

                    // Execute branch
                    let spawn_plan = self.plan_jobs_internal(
                        branch_to_execute,
                        base_dir,
                        project_root,
                        &mut samples,
                        &mut sample_cache,
                        &mut banks,
                        &mut inserts,
                        &mut alias_to_insert,
                        &groups,
                        child_vars,
                        tempo,
                        cursor,
                    )?;

                    jobs.extend(spawn_plan.jobs);
                    cursor = spawn_plan.cursor;
                    max_time = max_time.max(spawn_plan.max_time);
                }
                _ => {}
            }
        }
        max_time = max_time.max(cursor);
        Ok(RenderPlan {
            jobs,
            inserts,
            total_duration: max_time,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn plan_jobs_internal(
        &self,
        statements: &[Statement],
        base_dir: &Path,
        project_root: &Path,
        samples: &mut HashMap<String, SampleBuffer>,
        sample_cache: &mut HashMap<PathBuf, SampleBuffer>,
        banks: &mut BankRegistry,
        inserts: &mut HashMap<String, Option<String>>,
        alias_to_insert: &mut HashMap<String, String>,
        groups: &HashMap<String, Vec<Statement>>,
        mut variables: crate::language::scope::VariableTable,
        mut tempo: f32,
        mut cursor: f32,
    ) -> Result<SpawnPlan> {
        let mut jobs = Vec::new();
        let mut max_time = cursor;

        use crate::language::scope::BindingType;

        for statement in statements {
            match &statement.kind {
                StatementKind::Tempo => {
                    if let Value::Number(value) = statement.value {
                        if value > 0.0 {
                            tempo = value;
                        }
                    }
                }
                StatementKind::Let { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Let);
                    }
                }
                StatementKind::Var { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Var);
                    }
                }
                StatementKind::Const { name, value } => {
                    if let Some(val) = value {
                        let resolved = self.resolve_value(val, &variables);
                        variables.set_with_type(name.clone(), resolved, BindingType::Const);
                    }
                }
                StatementKind::Trigger {
                    entity,
                    duration,
                    effects,
                } => {
                    // Resolve entity through variables (e.g., if entity is "kick" and kick = drums.kick)
                    let resolved_entity = if let Some(var_value) = variables.get(entity) {
                        match self.resolve_value(&var_value, &variables) {
                            Value::Identifier(id) => id,
                            Value::String(s) => s,
                            _ => entity.clone(),
                        }
                    } else {
                        entity.clone()
                    };

                    let (target_alias, trigger_name) = split_trigger_entity(&resolved_entity);
                    let insert_name = alias_to_insert
                        .get(target_alias)
                        .cloned()
                        .unwrap_or_else(|| MASTER_INSERT.to_string());
                    let duration_seconds = duration_in_seconds(duration, tempo);
                    let is_auto = matches!(duration, DurationValue::Auto);
                    let start = cursor;
                    let mut resolved_sample = samples.get(target_alias).cloned();
                    if resolved_sample.is_none() {
                        if let Some(trigger) = trigger_name {
                            if let Some(path) = banks.resolve_trigger(target_alias, trigger) {
                                match self.load_sample_cached(sample_cache, path.clone()) {
                                    Ok(sample) => {
                                        self.logger.debug(format!(
                                            "Trigger '{}.{}' resolved to {}",
                                            target_alias,
                                            trigger,
                                            path.display()
                                        ));
                                        resolved_sample = Some(sample);
                                    }
                                    Err(err) => {
                                        self.logger.error(format!(
                                            "Failed to load trigger '{}.{}' from {}: {}",
                                            target_alias,
                                            trigger,
                                            path.display(),
                                            err
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    if let Some(mut sample) = resolved_sample {
                        // Apply effects if present
                        if let Some(fx) = effects {
                            let effect_count = if let Value::Map(m) = fx { m.len() } else { 0 };
                            if effect_count > 0 {
                                self.logger.debug(format!(
                                    "  🎛️  Applying {} effect(s) to trigger '{}'",
                                    effect_count, entity
                                ));

                                let mut buffer = sample.data_clone();
                                if let Err(e) = apply_trigger_effects(
                                    &mut buffer,
                                    fx,
                                    sample.sample_rate(),
                                    sample.channels(),
                                ) {
                                    self.logger.warn(format!(
                                        "Failed to apply effects to trigger '{}': {}",
                                        entity, e
                                    ));
                                } else {
                                    sample = sample.with_modified_data(buffer, None);
                                }
                            }
                        }

                        let sample_len = if sample.sample_rate() == 0 {
                            0.0
                        } else {
                            sample.frames() as f32 / sample.sample_rate() as f32
                        };
                        let (advance, playback_duration) = if let Some(value) = duration_seconds {
                            if is_auto {
                                (sample_len, sample_len)
                            } else {
                                (value, value.max(sample_len))
                            }
                        } else {
                            (sample_len, sample_len)
                        };
                        jobs.push(RenderJob::Sample {
                            start,
                            duration: playback_duration,
                            sample,
                            insert: insert_name.clone(),
                        });
                        cursor += advance;
                        max_time = max_time.max(start + playback_duration);
                    } else {
                        let duration_value =
                            duration_seconds.unwrap_or(beats_to_seconds(0.25, tempo));
                        cursor += duration_value;
                        max_time = max_time.max(start + duration_value);
                    }
                }
                StatementKind::Sleep => {
                    // Resolve value through variables first
                    let resolved = self.resolve_value(&statement.value, &variables);
                    let duration = match resolved {
                        Value::Duration(d) => {
                            duration_in_seconds(&d, tempo).unwrap_or(beats_to_seconds(0.25, tempo))
                        }
                        Value::Number(n) => n / 1_000.0, // Convert ms to seconds
                        _ => beats_to_seconds(0.25, tempo),
                    };
                    cursor += duration;
                    max_time = max_time.max(cursor);
                }
                StatementKind::Spawn { name, .. } => {
                    // Nested spawn support
                    if let Some(body) = groups.get(name) {
                        use crate::language::scope::VariableTable;
                        let child_vars = VariableTable::with_parent(variables.clone());

                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            samples,
                            sample_cache,
                            banks,
                            inserts,
                            alias_to_insert,
                            groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;
                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);
                    }
                }
                StatementKind::For {
                    variable,
                    iterable,
                    body,
                } => {
                    // Resolve iterable
                    let resolved_iterable = self.resolve_value(iterable, &variables);
                    let items = match resolved_iterable {
                        Value::Array(arr) => arr,
                        Value::Number(n) => {
                            (0..(n as i32)).map(|i| Value::Number(i as f32)).collect()
                        }
                        _ => continue,
                    };

                    for item in items {
                        use crate::language::scope::{BindingType, VariableTable};
                        let mut child_vars = VariableTable::with_parent(variables.clone());
                        child_vars.set_with_type(variable.clone(), item, BindingType::Let);

                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            samples,
                            sample_cache,
                            banks,
                            inserts,
                            alias_to_insert,
                            groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;
                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);
                    }
                }
                StatementKind::Loop { count, body } => {
                    let resolved_count = self.resolve_value(count, &variables);
                    let iterations = match resolved_count {
                        Value::Number(n) => n as usize,
                        _ => continue,
                    };

                    for _ in 0..iterations {
                        use crate::language::scope::VariableTable;
                        let child_vars = VariableTable::with_parent(variables.clone());

                        let spawn_plan = self.plan_jobs_internal(
                            body,
                            base_dir,
                            project_root,
                            samples,
                            sample_cache,
                            banks,
                            inserts,
                            alias_to_insert,
                            groups,
                            child_vars,
                            tempo,
                            cursor,
                        )?;
                        jobs.extend(spawn_plan.jobs);
                        cursor = spawn_plan.cursor;
                        max_time = max_time.max(spawn_plan.max_time);
                    }
                }
                StatementKind::If {
                    condition,
                    body,
                    else_body,
                } => {
                    let condition_result = self.evaluate_condition(condition, &variables);
                    let branch_to_execute = if condition_result {
                        body
                    } else if let Some(else_branch) = else_body {
                        else_branch
                    } else {
                        continue;
                    };

                    use crate::language::scope::VariableTable;
                    let child_vars = VariableTable::with_parent(variables.clone());

                    let spawn_plan = self.plan_jobs_internal(
                        branch_to_execute,
                        base_dir,
                        project_root,
                        samples,
                        sample_cache,
                        banks,
                        inserts,
                        alias_to_insert,
                        groups,
                        child_vars,
                        tempo,
                        cursor,
                    )?;
                    jobs.extend(spawn_plan.jobs);
                    cursor = spawn_plan.cursor;
                    max_time = max_time.max(spawn_plan.max_time);
                }
                _ => {}
            }
        }

        Ok(SpawnPlan {
            jobs,
            cursor,
            max_time,
        })
    }

    /// Resolve a value, replacing identifiers with their variable values
    fn resolve_value(
        &self,
        value: &Value,
        variables: &crate::language::scope::VariableTable,
    ) -> Value {
        match value {
            Value::Identifier(name) => {
                // Try to resolve as variable, then recursively resolve the result
                if let Some(var_value) = variables.get(name) {
                    return self.resolve_value(&var_value, variables);
                }
                // Return as-is if not found (might be a reference to something else)
                value.clone()
            }
            Value::String(s) => {
                // Try to parse as number first
                if let Ok(num) = s.parse::<f32>() {
                    return Value::Number(num);
                }
                // Check if it looks like an identifier (no spaces, quotes)
                if !s.contains(' ') && !s.contains('"') {
                    // Check if there's a variable with this name
                    if let Some(var_value) = variables.get(s) {
                        // Recursively resolve
                        return self.resolve_value(&var_value, variables);
                    }
                    // Otherwise, return as Identifier for entity resolution (drums.kick)
                    return Value::Identifier(s.clone());
                }
                value.clone()
            }
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .map(|v| self.resolve_value(v, variables))
                    .collect(),
            ),
            Value::Map(map) => Value::Map(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.resolve_value(v, variables)))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    /// Evaluate a condition to a boolean
    fn evaluate_condition(
        &self,
        condition: &Value,
        variables: &crate::language::scope::VariableTable,
    ) -> bool {
        match condition {
            Value::Map(map) => {
                // Condition is a comparison: { operator: ">", left: "tempo", right: 120 }
                let operator = map.get("operator").and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                });

                let left = map.get("left").map(|v| self.resolve_value(v, variables));
                let right = map.get("right").map(|v| self.resolve_value(v, variables));

                if let (Some(op), Some(left_val), Some(right_val)) = (operator, left, right) {
                    // Extract numbers from values
                    let left_num = match left_val {
                        Value::Number(n) => n,
                        _ => return false,
                    };
                    let right_num = match right_val {
                        Value::Number(n) => n,
                        _ => return false,
                    };

                    // Evaluate comparison
                    match op {
                        ">" => left_num > right_num,
                        "<" => left_num < right_num,
                        ">=" => left_num >= right_num,
                        "<=" => left_num <= right_num,
                        "==" => (left_num - right_num).abs() < f32::EPSILON,
                        "!=" => (left_num - right_num).abs() >= f32::EPSILON,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Value::Boolean(b) => *b,
            Value::Identifier(name) => {
                // Try to resolve as variable
                if let Some(var_value) = variables.get(name) {
                    self.evaluate_condition(&var_value, variables)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn load_sample_cached(
        &self,
        cache: &mut HashMap<PathBuf, SampleBuffer>,
        path: PathBuf,
    ) -> Result<SampleBuffer> {
        if let Some(existing) = cache.get(&path) {
            return Ok(existing.clone());
        }
        if let Ok(canonical) = std::fs::canonicalize(&path) {
            if let Some(existing) = cache.get(&canonical) {
                let sample = existing.clone();
                cache.insert(path.clone(), sample.clone());
                return Ok(sample);
            }
        }
        let sample = self.load_sample_from_path(&path)?;
        if let Ok(canonical) = std::fs::canonicalize(&path) {
            cache.insert(canonical, sample.clone());
        }
        cache.insert(path, sample.clone());
        Ok(sample)
    }
    fn load_sample_from_path(&self, path: &Path) -> Result<SampleBuffer> {
        let mut reader = WavReader::open(path)
            .with_context(|| format!("failed to open sample file: {}", path.display()))?;
        let spec = reader.spec();
        if spec.sample_format != SampleFormat::Int {
            return Err(anyhow!(
                "only PCM integer WAV samples are supported ({}): format {:?}",
                path.display(),
                spec.sample_format
            ));
        }
        let channels = spec.channels as usize;
        let samples: Vec<f32> = match spec.bits_per_sample {
            16 => {
                let raw: Result<Vec<f32>, _> = reader
                    .samples::<i16>()
                    .map(|sample| sample.map(|v| v as f32 / i16::MAX as f32))
                    .collect();
                raw.with_context(|| {
                    format!("failed to read samples from file: {}", path.display())
                })?
            }
            24 => {
                const I24_MAX: f32 = 8_388_607.0;
                let raw: Result<Vec<f32>, _> = reader
                    .samples::<i32>()
                    .map(|sample| sample.map(|v| v as f32 / I24_MAX))
                    .collect();
                raw.with_context(|| {
                    format!("failed to read samples from file: {}", path.display())
                })?
            }
            other => {
                return Err(anyhow!(
                    "unsupported bit depth {} in sample {} (expected 16-bit or 24-bit)",
                    other,
                    path.display()
                ));
            }
        };
        Ok(SampleBuffer::new(
            Arc::new(samples),
            channels,
            spec.sample_rate,
        ))
    }
    fn write_wav(
        &self,
        path: &Path,
        pcm: &[f32],
        sample_rate: u32,
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
    ) -> Result<AudioBitDepth> {
        let (bit_depth, sample_format) = match requested_bit_depth {
            AudioBitDepth::Bit32 => (AudioBitDepth::Bit32, SampleFormat::Float),
            AudioBitDepth::Bit24 => (AudioBitDepth::Bit24, SampleFormat::Int),
            AudioBitDepth::Bit16 => (AudioBitDepth::Bit16, SampleFormat::Int),
            AudioBitDepth::Bit8 => (AudioBitDepth::Bit8, SampleFormat::Int),
        };
        let spec = WavSpec {
            channels: channels.count(),
            sample_rate,
            bits_per_sample: bit_depth.bits(),
            sample_format,
        };
        let mut writer = WavWriter::create(path, spec)
            .with_context(|| format!("failed to open WAV writer for {}", path.display()))?;
        match bit_depth {
            AudioBitDepth::Bit32 => {
                for sample in pcm {
                    writer
                        .write_sample(sample.clamp(-1.0, 1.0))
                        .with_context(|| {
                            format!("unable to write audio sample to {}", path.display())
                        })?;
                }
            }
            AudioBitDepth::Bit24 => {
                for sample in pcm {
                    let scaled = (sample.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
                    writer.write_sample(scaled).with_context(|| {
                        format!("unable to write audio sample to {}", path.display())
                    })?;
                }
            }
            AudioBitDepth::Bit16 => {
                for sample in pcm {
                    let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                    writer.write_sample(scaled).with_context(|| {
                        format!("unable to write audio sample to {}", path.display())
                    })?;
                }
            }
            AudioBitDepth::Bit8 => {
                for sample in pcm {
                    let scaled = (sample.clamp(-1.0, 1.0) * i8::MAX as f32).round() as i8;
                    writer.write_sample(scaled).with_context(|| {
                        format!("unable to write audio sample to {}", path.display())
                    })?;
                }
            }
        }
        writer.finalize().with_context(|| {
            format!("failed to finalize WAV file writer for {}", path.display())
        })?;
        Ok(bit_depth)
    }

    /// Write MP3 file from PCM samples
    /// Note: Currently exports as WAV with .mp3 extension as a placeholder
    /// Full MP3 encoding requires additional dependencies (e.g., lame, minimp3)
    fn write_mp3(
        &self,
        path: &Path,
        pcm: &[f32],
        sample_rate: u32,
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
    ) -> Result<AudioBitDepth> {
        // For now, log a message and create a WAV file
        // In production, this would use mp3lame-encoder or similar
        self.logger.warn(format!(
            "MP3 encoding not yet fully implemented. Exporting as WAV to: {}",
            path.display()
        ));

        // TODO: Implement actual MP3 encoding
        // Options:
        // 1. Use mp3lame-encoder crate (requires lame library)
        // 2. Use minimp3 encoder (pure Rust)
        // 3. Shell out to ffmpeg/lame binary

        // For now, write as WAV for testing
        self.write_wav(path, pcm, sample_rate, requested_bit_depth, channels)
    }

    /// Write FLAC file from PCM samples
    /// Note: Currently exports as WAV with .flac extension as a placeholder
    /// Full FLAC encoding requires additional dependencies (e.g., claxon)
    fn write_flac(
        &self,
        path: &Path,
        pcm: &[f32],
        sample_rate: u32,
        requested_bit_depth: AudioBitDepth,
        channels: AudioChannels,
    ) -> Result<AudioBitDepth> {
        // For now, log a message and create a WAV file
        // In production, this would use claxon or flac crate
        self.logger.warn(format!(
            "FLAC encoding not yet fully implemented. Exporting as WAV to: {}",
            path.display()
        ));

        // TODO: Implement actual FLAC encoding
        // Options:
        // 1. Use claxon crate (pure Rust encoder)
        // 2. Use flac-sys (bindings to libFLAC)
        // 3. Shell out to flac binary

        // For now, write as WAV for testing
        self.write_wav(path, pcm, sample_rate, requested_bit_depth, channels)
    }
}

fn find_project_root(entry_path: &Path) -> PathBuf {
    let mut current = entry_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    loop {
        if current.join(".deva").is_dir()
            || current.join("devalang.json").is_file()
            || current.join("devalang.toml").is_file()
            || current.join("Cargo.toml").is_file()
        {
            return current;
        }
        if !current.pop() {
            break;
        }
    }
    entry_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_sample_reference(base_dir: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base_dir.join(candidate)
    }
}

fn split_trigger_entity(entity: &str) -> (&str, Option<&str>) {
    if let Some((alias, rest)) = entity.split_once('.') {
        if rest.is_empty() {
            (alias, None)
        } else {
            (alias, Some(rest))
        }
    } else {
        (entity, None)
    }
}

fn register_route_for_alias(
    inserts: &mut HashMap<String, Option<String>>,
    alias_map: &mut HashMap<String, String>,
    alias: &str,
    prefix: &str,
    parent: Option<&str>,
) -> String {
    let sanitized = AudioMixer::sanitize_label(alias);
    let insert_name = format!("{}::{}", prefix, sanitized);
    if insert_name != MASTER_INSERT {
        let normalized_parent = parent
            .filter(|label| !is_master_label(label))
            .map(|label| label.to_string());
        match inserts.entry(insert_name.clone()) {
            Entry::Occupied(mut entry) => {
                entry.insert(normalized_parent.clone());
            }
            Entry::Vacant(slot) => {
                slot.insert(normalized_parent.clone());
            }
        }
    }
    alias_map
        .entry(alias.to_string())
        .or_insert_with(|| insert_name.clone());
    insert_name
}

fn is_master_label(label: &str) -> bool {
    let lower = label.trim().trim_start_matches('$').to_ascii_lowercase();
    lower == MASTER_INSERT
}

fn resolve_parent_for_alias(
    inserts: &mut HashMap<String, Option<String>>,
    alias_map: &mut HashMap<String, String>,
    target: &str,
) -> Option<String> {
    if is_master_label(target) {
        return None;
    }
    if let Some(existing) = alias_map.get(target) {
        return Some(existing.clone());
    }
    if target.contains("::") {
        inserts.entry(target.to_string()).or_insert(None);
        return Some(target.to_string());
    }
    let insert = register_route_for_alias(inserts, alias_map, target, "bus", None);
    Some(insert)
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(v)
        | Value::Identifier(v)
        | Value::Sample(v)
        | Value::Beat(v)
        | Value::Midi(v) => Some(v.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn default_bank_alias(identifier: &str) -> String {
    let candidate = identifier
        .rsplit(|c| c == '.' || c == '/' || c == '\\')
        .next()
        .unwrap_or(identifier);
    candidate.replace('-', "_").replace(' ', "_")
}

fn duration_in_seconds(duration: &DurationValue, tempo: f32) -> Option<f32> {
    match duration {
        DurationValue::Milliseconds(ms) => Some(ms / 1_000.0),
        DurationValue::Beats(beats) => Some(beats_to_seconds(*beats, tempo)),
        DurationValue::Number(value) => Some(value / 1_000.0),
        DurationValue::Beat(value) => parse_fraction(value).map(|b| beats_to_seconds(b, tempo)),
        DurationValue::Identifier(_) | DurationValue::Auto => None,
    }
}

fn beats_to_seconds(beats: f32, tempo: f32) -> f32 {
    if tempo <= 0.0 {
        return 0.0;
    }
    beats * (60.0 / tempo)
}

fn parse_fraction(token: &str) -> Option<f32> {
    let mut split = token.split('/');
    let numerator: f32 = split.next()?.trim().parse().ok()?;
    let denominator: f32 = split.next()?.trim().parse().ok()?;
    if denominator.abs() < f32::EPSILON {
        return None;
    }
    Some(numerator / denominator)
}

fn normalize(buffer: &mut [f32]) {
    let peak = buffer
        .iter()
        .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));
    if peak > 1.0 {
        let inv = 1.0 / peak;
        for sample in buffer.iter_mut() {
            *sample *= inv;
        }
    }
}

fn calculate_rms(buffer: &[f32]) -> f32 {
    if buffer.is_empty() {
        return 0.0;
    }
    let sum = buffer.iter().map(|sample| sample * sample).sum::<f32>();
    (sum / buffer.len() as f32).sqrt()
}

// ============================================================================
// EFFECT PROCESSING FUNCTIONS
// ============================================================================

/// Apply effects defined in trigger metadata to sample buffer
fn apply_trigger_effects(
    buffer: &mut Vec<f32>,
    effects: &Value,
    sample_rate: u32,
    channels: usize,
) -> Result<()> {
    let Value::Map(effect_map) = effects else {
        return Ok(());
    };

    // 1. Reverse - flip buffer backwards
    if let Some(Value::Boolean(true)) = effect_map.get("reverse") {
        buffer.reverse();
    }

    // 2. Pitch shift (via simple resampling)
    if let Some(Value::Number(pitch)) = effect_map.get("pitch") {
        // pitch: -1.0 = down octave, 0.0 = no change, +1.0 = up octave
        let rate_multiplier = 2.0_f32.powf(*pitch);
        *buffer = resample_buffer(buffer, rate_multiplier, channels);
    }

    // 3. Velocity (volume adjustment)
    if let Some(Value::Number(velocity)) = effect_map.get("velocity") {
        let gain = (velocity / 127.0).clamp(0.0, 2.0); // MIDI velocity to gain
        for sample in buffer.iter_mut() {
            *sample *= gain;
        }
    }

    // 4. Delay effect
    if let Some(Value::Number(delay_ms)) = effect_map.get("delay") {
        let feedback = effect_map
            .get("delay_feedback")
            .and_then(|v| {
                if let Value::Number(f) = v {
                    Some(*f)
                } else {
                    None
                }
            })
            .unwrap_or(0.5);
        apply_delay(buffer, *delay_ms, sample_rate, channels, feedback)?;
    }

    // 5. Reverb effect
    if let Some(Value::Number(reverb_amount)) = effect_map.get("reverb") {
        apply_reverb(buffer, *reverb_amount, sample_rate, channels)?;
    }

    Ok(())
}

/// Simple resampling for pitch shift effect
fn resample_buffer(buffer: &[f32], rate: f32, channels: usize) -> Vec<f32> {
    if rate <= 0.0 || (rate - 1.0).abs() < 0.001 {
        return buffer.to_vec();
    }

    let frames = buffer.len() / channels.max(1);
    let new_frames = (frames as f32 / rate).max(1.0) as usize;
    let mut result = Vec::with_capacity(new_frames * channels);

    for i in 0..new_frames {
        let src_frame_f = i as f32 * rate;
        let src_frame = src_frame_f as usize;

        if src_frame >= frames {
            // Pad with zeros if we've run out
            for _ in 0..channels {
                result.push(0.0);
            }
            continue;
        }

        // Linear interpolation between frames
        let frac = src_frame_f - src_frame as f32;
        let next_frame = (src_frame + 1).min(frames - 1);

        for ch in 0..channels {
            let sample1 = buffer[src_frame * channels + ch];
            let sample2 = buffer[next_frame * channels + ch];
            let interpolated = sample1 + (sample2 - sample1) * frac;
            result.push(interpolated);
        }
    }

    result
}

/// Apply delay effect (echo)
fn apply_delay(
    buffer: &mut Vec<f32>,
    delay_ms: f32,
    sample_rate: u32,
    channels: usize,
    feedback: f32,
) -> Result<()> {
    let delay_frames = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
    let delay_samples = delay_frames * channels;

    if delay_samples == 0 {
        return Ok(());
    }

    let original_len = buffer.len();
    let frames = original_len / channels.max(1);

    // Extend buffer to accommodate delay tail (3 echoes)
    let extended_frames = frames + delay_frames * 3;
    buffer.resize(extended_frames * channels, 0.0);

    // Apply feedback delay
    let feedback_clamped = feedback.clamp(0.0, 0.9);
    for i in delay_samples..buffer.len() {
        buffer[i] += buffer[i - delay_samples] * feedback_clamped;
    }

    Ok(())
}

/// Apply simple reverb (multi-tap delay)
fn apply_reverb(
    buffer: &mut Vec<f32>,
    amount: f32,
    sample_rate: u32,
    channels: usize,
) -> Result<()> {
    if amount <= 0.0 {
        return Ok(());
    }

    let amount_clamped = amount.clamp(0.0, 1.0);

    // Prime number delays for natural reverb (in milliseconds)
    let delay_times = [13.0, 23.0, 37.0, 53.0, 71.0, 97.0];

    let original = buffer.clone();
    let frames = original.len() / channels.max(1);

    // Find longest delay to extend buffer
    let max_delay_frames = delay_times
        .iter()
        .map(|&ms| ((ms / 1000.0) * sample_rate as f32) as usize)
        .max()
        .unwrap_or(0);

    buffer.resize((frames + max_delay_frames) * channels, 0.0);

    // Apply each delay tap
    for (tap_idx, &delay_ms) in delay_times.iter().enumerate() {
        let delay_frames = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
        let delay_samples = delay_frames * channels;

        if delay_samples >= original.len() {
            continue;
        }

        // Decreasing gain for each tap
        let tap_gain = amount_clamped * (0.3 / (tap_idx + 1) as f32);

        for i in 0..original.len() {
            let target_idx = i + delay_samples;
            if target_idx < buffer.len() {
                buffer[target_idx] += original[i] * tap_gain;
            }
        }
    }

    Ok(())
}
