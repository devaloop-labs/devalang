#![cfg(feature = "cli")]

use anyhow::Result;
use clap::Args;
use std::path::Path;
use std::time::Instant;

use crate::language::syntax::parser::driver::SimpleParser;
use crate::tools::cli::state::CliContext;

#[derive(Debug, Clone, Args)]
pub struct CheckCommand {
    /// Entry file or directory to check
    #[arg(short, long, default_value = "./examples")]
    pub entry: String,

    /// Watch for changes and re-check automatically
    #[arg(short, long, default_value_t = false)]
    pub watch: bool,

    /// Enable debug output (lexer, parser logs)
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

impl CheckCommand {
    pub async fn execute(&self, ctx: &CliContext) -> Result<()> {
        let logger = ctx.logger();

        if self.watch {
            logger.info("Watch mode not yet implemented");
            logger.info("Use: devalang check --entry ./examples");
            return Ok(());
        }

        logger.action(format!("Checking '{}'...", self.entry));

        let start = Instant::now();

        // Find entry file
        let entry_path = Path::new(&self.entry);
        if !entry_path.exists() {
            logger.error(format!("Entry path '{}' does not exist", self.entry));
            return Err(anyhow::anyhow!("Entry path not found"));
        }

        // Determine if it's a file or directory
        let files_to_check = if entry_path.is_file() {
            vec![entry_path.to_path_buf()]
        } else {
            // Find all .deva files in directory
            find_deva_files(entry_path)?
        };

        if files_to_check.is_empty() {
            logger.warn("No .deva files found");
            return Ok(());
        }

        logger.info(format!("Found {} file(s) to check", files_to_check.len()));

        // Parse all files
        let mut total_errors = 0;

        for file_path in &files_to_check {
            let file_display = file_path.display();

            match SimpleParser::parse_file(file_path) {
                Ok(statements) => {
                    if self.debug {
                        logger.debug(format!(
                            "✓ {} - {} statements",
                            file_display,
                            statements.len()
                        ));
                    }

                    // TODO: Type checking and validation could be added here
                    // For now, successful parsing = valid syntax
                }
                Err(e) => {
                    logger.error(format!("✗ {} - {}", file_display, e));
                    total_errors += 1;
                }
            }
        }

        let duration = start.elapsed();

        // Summary
        if total_errors > 0 {
            logger.error(format!(
                "Check failed with {} error(s) in {:.2?}",
                total_errors, duration
            ));
            return Err(anyhow::anyhow!("Syntax errors detected"));
        } else {
            logger.success(format!(
                "No errors detected. Checked {} file(s) in {:.2?}",
                files_to_check.len(),
                duration
            ));
        }

        Ok(())
    }
}

/// Recursively find all .deva files in a directory
fn find_deva_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                files.extend(find_deva_files(&path)?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("deva") {
                files.push(path);
            }
        }
    }

    Ok(files)
}
