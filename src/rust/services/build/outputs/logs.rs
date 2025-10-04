#![cfg(feature = "cli")]

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use time::OffsetDateTime;
use time::macros::format_description;

const LOG_FILE_NAME: &str = "build.log";
const TIMESTAMP_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour padding:zero]:[minute padding:zero]:[second padding:zero]");

#[derive(Debug, Clone, Copy, Default)]
pub struct LogWriter;

impl LogWriter {
    pub fn new() -> Self {
        Self
    }

    pub fn clear(&self, output_root: impl AsRef<Path>) -> Result<()> {
        let root = output_root.as_ref().join("logs");
        create_dir_all(&root)
            .with_context(|| format!("failed to create log directory: {}", root.display()))?;

        let log_path = root.join(LOG_FILE_NAME);
        if log_path.exists() {
            std::fs::remove_file(&log_path)
                .with_context(|| format!("failed to remove log file: {}", log_path.display()))?;
        }

        Ok(())
    }

    pub fn append(&self, output_root: impl AsRef<Path>, message: &str) -> Result<()> {
        let root = output_root.as_ref().join("logs");
        create_dir_all(&root)
            .with_context(|| format!("failed to create log directory: {}", root.display()))?;

        let log_path = root.join(LOG_FILE_NAME);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("failed to open log file: {}", log_path.display()))?;

        let timestamp = OffsetDateTime::now_utc();
        let formatted = timestamp
            .format(TIMESTAMP_FORMAT)
            .unwrap_or_else(|_| "00:00:00".to_string());

        writeln!(file, "[{}] {}", formatted, message)
            .with_context(|| format!("unable to write log record: {}", log_path.display()))?;

        Ok(())
    }
}
