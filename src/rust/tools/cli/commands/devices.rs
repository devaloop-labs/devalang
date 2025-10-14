#![cfg(feature = "cli")]

use crate::engine::events::{EventPayload, EventRegistry};
use crate::language::syntax::ast::StatementKind;
use crate::language::syntax::parser::driver::SimpleParser;
use crate::tools::cli::state::CliContext;
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// MIDI NOTE UTILITIES
// ============================================================================

/// Convert MIDI key number to note name (C4, C#4, etc.)
fn midi_note_to_name(note: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    let name = names[(note % 12) as usize];
    format!("{}{}", name, octave)
}

// ============================================================================
// MATH UTILITIES
// ============================================================================

/// Greatest common divisor for fraction simplification
fn gcd(mut a: i32, mut b: i32) -> i32 {
    if a < 0 {
        a = -a;
    }
    if b < 0 {
        b = -b;
    }
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

/// Convert duration in beats to a simplified fraction string "N/D"
/// Optimized for precision with tighter tolerance (5%) and extended denominators
fn beats_to_fraction(beats: f32) -> String {
    if beats <= 0.0 {
        return "1/16".to_string();
    }

    // Try common denominators with extended range: 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64
    // This covers dotted notes, triplets, and complex divisions
    for den in [1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64] {
        let num_f = beats * (den as f32);
        let num = num_f.round() as i32;

        if num <= 0 {
            continue;
        }

        // Check if this is a good approximation (within 5% for better precision)
        let actual = (num as f32) / (den as f32);
        let tolerance = 0.05; // 5% tolerance for ms-level precision
        if (actual - beats).abs() / beats.max(0.01) < tolerance {
            let g = gcd(num, den);
            let num = num / g;
            let den = den / g;
            return if den == 1 {
                format!("{}", num)
            } else {
                format!("{}/{}", num, den)
            };
        }
    }

    // Fallback: use 64ths (higher resolution)
    let num = (beats * 64.0).round() as i32;
    let g = gcd(num, 64);
    let num = num / g;
    let den = 64 / g;
    if den == 1 {
        format!("{}", num)
    } else {
        format!("{}/{}", num, den)
    }
}

// ============================================================================
// FILE MANIPULATION UTILITIES
// ============================================================================

fn ensure_file_ends_with_newline(path: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(path).exists() {
        return Ok(());
    }
    let s = std::fs::read_to_string(path).unwrap_or_default();
    if !s.is_empty() && !s.ends_with('\n') {
        let mut f = File::options().append(true).open(path)?;
        writeln!(f)?;
    }
    Ok(())
}

fn ensure_synth_decl_exists(path: &str, synth_name: &str, waveform: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    if !content.contains(&format!("let {}", synth_name)) {
        let mut f = if content.is_empty() {
            File::create(path)?
        } else {
            File::options().append(true).open(path)?
        };
        writeln!(f, "let {} = synth {}", synth_name, waveform)?;
        f.sync_all()?;
    }
    Ok(())
}

fn replace_or_insert_pattern_let(
    path: &str,
    var_name: &str,
    trigger: &str,
    pattern: &str,
    rewrite: bool,
) -> anyhow::Result<()> {
    let line = format!("pattern {} with {} = \"{}\"", var_name, trigger, pattern);

    // Ensure file exists
    if !std::path::Path::new(path).exists() {
        std::fs::write(path, "")?;
    }

    let stmts = SimpleParser::parse_file(path).unwrap_or_default();
    let mut found: Option<(usize, bool)> = None; // (line_idx, is_pattern)

    for stmt in stmts.iter() {
        match &stmt.kind {
            StatementKind::Pattern { name, .. } if name == var_name => {
                found = Some((stmt.line.saturating_sub(1), true));
                break;
            }
            StatementKind::Let { name, .. } if name == var_name => {
                found = Some((stmt.line.saturating_sub(1), false));
                break;
            }
            _ => {}
        }
    }

    if let Some((idx, is_pattern)) = found {
        let lines: Vec<String> = std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect();

        if idx < lines.len() {
            if rewrite || is_pattern {
                let mut new_lines = lines.clone();
                new_lines[idx] = line;
                let mut out = new_lines.join("\n");
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                std::fs::write(path, out)?;
                return Ok(());
            }
        }
    }

    ensure_file_ends_with_newline(path)?;
    let mut f = File::options().append(true).open(path)?;
    writeln!(f, "{}", line)?;
    f.sync_all()?;
    Ok(())
}

fn insert_lines_after_let(
    path: &str,
    var_name: &str,
    lines_to_insert: &[String],
) -> anyhow::Result<()> {
    let stmts = SimpleParser::parse_file(path).unwrap_or_default();

    for (i, stmt) in stmts.iter().enumerate() {
        if let StatementKind::Let { name, .. } = &stmt.kind {
            if name == var_name {
                let insert_line = if i + 1 < stmts.len() {
                    stmts[i + 1].line.saturating_sub(1)
                } else {
                    std::fs::read_to_string(path)
                        .unwrap_or_default()
                        .lines()
                        .count()
                };

                let lines: Vec<String> = std::fs::read_to_string(path)
                    .unwrap_or_default()
                    .lines()
                    .map(|s| s.to_string())
                    .collect();

                let insert_at = insert_line.min(lines.len());
                let mut out: Vec<String> = Vec::new();
                out.extend_from_slice(&lines[..insert_at]);
                for l in lines_to_insert.iter() {
                    out.push(l.clone());
                }
                out.extend_from_slice(&lines[insert_at..]);

                let mut joined = out.join("\n");
                if !joined.ends_with('\n') {
                    joined.push('\n');
                }
                std::fs::write(path, joined)?;
                return Ok(());
            }
        }
    }

    let mut f = File::options().append(true).open(path)?;
    for l in lines_to_insert {
        writeln!(f, "{}", l)?;
    }
    Ok(())
}

// ============================================================================
// PATTERN GENERATION
// ============================================================================

/// Generate a pattern string from recorded notes
/// Returns a pattern like "x--- x--- --x- x---" with correct timing
fn generate_pattern_from_recorded(
    recorded: &[(u8, f32, f32, u8, u8)], // (note, start_ms, dur_ms, vel, ch)
    step_duration_ms: f32,
    debug: bool,
    velocity_zero_as_rest: bool,
) -> String {
    if recorded.is_empty() {
        return "----".to_string();
    }

    // Filter velocity=0 if requested
    let notes: Vec<_> = if velocity_zero_as_rest {
        recorded
            .iter()
            .filter(|(_, _, _, vel, _)| *vel > 0)
            .copied()
            .collect()
    } else {
        recorded.to_vec()
    };

    if notes.is_empty() {
        return "----".to_string();
    }

    // Find the range of steps covered by the recording
    let first_start = notes
        .iter()
        .map(|(_, start, _, _, _)| *start)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let last_start = notes
        .iter()
        .map(|(_, start, _, _, _)| *start)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let first_step = (first_start / step_duration_ms).floor() as usize;
    let last_step = (last_start / step_duration_ms).floor() as usize;

    let grid_len = (last_step - first_step + 1).max(4);
    let mut grid = vec!['-'; grid_len];

    // Mark onsets in the grid
    // Use deduplication: only mark first note per step to avoid T-8 sending multiple events
    let mut occupied_steps = std::collections::HashSet::new();

    for (i, (note, start, dur, vel, _ch)) in notes.iter().enumerate() {
        let step = ((*start / step_duration_ms).round() as usize).saturating_sub(first_step);

        if debug {
            println!(
                "[PATTERN DEBUG] note {} ({}): start={:.1}ms dur={:.1}ms vel={} → step {}",
                i,
                midi_note_to_name(*note),
                start,
                dur,
                vel,
                step + first_step
            );
        }

        // Only mark if this step hasn't been marked yet (deduplication)
        if step < grid_len && !occupied_steps.contains(&step) {
            grid[step] = 'x';
            occupied_steps.insert(step);
        }
    }

    // Format in groups of 4
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < grid.len() {
        let end = (i + 4).min(grid.len());
        let segment: String = grid[i..end].iter().collect();
        parts.push(segment);
        i += 4;
    }

    let pattern = parts.join(" ");

    if debug {
        println!(
            "[PATTERN DEBUG] Generated pattern: \"{}\" (grid_len={}, unique_steps={})",
            pattern,
            grid_len,
            occupied_steps.len()
        );
    }

    pattern
}

// ============================================================================
// COMMAND: LIST DEVICES
// ============================================================================

#[derive(Args, Debug)]
pub struct DevicesListCommand {}

pub fn execute_list(_cmd: DevicesListCommand, ctx: &CliContext) -> Result<()> {
    let logger = ctx.logger();

    #[cfg(feature = "cli")]
    {
        use crate::engine::audio::midi_native::MidiManager;
        use crate::engine::events::EventRegistry;
        use std::sync::{Arc, Mutex};

        let registry = Arc::new(Mutex::new(EventRegistry::new()));
        let _mgr = MidiManager::new(registry.clone());
        let inputs = MidiManager::list_input_ports();

        logger.info("MIDI inputs:");
        for (i, name) in inputs.iter().enumerate() {
            logger.info(format!("  [{}] {} (channels: 16)", i, name));
        }

        #[cfg(feature = "cli")]
        {
            use midir::MidiOutput;
            if let Ok(mout) = MidiOutput::new("devalang-out") {
                let ports = mout.ports();
                logger.info("MIDI outputs:");
                for (i, port) in ports.iter().enumerate() {
                    let pname = mout
                        .port_name(port)
                        .unwrap_or_else(|_| "unknown".to_string());
                    logger.info(format!("  [{}] {} (channels: 16)", i, pname));
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// COMMAND: LIVE PREVIEW
// ============================================================================

#[derive(Args, Debug)]
pub struct DevicesLiveCommand {
    #[arg(long)]
    pub port: Option<usize>,
}

pub async fn execute_preview(cmd: DevicesLiveCommand, _ctx: &CliContext) -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use crate::engine::audio::midi_native::MidiManager;

        let registry = Arc::new(Mutex::new(EventRegistry::new()));
        let mut mgr = MidiManager::new(registry.clone());

        let inputs = MidiManager::list_input_ports();

        if let Some(idx) = cmd.port {
            if idx < inputs.len() {
                mgr.open_input_by_index(idx, &format!("dev{}", idx))
                    .map_err(|e: String| anyhow::anyhow!(e))?;
            } else {
                println!("Port index {} out of range ({} ports)", idx, inputs.len());
                return Ok(());
            }
        } else {
            for i in 0..inputs.len() {
                let _ = mgr.open_input_by_index(i, &format!("dev{}", i));
            }
        }

        println!("Listening for MIDI events (Ctrl+C to stop)...");

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("Stopping live MIDI listener");
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                    let mut to_print: Vec<EventPayload> = Vec::new();
                    if let Ok(mut reg) = registry.lock() {
                        let events = reg.get_emitted_events().to_vec();
                        if !events.is_empty() {
                            to_print = events;
                            reg.clear_events();
                        }
                    }

                    for ev in to_print {
                        println!("[{}] {} -> {:?}", ev.timestamp, ev.event_name, ev.data);
                    }
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// COMMAND: WRITE/RECORD
// ============================================================================

#[derive(Args, Debug)]
pub struct DevicesWriteCommand {
    /// Output file path
    #[arg(long)]
    pub out: String,

    /// Recording mode: "synth" (temporal sequence) or "pattern" (grid triggers)
    #[arg(long, default_value = "synth")]
    pub mode: String,

    /// Synth variable name
    #[arg(long, default_value = "mySynth")]
    pub synth_name: String,

    /// Synth waveform
    #[arg(long, default_value = "saw")]
    pub waveform: String,

    /// Pattern variable name (for pattern mode)
    #[arg(long, default_value = "myPattern")]
    pub pattern_name: String,

    /// MIDI port index (if not specified, opens all ports)
    #[arg(long)]
    pub port: Option<usize>,

    /// Step duration as fraction (e.g., "1/4" = quarter note, "1/16" = 16th note)
    #[arg(long, default_value = "1/16")]
    pub step: String,

    /// Tempo in BPM
    #[arg(long, default_value_t = 120.0)]
    pub bpm: f32,

    /// Group name to insert notes into (for synth live mode)
    #[arg(long)]
    pub group: Option<String>,

    /// Trigger name (for pattern mode, e.g., "myBank.kick")
    #[arg(long)]
    pub trigger: Option<String>,

    /// Enable live writing (update file during recording)
    #[arg(long, default_value_t = false)]
    pub live: bool,

    /// Enable debug output
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    /// Rewrite file from scratch (for live mode)
    #[arg(long, default_value_t = false)]
    pub rewrite: bool,

    /// Treat velocity 0 as rest (skip in pattern)
    #[arg(long, default_value_t = true)]
    pub velocity_zero_as_rest: bool,
}

/// Parse step fraction (e.g., "1/4" -> 0.25)
fn parse_step(step: &str) -> f32 {
    if let Some(slash) = step.find('/') {
        let num: f32 = step[..slash].parse().unwrap_or(1.0);
        let den: f32 = step[slash + 1..].parse().unwrap_or(4.0);
        num / den
    } else {
        step.parse::<f32>().unwrap_or(1.0)
    }
}

/// Main recording/writing logic
pub async fn execute_write(cmd: DevicesWriteCommand, _ctx: &CliContext) -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use crate::engine::audio::midi_native::MidiManager;

        let registry = Arc::new(Mutex::new(EventRegistry::new()));
        let mut mgr = MidiManager::new(registry.clone());

        let inputs = MidiManager::list_input_ports();

        // Open MIDI ports
        if let Some(idx) = cmd.port {
            if idx < inputs.len() {
                mgr.open_input_by_index(idx, &format!("dev{}", idx))
                    .map_err(|e: String| anyhow::anyhow!(e))?;
            } else {
                println!("Port index {} out of range ({} ports)", idx, inputs.len());
                return Ok(());
            }
        } else {
            for i in 0..inputs.len() {
                let _ = mgr.open_input_by_index(i, &format!("dev{}", i));
            }
        }

        println!("Recording MIDI (press Ctrl+C to stop)");
        println!("  Output: {}", cmd.out);
        println!("  Mode: {}", cmd.mode);
        println!("  BPM: {}", cmd.bpm);
        println!("  Step: {}", cmd.step);

        // Initialize file if live + rewrite
        if cmd.live && cmd.rewrite {
            let mut f = File::create(&cmd.out)?;
            if cmd.mode == "synth" {
                writeln!(f, "let {} = synth {}", cmd.synth_name, cmd.waveform)?;
            }
            f.sync_all()?;
        }

        // Calculate timing constants
        let step_fraction = parse_step(&cmd.step);
        let beats_per_step = step_fraction * 4.0; // Convert to beats (1 whole note = 4 beats)
        let step_duration_ms = beats_per_step * (60_000.0 / cmd.bpm);

        if cmd.debug {
            println!(
                "[DEBUG] Timing: step={} beats_per_step={:.3} step_duration_ms={:.1}ms",
                cmd.step, beats_per_step, step_duration_ms
            );
        }

        // Recording state
        let mut active_notes: HashMap<u8, (f32, u8, u8)> = HashMap::new(); // note -> (start_ms, vel, ch)
        let mut recorded: Vec<(u8, f32, f32, u8, u8)> = Vec::new(); // (note, start_ms, dur_ms, vel, ch)
        let mut logical_time_ms: f32 = 0.0;
        let mut last_tick = std::time::Instant::now();
        let mut paused = false;

        // Live mode state
        let mut last_timeline_position_ms: f32 = 0.0;

        // Stdin for pause command
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<String>(8);
        let tx_clone = cmd_tx.clone();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => {
                        let _ = tx_clone.blocking_send(l);
                    }
                    Err(_) => break,
                }
            }
        });

        // Main recording loop
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("\nStopping recording");
                    break;
                }

                maybe_cmd = cmd_rx.recv() => {
                    if let Some(line) = maybe_cmd {
                        if line.trim().eq_ignore_ascii_case("p") {
                            if !paused {
                                paused = true;
                                println!("⏸ Paused");
                            } else {
                                paused = false;
                                last_tick = std::time::Instant::now();
                                println!("▶ Resumed");
                            }
                        }
                    }
                }

                _ = sleep(Duration::from_millis(50)) => {
                    // Update logical time
                    let now_tick = std::time::Instant::now();
                    let delta = now_tick.duration_since(last_tick);
                    last_tick = now_tick;

                    if !paused {
                        logical_time_ms += delta.as_secs_f32() * 1000.0;
                    }

                    // Process MIDI events
                    if let Ok(mut reg) = registry.lock() {
                        let events = reg.get_emitted_events().to_vec();

                        if !events.is_empty() {
                            reg.clear_events();

                            for ev in events {
                                if paused {
                                    continue;
                                }

                                let ev_lower = ev.event_name.to_lowercase();
                                let is_note_on = ev_lower.contains("noteon") || ev_lower.contains("note_on") || ev_lower == "note on";
                                let is_note_off = ev_lower.contains("noteoff") || ev_lower.contains("note_off") || ev_lower == "note off";

                                if !is_note_on && !is_note_off {
                                    continue;
                                }

                                let note_val = ev.data.get("note");
                                let vel_val = ev.data.get("velocity");
                                let ch_val = ev.data.get("channel");

                                if let Some(crate::language::syntax::ast::Value::Number(note_num)) = note_val {
                                    let note = *note_num as u8;
                                    let vel = if let Some(crate::language::syntax::ast::Value::Number(v)) = vel_val {
                                        *v as u8
                                    } else {
                                        127
                                    };
                                    let ch = if let Some(crate::language::syntax::ast::Value::Number(c)) = ch_val {
                                        *c as u8
                                    } else {
                                        0
                                    };

                                    if cmd.debug {
                                        println!("[MIDI] {:.1}ms: {} {} vel={} ch={}",
                                                 logical_time_ms,
                                                 if is_note_on { "ON " } else { "OFF" },
                                                 midi_note_to_name(note),
                                                 vel,
                                                 ch);
                                    }

                                    // Note ON (with velocity > 0) or Note OFF / velocity=0
                                    if is_note_on && vel > 0 {
                                        // Note ON: start tracking
                                        active_notes.insert(note, (logical_time_ms, vel, ch));
                                    } else {
                                        // Note OFF or velocity=0: finalize note
                                        if let Some((start_ms, vel, ch)) = active_notes.remove(&note) {
                                            let dur_ms = (logical_time_ms - start_ms).max(1.0);
                                            recorded.push((note, start_ms, dur_ms, vel, ch));

                                            if cmd.debug {
                                                let dur_beats = (dur_ms / 1000.0) * (cmd.bpm / 60.0);
                                                println!("  → Recorded: {} start={:.1}ms dur={:.1}ms ({:.2} beats)",
                                                         midi_note_to_name(note), start_ms, dur_ms, dur_beats);
                                            }

                                            // LIVE WRITE: append events as they happen
                                            if cmd.live && cmd.mode == "synth" {
                                                write_synth_event_live(
                                                    &cmd,
                                                    note,
                                                    start_ms,
                                                    dur_ms,
                                                    vel,
                                                    &mut last_timeline_position_ms,
                                                    beats_per_step,
                                                )?;
                                            } else if cmd.live && cmd.mode == "pattern" {
                                                write_pattern_live(
                                                    &cmd,
                                                    &recorded,
                                                    step_duration_ms,
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // FINAL WRITE: write complete file if not live mode
        if !cmd.live {
            if cmd.mode == "synth" {
                write_synth_final(&cmd, &recorded, beats_per_step)?;
            } else if cmd.mode == "pattern" {
                write_pattern_final(&cmd, &recorded, step_duration_ms)?;
            }
        }

        println!("✓ Recorded {} notes to {}", recorded.len(), cmd.out);
    }

    Ok(())
}

// ============================================================================
// SYNTH MODE: LIVE WRITE
// ============================================================================

fn write_synth_event_live(
    cmd: &DevicesWriteCommand,
    note: u8,
    start_ms: f32,
    _dur_ms: f32,
    vel: u8,
    last_timeline_pos: &mut f32,
    beats_per_step: f32,
) -> Result<()> {
    let synth_name = cmd.group.as_ref().unwrap_or(&cmd.synth_name);

    // Ensure synth declaration exists
    ensure_synth_decl_exists(&cmd.out, synth_name, &cmd.waveform)?;

    // Calculate step duration
    let step_duration_ms = beats_per_step * (60_000.0 / cmd.bpm);

    // Calculate step positions for onset-to-onset analysis
    let is_first_note = *last_timeline_pos < 0.1;

    if !is_first_note {
        // Calculate step indices for both notes
        let prev_step = (*last_timeline_pos / step_duration_ms).round() as i32;
        let cur_step = (start_ms / step_duration_ms).round() as i32;
        let steps_between = cur_step - prev_step;

        // If there are multiple steps between onsets, there are rest steps
        // steps_rest = steps_between - 1 (because prev note occupies prev_step)
        let steps_rest = steps_between - 1;

        if steps_rest > 0 {
            // Insert sleep for the rest steps
            let rest_beats = (steps_rest as f32) * beats_per_step;
            let sleep_frac = beats_to_fraction(rest_beats);
            let sleep_line = format!("sleep {}", sleep_frac);

            if let Some(group_name) = &cmd.group {
                insert_lines_after_let(&cmd.out, group_name, &[sleep_line.clone()])?;
            } else {
                let mut f = File::options().append(true).open(&cmd.out)?;
                writeln!(f, "{}", sleep_line)?;
                f.flush()?;
            }

            if cmd.debug {
                println!(
                    "  [LIVE] Appended sleep: {} (steps_between={}, steps_rest={}, prev_step={}, cur_step={})",
                    sleep_frac, steps_between, steps_rest, prev_step, cur_step
                );
            }
        } else if cmd.debug {
            println!(
                "  [LIVE] No rest (steps_between={}, prev_step={}, cur_step={})",
                steps_between, prev_step, cur_step
            );
        }
    } else if cmd.debug {
        println!(
            "  [LIVE] First note at {:.1}ms (step {})",
            start_ms,
            (start_ms / step_duration_ms).round()
        );
    }

    // Write the note with default duration of 1/4 (quarter note)
    let note_name = midi_note_to_name(note);
    let note_line = format!(
        "{} -> note({}) -> duration(1/4) -> velocity({})",
        synth_name, note_name, vel
    );

    if let Some(group_name) = &cmd.group {
        insert_lines_after_let(&cmd.out, group_name, &[note_line.clone()])?;
    } else {
        let mut f = File::options().append(true).open(&cmd.out)?;
        writeln!(f, "{}", note_line)?;
        f.flush()?;
    }

    if cmd.debug {
        println!("  [LIVE] Appended note: {}", note_line);
    }

    // Update timeline position to THIS note's onset
    *last_timeline_pos = start_ms;

    Ok(())
}

// ============================================================================
// SYNTH MODE: FINAL WRITE
// ============================================================================

fn write_synth_final(
    cmd: &DevicesWriteCommand,
    recorded: &[(u8, f32, f32, u8, u8)],
    beats_per_step: f32,
) -> Result<()> {
    let mut f = File::create(&cmd.out)?;
    writeln!(f, "let {} = synth {}", cmd.synth_name, cmd.waveform)?;

    if recorded.is_empty() {
        return Ok(());
    }

    // Calculate step duration
    let step_duration_ms = beats_per_step * (60_000.0 / cmd.bpm);

    let mut last_onset_ms: Option<f32> = None;

    for (note, start_ms, _dur_ms, vel, _ch) in recorded {
        // Calculate step-based rest detection (skip for first note)
        if let Some(prev_onset) = last_onset_ms {
            let prev_step = (prev_onset / step_duration_ms).round() as i32;
            let cur_step = (*start_ms / step_duration_ms).round() as i32;
            let steps_between = cur_step - prev_step;
            let steps_rest = steps_between - 1;

            if steps_rest > 0 {
                // Insert sleep for rest steps
                let rest_beats = (steps_rest as f32) * beats_per_step;
                let sleep_frac = beats_to_fraction(rest_beats);
                writeln!(f, "sleep {}", sleep_frac)?;

                if cmd.debug {
                    println!(
                        "[FINAL] Inserted sleep {} (steps_between={}, steps_rest={}, prev_step={}, cur_step={})",
                        sleep_frac, steps_between, steps_rest, prev_step, cur_step
                    );
                }
            } else if cmd.debug {
                println!(
                    "[FINAL] No rest (steps_between={}, prev_step={}, cur_step={})",
                    steps_between, prev_step, cur_step
                );
            }
        } else if cmd.debug {
            println!(
                "[FINAL] First note at {:.1}ms (step {})",
                start_ms,
                (*start_ms / step_duration_ms).round()
            );
        }

        // Write note with default duration of 1/4 (quarter note)
        let note_name = midi_note_to_name(*note);
        writeln!(
            f,
            "{} -> note({}) -> duration(1/4) -> velocity({})",
            cmd.synth_name, note_name, vel
        )?;

        // Update last onset to THIS note's onset
        last_onset_ms = Some(*start_ms);
    }

    f.sync_all()?;
    Ok(())
}

// ============================================================================
// PATTERN MODE: LIVE WRITE
// ============================================================================

fn write_pattern_live(
    cmd: &DevicesWriteCommand,
    recorded: &[(u8, f32, f32, u8, u8)],
    step_duration_ms: f32,
) -> Result<()> {
    let pattern_name = &cmd.pattern_name;
    let trigger = cmd
        .trigger
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("TODO.trigger");

    let pattern_str = generate_pattern_from_recorded(
        recorded,
        step_duration_ms,
        cmd.debug,
        cmd.velocity_zero_as_rest,
    );

    replace_or_insert_pattern_let(&cmd.out, pattern_name, trigger, &pattern_str, true)?;

    if cmd.debug {
        println!("  [LIVE] Updated pattern: \"{}\"", pattern_str);
    }

    Ok(())
}

// ============================================================================
// PATTERN MODE: FINAL WRITE
// ============================================================================

fn write_pattern_final(
    cmd: &DevicesWriteCommand,
    recorded: &[(u8, f32, f32, u8, u8)],
    step_duration_ms: f32,
) -> Result<()> {
    let pattern_name = &cmd.pattern_name;
    let trigger = cmd
        .trigger
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("TODO.trigger");

    let pattern_str = generate_pattern_from_recorded(
        recorded,
        step_duration_ms,
        cmd.debug,
        cmd.velocity_zero_as_rest,
    );

    replace_or_insert_pattern_let(&cmd.out, pattern_name, trigger, &pattern_str, cmd.rewrite)?;

    if cmd.debug {
        println!("\n[FINAL] Generated pattern: \"{}\"", pattern_str);
        println!("        Recorded {} notes", recorded.len());
    }

    Ok(())
}
