use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tokio::time::sleep;

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};
use crate::tools::logger::Logger;

#[derive(Clone)]
pub struct LivePlaybackEngine {
    inner: Arc<LivePlaybackInner>,
}

struct LivePlaybackInner {
    logger: Arc<Logger>,
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl LivePlaybackEngine {
    pub fn new(logger: Arc<Logger>) -> Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().context("failed to access default audio output stream")?;
        Ok(Self {
            inner: Arc::new(LivePlaybackInner {
                logger,
                _stream: stream,
                handle,
            }),
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.inner.logger
    }

    fn handle(&self) -> &OutputStreamHandle {
        &self.inner.handle
    }

    fn create_sink(&self, source: &LiveAudioSource) -> Result<Sink> {
        create_sink_with_handle(self.handle(), source)
    }

    pub async fn play_once(&self, source: LiveAudioSource, volume: f32) -> Result<()> {
        let volume_display = if volume == 0.0 {
            " [MUTED]".to_string()
        } else if volume < 1.0 {
            format!(" [volume: {:.0}%]", volume * 100.0)
        } else {
            String::new()
        };

        self.logger().action(format!(
            "Playing {} ({:?}, {}-bit, {} ch, {}, {} Hz, length {}){}",
            source.path.display(),
            source.format,
            source.bit_depth.bits(),
            source.channels.count(),
            source.resample_quality,
            source.sample_rate,
            format_duration_short(source.length),
            volume_display
        ));
        let sink = Arc::new(self.create_sink(&source)?);
        sink.set_volume(volume);

        // Attempt to load scheduled print events sidecar (module.printlog) for this audio
        let mut scheduled_logs: Vec<(f32, String)> = Vec::new();
        if let Some(stem) = source.path.file_stem().and_then(|s| s.to_str()) {
            let log_path = source.path.with_file_name(format!("{}.printlog", stem));
            if let Ok(contents) = std::fs::read_to_string(&log_path) {
                for line in contents.lines() {
                    if let Some((t, msg)) = line.split_once('\t') {
                        if let Ok(secs) = t.parse::<f32>() {
                            scheduled_logs.push((secs, msg.to_string()));
                        }
                    }
                }
                scheduled_logs
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        let sink_clone = Arc::clone(&sink);
        let wait_handle = std::thread::spawn(move || {
            sink_clone.sleep_until_end();
        });

        // Track playback start time so we can schedule print events
        let start_instant = std::time::Instant::now();
        let mut next_log_idx: usize = 0;

        // Poll loop: while playback is ongoing emit scheduled prints
        let poll_interval = std::time::Duration::from_millis(25);
        loop {
            if wait_handle.is_finished() {
                let _ = wait_handle.join();
                break;
            }

            if !scheduled_logs.is_empty() {
                let elapsed = start_instant.elapsed().as_secs_f32();
                while next_log_idx < scheduled_logs.len()
                    && scheduled_logs[next_log_idx].0 <= elapsed
                {
                    let msg = &scheduled_logs[next_log_idx].1;
                    self.logger().print(msg.clone());
                    next_log_idx += 1;
                }
            }

            std::thread::sleep(poll_interval);
        }

        sink.stop();
        self.logger().success("Playback completed.");
        Ok(())
    }

    pub async fn start_live_session(
        &self,
        source: LiveAudioSource,
        options: LivePlaybackOptions,
        background_event_rx: Option<
            std::sync::Arc<
                std::sync::Mutex<
                    std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>,
                >,
            >,
        >,
    ) -> Result<LivePlaybackSession> {
        let volume = options.volume();
        let volume_display = if volume == 0.0 {
            " [MUTED]".to_string()
        } else if volume < 1.0 {
            format!(" [volume: {:.0}%]", volume * 100.0)
        } else {
            String::new()
        };

        self.logger().action(format!(
            "Starting live session from {} ({:?}, {}-bit, {} ch, {}, {} Hz, loop {}){}",
            source.path.display(),
            source.format,
            source.bit_depth.bits(),
            source.channels.count(),
            source.resample_quality,
            source.sample_rate,
            format_duration_short(source.length),
            volume_display
        ));
        let (tx, rx) = mpsc::channel();
        let last_update = Arc::new(Mutex::new(Instant::now()));
        let logger = Arc::clone(&self.inner.logger);
        let handle_clone = self.handle().clone();
        let options_clone = options.clone();
        let source_clone = source.clone();
        let last_update_for_thread = Arc::clone(&last_update);
        let handle = thread::spawn(move || {
            run_loop(
                logger,
                handle_clone,
                source_clone,
                options_clone,
                rx,
                last_update_for_thread,
            )
        });

        Ok(LivePlaybackSession::new(
            self.clone(),
            tx,
            handle,
            last_update,
            options,
            background_event_rx,
        ))
    }
}

fn create_sink_with_handle(handle: &OutputStreamHandle, source: &LiveAudioSource) -> Result<Sink> {
    let file = File::open(&source.path)
        .with_context(|| format!("unable to open audio file: {}", source.path.display()))?;
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader)
        .with_context(|| format!("failed to decode audio file: {}", source.path.display()))?;
    let sink = Sink::try_new(handle).context("failed to create audio sink")?;
    sink.append(decoder);
    sink.set_volume(1.0);
    Ok(sink)
}

fn run_loop(
    logger: Arc<Logger>,
    handle: OutputStreamHandle,
    initial: LiveAudioSource,
    options: LivePlaybackOptions,
    rx: mpsc::Receiver<PlaybackCommand>,
    last_update: Arc<Mutex<Instant>>,
) -> Result<()> {
    let mut current = initial;
    let mut pending: Option<LiveAudioSource> = None;
    let poll_interval = options.poll_interval().max(Duration::from_millis(25));

    loop {
        logger.watch(format!(
            "Looping {} (~{})",
            current.path.display(),
            format_duration_short(current.length)
        ));
        if let Ok(mut guard) = last_update.lock() {
            *guard = Instant::now();
        }

        let sink = match create_sink_with_handle(&handle, &current) {
            Ok(sink) => {
                sink.set_volume(options.volume());
                Arc::new(sink)
            }
            Err(err) => {
                logger.error(format!("Failed to prepare live buffer: {err}"));
                match rx.recv() {
                    Ok(PlaybackCommand::Queue(next)) => {
                        pending = Some(next);
                        continue;
                    }
                    Ok(PlaybackCommand::Stop) | Err(_) => break,
                }
            }
        };

        // Attempt to load scheduled print events sidecar (module.printlog) for this audio
        let mut scheduled_logs: Vec<(f32, String)> = Vec::new();
        if let Some(stem) = current.path.file_stem().and_then(|s| s.to_str()) {
            let log_path = current.path.with_file_name(format!("{}.printlog", stem));
            if let Ok(contents) = std::fs::read_to_string(&log_path) {
                for line in contents.lines() {
                    if let Some((t, msg)) = line.split_once('\t') {
                        if let Ok(secs) = t.parse::<f32>() {
                            scheduled_logs.push((secs, msg.to_string()));
                        }
                    }
                }
                scheduled_logs
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        let sink_clone = Arc::clone(&sink);
        let wait_handle = thread::spawn(move || {
            sink_clone.sleep_until_end();
        });

        // Track playback start time so we can schedule print events
        let start_instant = Instant::now();
        let mut next_log_idx: usize = 0;

        let mut stop_requested = false;

        loop {
            if wait_handle.is_finished() {
                let _ = wait_handle.join();
                break;
            }
            // Emit scheduled prints at the correct playback time
            if !scheduled_logs.is_empty() {
                let elapsed = start_instant.elapsed().as_secs_f32();
                while next_log_idx < scheduled_logs.len()
                    && scheduled_logs[next_log_idx].0 <= elapsed
                {
                    let msg = &scheduled_logs[next_log_idx].1;
                    // Emit using the engine logger so print messages use the [PRINT] format
                    logger.print(msg.clone());
                    next_log_idx += 1;
                }
            }

            match rx.recv_timeout(poll_interval) {
                Ok(PlaybackCommand::Queue(next)) => {
                    pending = Some(next);
                }
                Ok(PlaybackCommand::Stop) => {
                    stop_requested = true;
                    sink.stop();
                    let _ = wait_handle.join();
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    stop_requested = true;
                    sink.stop();
                    let _ = wait_handle.join();
                    break;
                }
            }
        }

        if stop_requested {
            break;
        }

        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                PlaybackCommand::Queue(next) => pending = Some(next),
                PlaybackCommand::Stop => {
                    stop_requested = true;
                    break;
                }
            }
        }

        if stop_requested {
            break;
        }

        if let Some(next) = pending.take() {
            logger.success(format!(
                "Next build ready -> {} (~{}). Switching after current loop.",
                next.path.display(),
                format_duration_short(next.length)
            ));
            current = next;
        } else {
            logger.info("Replaying current loop (no pending build).");
        }
    }

    logger.info("Live playback loop stopped.");
    Ok(())
}

fn format_duration_short(duration: Duration) -> String {
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

#[derive(Clone)]
pub struct LiveAudioSource {
    pub path: PathBuf,
    pub format: AudioFormat,
    pub bit_depth: AudioBitDepth,
    pub channels: AudioChannels,
    pub sample_rate: u32,
    pub resample_quality: ResampleQuality,
    pub length: Duration,
}

impl LiveAudioSource {
    pub fn with_path(
        path: PathBuf,
        format: AudioFormat,
        bit_depth: AudioBitDepth,
        channels: AudioChannels,
        sample_rate: u32,
        resample_quality: ResampleQuality,
        length: Duration,
    ) -> Self {
        Self {
            path,
            format,
            bit_depth,
            channels,
            sample_rate,
            resample_quality,
            length,
        }
    }
}

#[derive(Clone)]
pub struct LivePlaybackOptions {
    poll_interval: Duration,
    volume: f32,
}

impl LivePlaybackOptions {
    pub fn new(poll_interval: Duration) -> Self {
        Self {
            poll_interval,
            volume: 1.0,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

enum PlaybackCommand {
    Queue(LiveAudioSource),
    Stop,
}

pub struct LivePlaybackSession {
    engine: LivePlaybackEngine,
    commands: mpsc::Sender<PlaybackCommand>,
    handle: Option<thread::JoinHandle<Result<()>>>,
    last_update: Arc<Mutex<Instant>>,
    options: LivePlaybackOptions,
    background_event_rx: Option<
        std::sync::Arc<
            std::sync::Mutex<
                std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>,
            >,
        >,
    >,
}

impl LivePlaybackSession {
    fn new(
        engine: LivePlaybackEngine,
        commands: mpsc::Sender<PlaybackCommand>,
        handle: thread::JoinHandle<Result<()>>,
        last_update: Arc<Mutex<Instant>>,
        options: LivePlaybackOptions,
        background_event_rx: Option<
            std::sync::Arc<
                std::sync::Mutex<
                    std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>,
                >,
            >,
        >,
    ) -> Self {
        Self {
            engine,
            commands,
            handle: Some(handle),
            last_update,
            options,
            background_event_rx,
        }
    }

    pub fn queue_source(&self, next: LiveAudioSource) -> Result<()> {
        self.commands
            .send(PlaybackCommand::Queue(next))
            .context("failed to queue next live buffer")
    }

    pub async fn heartbeat(&self) {
        sleep(self.options.poll_interval()).await;
    }

    pub async fn finish(mut self) -> Result<()> {
        let _ = self.commands.send(PlaybackCommand::Stop);
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => result?,
                Err(err) => bail!("live playback thread panicked: {err:?}"),
            }
        }
        self.engine
            .logger()
            .info("Live session finished; awaiting next command.");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn last_update(&self) -> Instant {
        *self.last_update.lock().expect("last_update poisoned")
    }
}
