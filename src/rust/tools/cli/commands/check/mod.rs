#![cfg(feature = "cli")]

use anyhow::Result;
use clap::Args;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Instant;

use crate::language::syntax::ast::{Statement, StatementKind, Value};
use crate::language::syntax::parser::driver::SimpleParser;
use crate::tools::cli::rules_reporter::RulesReporter;
use crate::tools::cli::state::CliContext;
use crate::tools::logger::structured_error::StructuredError;

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

    /// Disable rule checking during validation
    #[arg(long, default_value_t = false)]
    pub no_rule: bool,
}

impl CheckCommand {
    pub async fn execute(&self, ctx: &CliContext) -> Result<()> {
        if self.watch {
            return self.execute_watch(ctx).await;
        }

        self.execute_once(ctx)
    }

    fn execute_once(&self, ctx: &CliContext) -> Result<()> {
        let logger = ctx.logger();
        logger.action(format!("Checking '{}'...", self.entry));

        // Initialize rules reporter (only if rules not disabled)
        let rules_reporter = if !self.no_rule {
            Some(RulesReporter::from_current_dir(logger.clone())?)
        } else {
            None
        };

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

                    let invalid_statements = collect_invalid_statements(&statements);
                    total_errors += invalid_statements.len();
                    for statement in invalid_statements {
                        logger.log_structured_error(&structured_statement_error(
                            statement,
                            &file_display.to_string(),
                        ));
                    }

                    // Report on rules (var_keyword, deprecated_syntax, etc.) if enabled
                    if let Some(ref reporter) = rules_reporter {
                        let content = std::fs::read_to_string(file_path)?;
                        for (line_num, line) in content.lines().enumerate() {
                            let line_number = line_num + 1;

                            // Check for deprecated @-prefixed syntax
                            if line.trim_start().starts_with('@') {
                                if let Some(rule_msg) = reporter.checker().check_deprecated_syntax(
                                    line_number,
                                    "@ prefix syntax",
                                    "keyword syntax (import, export, use, load)",
                                ) {
                                    reporter.logger().log_rule_message(&rule_msg);
                                }
                            }
                        }
                    }
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

    async fn execute_watch(&self, ctx: &CliContext) -> Result<()> {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use std::time::Duration;

        let logger = ctx.logger();
        let entry_path = Path::new(&self.entry);
        if !entry_path.exists() {
            logger.error(format!("Entry path '{}' does not exist", self.entry));
            return Err(anyhow::anyhow!("Entry path not found"));
        }

        self.execute_once(ctx).ok();

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            move |result| {
                let _ = tx.send(result);
            },
            Config::default(),
        )?;

        let recursive_mode = if entry_path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(entry_path, recursive_mode)?;
        logger.info(format!("Watching '{}' for changes...", self.entry));

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if should_recheck(&event.kind) {
                        std::thread::sleep(Duration::from_millis(75));
                        if let Err(error) = self.execute_once(ctx) {
                            logger.error(format!("Watch check failed: {}", error));
                        }
                    }
                }
                Ok(Err(error)) => logger.error(format!("Watch error: {}", error)),
                Err(error) => return Err(anyhow::anyhow!("Watch channel closed: {}", error)),
            }
        }
    }
}

fn should_recheck(kind: &notify::EventKind) -> bool {
    matches!(
        kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Modify(_)
            | notify::EventKind::Remove(_)
            | notify::EventKind::Any
    )
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

fn collect_invalid_statements(statements: &[Statement]) -> Vec<&Statement> {
    let mut invalid = Vec::new();
    for statement in statements {
        collect_invalid_statement(statement, &mut invalid);
    }
    invalid
}

fn collect_invalid_statement<'a>(statement: &'a Statement, invalid: &mut Vec<&'a Statement>) {
    match &statement.kind {
        StatementKind::Unknown | StatementKind::Error { .. } => invalid.push(statement),
        StatementKind::Tempo {
            body: Some(body), ..
        }
        | StatementKind::Function { body, .. }
        | StatementKind::Group { body, .. }
        | StatementKind::Loop { body, .. }
        | StatementKind::For { body, .. }
        | StatementKind::Routing { body }
        | StatementKind::On { body, .. }
        | StatementKind::While { body, .. }
        | StatementKind::Section { body, .. } => {
            for child in body {
                collect_invalid_statement(child, invalid);
            }
        }
        StatementKind::If {
            body, else_body, ..
        } => {
            for child in body {
                collect_invalid_statement(child, invalid);
            }
            if let Some(else_body) = else_body {
                for child in else_body {
                    collect_invalid_statement(child, invalid);
                }
            }
        }
        StatementKind::Schedule { events } => {
            for (_, _, body) in events {
                for child in body {
                    collect_invalid_statement(child, invalid);
                }
            }
        }
        _ => {}
    }
}

fn structured_statement_error(statement: &Statement, fallback_file: &str) -> StructuredError {
    match &statement.kind {
        StatementKind::Error { message } => StructuredError::new(message.clone())
            .with_file(fallback_file.to_string())
            .with_location(statement.line, statement.column)
            .with_type("SyntaxError"),
        _ => {
            let (message, file, suggestion) =
                parse_unknown_message(&statement.value, fallback_file);
            let mut error = StructuredError::new(message)
                .with_file(file)
                .with_location(statement.line, statement.column)
                .with_type("UnknownStatement");

            if let Some(suggestion) = suggestion {
                error = error.with_suggestion(format!("Did you mean '{}' ?", suggestion));
            }

            error
        }
    }
}

fn parse_unknown_message(value: &Value, fallback_file: &str) -> (String, String, Option<String>) {
    let raw = match value {
        Value::String(message) => message.as_str(),
        _ => {
            return (
                "Unknown statement".to_string(),
                fallback_file.to_string(),
                None,
            );
        }
    };

    let mut parts = raw.split("|||");
    let message = parts.next().unwrap_or("Unknown statement").to_string();
    let file = parts.next().unwrap_or(fallback_file).to_string();
    let suggestion = parts
        .next()
        .filter(|s| !s.is_empty() && *s != "NO_SUGGESTION")
        .map(|s| s.to_string());

    (message, file, suggestion)
}
