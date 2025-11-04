use crate::engine::audio::events::AudioEventList;
use crate::engine::audio::interpreter::collector;
use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::ast::{Statement, Value};
use anyhow::Result;
use std::sync::atomic::{AtomicUsize, Ordering};

static BG_WORKER_COUNTER: AtomicUsize = AtomicUsize::new(1);

impl AudioInterpreter {
    pub fn execute_loop(&mut self, count: &Value, body: &[Statement]) -> Result<()> {
        match count {
            Value::Number(n) => {
                let loop_count = (*n) as usize;
                for i in 0..loop_count {
                    self.collect_events(body)?;
                    // If a break was signalled inside the body, consume flag and exit loop
                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                }
                Ok(())
            }
            Value::Identifier(ident) if ident == "pass" => {
                // offline: run in-place per beat
                if self.background_event_tx.is_none() {
                    let beat_secs = if self.bpm > 0.0 { 60.0 / self.bpm } else { 1.0 };
                    let interval_secs = beat_secs.max(0.001);
                    let mut iter_count: usize = 0;
                    let hard_iter_cap: usize = 100_000;
                    let start = self.cursor_time;
                    let render_target = self.special_vars.total_duration.max(1.0);

                    loop {
                        let before_cursor = self.cursor_time;
                        // Avoid nested beat emission when executing loop body repeatedly:
                        // suppress beat emission during inner collection so 'on beat'/'on bar'
                        // handlers are only executed by the outer/top-level collector.
                        let prev = self.suppress_beat_emit;
                        self.suppress_beat_emit = true;
                        let res = self.collect_events(body);
                        // restore previous flag even if collect_events returned an error
                        self.suppress_beat_emit = prev;
                        res?;
                        // Break signalled inside loop body -> exit pass loop
                        if self.break_flag {
                            self.break_flag = false;
                            break;
                        }
                        iter_count = iter_count.saturating_add(1);
                        let after_cursor = self.cursor_time;
                        // If the body advanced the cursor_time, we keep that (no extra gap).
                        // Otherwise advance by the interval (pass/beat) to avoid stalling.
                        if (after_cursor - before_cursor).abs() < f32::EPSILON {
                            self.cursor_time += interval_secs;
                        }
                        if self.cursor_time - start >= render_target { break; }
                        if iter_count > hard_iter_cap { break; }
                    }
                    return Ok(());
                }

                // live path handled elsewhere (spawning worker) in previous code paths
                anyhow::bail!("Live background worker path should be handled earlier");
            }
            Value::Call { name, args } if name == "pass" => {
                // treat as pass(ms) with offline synchronous behavior
                let mut interval_ms: u64 = 1000;
                if let Some(Value::Number(n)) = args.get(0) {
                    interval_ms = (*n) as u64;
                }
                let interval_secs = (interval_ms as f32) / 1000.0;
                let mut iter_count: usize = 0;
                let hard_iter_cap: usize = 100_000;
                let start = self.cursor_time;
                let render_target = self.special_vars.total_duration.max(1.0);
                loop {
                    let before_cursor = self.cursor_time;
                    let prev = self.suppress_beat_emit;
                    self.suppress_beat_emit = true;
                    let res = self.collect_events(body);
                    self.suppress_beat_emit = prev;
                    res?;
                    // Break signalled inside loop body -> exit pass loop
                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                    iter_count = iter_count.saturating_add(1);
                    let after_cursor = self.cursor_time;
                    if (after_cursor - before_cursor).abs() < f32::EPSILON {
                        self.cursor_time += interval_secs;
                    }
                    if self.cursor_time - start >= render_target { break; }
                    if iter_count > hard_iter_cap { break; }
                }
                Ok(())
            }
            Value::Null => {
                // Indefinite loop: run until no further audio is produced or time limit hit
                let start_time = self.cursor_time;
                let time_limit = 60.0_f32;
                loop {
                    let before_cursor = self.cursor_time;
                    let prev = self.suppress_beat_emit;
                    self.suppress_beat_emit = true;
                    let res = self.collect_events(body);
                    self.suppress_beat_emit = prev;
                    res?;
                    // Break signalled inside indefinite loop -> exit
                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                    if (self.cursor_time - before_cursor).abs() < f32::EPSILON { break; }
                    if self.cursor_time - start_time >= time_limit { break; }
                }
                Ok(())
            }
            other => anyhow::bail!("❌ Loop iterator must be a number, 'pass' or null, found: {:?}", other),
        }
    }

    pub fn execute_for(&mut self, variable: &str, iterable: &Value, body: &[Statement]) -> Result<()> {
        let items = match iterable {
            Value::Array(arr) => arr.clone(),
            Value::Identifier(ident) => {
                if let Some(Value::Array(arr)) = self.variables.get(ident) { arr.clone() } else { anyhow::bail!("❌ For iterable '{}' must be an array", ident) }
            }
            Value::Range { start, end } => {
                let start_val = match start.as_ref() { Value::Number(n) => *n as i32, _ => anyhow::bail!("❌ Range start must be a number") };
                let end_val = match end.as_ref() { Value::Number(n) => *n as i32, _ => anyhow::bail!("❌ Range end must be a number") };
                (start_val..end_val).map(|i| Value::Number(i as f32)).collect()
            }
            _ => anyhow::bail!("❌ For iterable must be an array or range, found: {:?}", iterable),
        };

        for item in items.iter() {
            let old_value = self.variables.insert(variable.to_string(), item.clone());
            self.collect_events(body)?;
            // If a break was signalled inside the body, consume flag and exit for-loop
            if self.break_flag {
                self.break_flag = false;
                // Restore previous variable state before exiting
                match old_value {
                    Some(val) => {
                        self.variables.insert(variable.to_string(), val);
                    }
                    None => {
                        self.variables.remove(variable);
                    }
                }
                break;
            }
            match old_value { Some(val) => { self.variables.insert(variable.to_string(), val); } None => { self.variables.remove(variable); } }
        }

        Ok(())
    }
}
