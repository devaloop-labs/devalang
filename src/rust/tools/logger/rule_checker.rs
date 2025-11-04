/// Rule-based logging system for handling code quality rules
///
/// This module provides a system for enforcing coding rules at different severity levels:
/// - Error: Compilation should fail
/// - Warning: Compilation continues but warning is shown
/// - Info: Informational message only
/// - Off: Rule is disabled
use crate::platform::config::{AppConfig, RuleLevel};

pub struct RuleChecker {
    config: AppConfig,
}

impl RuleChecker {
    /// Create a new rule checker from configuration
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Check if explicit durations are required
    pub fn check_explicit_duration(
        &self,
        line_number: usize,
        context: &str,
    ) -> Option<RuleMessage> {
        let level = self.config.rules.explicit_durations;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "explicit_durations",
            message: format!(
                "Line {}: Duration not explicitly specified. {}",
                line_number, context
            ),
        })
    }

    /// Check if deprecated syntax is used
    pub fn check_deprecated_syntax(
        &self,
        line_number: usize,
        old_syntax: &str,
        new_syntax: &str,
    ) -> Option<RuleMessage> {
        let level = self.config.rules.deprecated_syntax;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "deprecated_syntax",
            message: format!(
                "Line {}: '{}' is deprecated, use '{}' instead",
                line_number, old_syntax, new_syntax
            ),
        })
    }

    /// Check if 'var' keyword is used
    pub fn check_var_keyword(&self, line_number: usize) -> Option<RuleMessage> {
        let level = self.config.rules.var_keyword;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "var_keyword",
            message: format!(
                "Line {}: 'var' keyword is not allowed, use 'let' instead",
                line_number
            ),
        })
    }

    /// Check for missing duration
    pub fn check_missing_duration(&self, line_number: usize, element: &str) -> Option<RuleMessage> {
        let level = self.config.rules.missing_duration;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "missing_duration",
            message: format!(
                "Line {}: '{}' might benefit from an explicit duration",
                line_number, element
            ),
        })
    }

    /// Check for implicit type conversion
    pub fn check_implicit_conversion(
        &self,
        line_number: usize,
        from_type: &str,
        to_type: &str,
    ) -> Option<RuleMessage> {
        let level = self.config.rules.implicit_type_conversion;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "implicit_type_conversion",
            message: format!(
                "Line {}: Implicit conversion from '{}' to '{}'. Consider explicit conversion",
                line_number, from_type, to_type
            ),
        })
    }

    /// Check for unused variables
    pub fn check_unused_variable(&self, line_number: usize, var_name: &str) -> Option<RuleMessage> {
        let level = self.config.rules.unused_variables;
        if !level.should_report() {
            return None;
        }

        Some(RuleMessage {
            level,
            rule_name: "unused_variables",
            message: format!(
                "Line {}: Variable '{}' is defined but never used",
                line_number, var_name
            ),
        })
    }
}

/// A rule violation message with associated severity level
#[derive(Debug, Clone)]
pub struct RuleMessage {
    pub level: RuleLevel,
    pub rule_name: &'static str,
    pub message: String,
}

impl RuleMessage {
    /// Format the message for display with level indicator
    pub fn formatted(&self) -> String {
        let level_str = match self.level {
            RuleLevel::Error => "ERROR",
            RuleLevel::Warning => "WARNING",
            RuleLevel::Info => "INFO",
            RuleLevel::Off => "OFF",
        };

        format!("[{}] {}: {}", level_str, self.rule_name, self.message)
    }

    /// Get the ANSI color code for the level
    pub fn color_code(&self) -> &'static str {
        match self.level {
            RuleLevel::Error => "\x1b[91m",   // Bright red
            RuleLevel::Warning => "\x1b[93m", // Bright yellow
            RuleLevel::Info => "\x1b[94m",    // Bright blue
            RuleLevel::Off => "\x1b[37m",     // White
        }
    }

    /// Get the reset color code
    pub fn reset_color() -> &'static str {
        "\x1b[0m"
    }

    /// Format with ANSI colors
    pub fn colored(&self) -> String {
        format!(
            "{}{}{}",
            self.color_code(),
            self.formatted(),
            Self::reset_color()
        )
    }
}
