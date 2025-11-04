use crate::platform::config::AppConfig;
use crate::tools::logger::{Logger, RuleChecker};
/// Helper module to activate and report rules in CLI commands
use std::sync::Arc;

pub struct RulesReporter {
    rule_checker: RuleChecker,
    logger: Arc<Logger>,
}

impl RulesReporter {
    /// Create a new rules reporter from config
    pub fn new(config: AppConfig, logger: Arc<Logger>) -> Self {
        let rule_checker = RuleChecker::new(config);
        Self {
            rule_checker,
            logger,
        }
    }

    /// Check and report on all parsing rules for a file
    pub fn report_check_results(&self, _filename: &str, _line_number: usize) {
        // This can be extended with actual parsing results
        // For now, it's a placeholder for future integration
    }

    /// Create a reporter from current directory config
    pub fn from_current_dir(logger: Arc<Logger>) -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let config = AppConfig::load(&current_dir)?;
        Ok(Self::new(config, logger))
    }

    /// Get the internal rule checker
    pub fn checker(&self) -> &RuleChecker {
        &self.rule_checker
    }

    /// Get the logger
    pub fn logger(&self) -> &Arc<Logger> {
        &self.logger
    }
}
