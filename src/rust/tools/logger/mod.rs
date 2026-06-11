#[cfg(feature = "cli")]
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
#[cfg(feature = "cli")]
use std::fmt::Write;

pub mod rule_checker;
pub use rule_checker::{RuleChecker, RuleMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Success,
    Error,
    Info,
    Warning,
    Watch,
    Debug,
    Print,
    Action,
}

#[derive(Debug, Clone, Default)]
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self
    }

    /// Log a rule message with appropriate severity handling
    pub fn log_rule_message(&self, rule_msg: &RuleMessage) {
        match rule_msg.level {
            crate::platform::config::RuleLevel::Error => {
                self.error(&rule_msg.formatted());
            }
            crate::platform::config::RuleLevel::Warning => {
                self.warn(&rule_msg.formatted());
            }
            crate::platform::config::RuleLevel::Info => {
                self.info(&rule_msg.formatted());
            }
            crate::platform::config::RuleLevel::Off => {
                // Silent
            }
        }
    }

    pub fn log(&self, level: LogLevel, message: impl AsRef<str>) {
        self.print_line(level, message.as_ref());
    }

    pub fn log_with_details<I, S>(&self, level: LogLevel, message: impl AsRef<str>, details: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.print_line(level, message.as_ref());
        for detail in details {
            self.print_detail(detail.as_ref());
        }
    }

    /// Log a structured error with formatted details including file location, type, and suggestions
    pub fn log_structured_error(&self, error: &StructuredError) {
        self.log(LogLevel::Error, &error.message);
        let colored_details = error.build_colored_details();
        for (label, content) in colored_details {
            self.print_colored_detail(&label, &content);
        }
    }

    pub fn success(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Success, message);
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Error, message);
    }

    pub fn watch(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Watch, message);
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Debug, message);
    }

    pub fn action(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Action, message);
    }

    /// Print-level messages coming from user `print` statements. Rendered as [PRINT]
    /// in plain/CLI output. Convenience wrapper around `log`.
    pub fn print(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Print, message);
    }

    fn print_detail(&self, detail: &str) {
        #[cfg(feature = "cli")]
        {
            println!("   ↳ {}", detail);
        }
        #[cfg(not(feature = "cli"))]
        {
            println!("   -> {}", detail);
        }
    }

    /// Print a colored detail line with a label
    /// Format: "   ↳ label: content" where label is in medium-dark grey
    #[cfg(feature = "cli")]
    fn print_colored_detail(&self, label: &str, content: &str) {
        let mut output = String::new();
        output.push_str("   ↳ ");

        // Label in medium-dark grey (RGB 110, 110, 110) with bold
        output.push_str(&format!(
            "{}{}{}{}",
            SetForegroundColor(Color::Rgb {
                r: 110,
                g: 110,
                b: 110
            }),
            SetAttribute(Attribute::Bold),
            label,
            SetAttribute(Attribute::Reset)
        ));

        // Colon separator
        output.push_str(": ");

        // Content in normal color (white)
        output.push_str(&format!("{}{}", SetForegroundColor(Color::White), content));

        output.push_str(&format!("{}", ResetColor));

        println!("{}", output);
    }

    /// Print a colored detail line with a label (non-CLI version)
    #[cfg(not(feature = "cli"))]
    fn print_colored_detail(&self, label: &str, content: &str) {
        println!("   -> {}: {}", label, content);
    }

    fn print_line(&self, level: LogLevel, message: &str) {
        #[cfg(feature = "cli")]
        {
            println!("{}", self.render_colored_line(level, message));
        }
        #[cfg(not(feature = "cli"))]
        {
            println!("[{}] {}", level.as_plain_label(), message);
        }
    }

    #[cfg(feature = "cli")]
    fn render_colored_line(&self, level: LogLevel, message: &str) -> String {
        let mut out = String::new();
        let (emoji, color) = level.visuals();

        out.push_str(emoji);
        out.push(' ');
        out.push_str(&self.render_signature());
        out.push(' ');
        out.push_str(&self.render_status(level, color));
        out.push(' ');
        out.push_str(message);
        out
    }

    #[cfg(feature = "cli")]
    fn render_signature(&self) -> String {
        let mut s = String::new();
        let _ = write!(&mut s, "{}", SetForegroundColor(Color::Grey));
        s.push('[');
        let _ = write!(
            &mut s,
            "{}",
            SetForegroundColor(Color::Rgb {
                r: 36,
                g: 199,
                b: 181,
            })
        );
        let _ = write!(&mut s, "{}", SetAttribute(Attribute::Bold));
        s.push_str("Devalang");
        let _ = write!(&mut s, "{}", SetAttribute(Attribute::Reset));
        let _ = write!(&mut s, "{}", SetForegroundColor(Color::Grey));
        s.push(']');
        let _ = write!(&mut s, "{}", ResetColor);
        s
    }

    #[cfg(feature = "cli")]
    fn render_status(&self, level: LogLevel, color: Color) -> String {
        let mut s = String::new();
        let _ = write!(&mut s, "{}", SetForegroundColor(color));
        let _ = write!(&mut s, "{}", SetAttribute(Attribute::Bold));
        s.push('[');
        s.push_str(level.as_label());
        s.push(']');
        let _ = write!(&mut s, "{}", SetAttribute(Attribute::Reset));
        let _ = write!(&mut s, "{}", ResetColor);
        s
    }
}

impl LogLevel {
    fn as_label(self) -> &'static str {
        match self {
            LogLevel::Success => "SUCCESS",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Watch => "WATCH",
            LogLevel::Debug => "DEBUG",
            LogLevel::Action => "ACTION",
            LogLevel::Print => "PRINT",
        }
    }

    fn as_plain_label(self) -> &'static str {
        match self {
            LogLevel::Success => "SUCCESS",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Watch => "WATCH",
            LogLevel::Debug => "DEBUG",
            LogLevel::Action => "ACTION",
            LogLevel::Print => "PRINT",
        }
    }

    #[cfg(feature = "cli")]
    fn visuals(self) -> (&'static str, Color) {
        match self {
            LogLevel::Success => (
                "✅",
                Color::Rgb {
                    r: 76,
                    g: 175,
                    b: 80,
                },
            ),
            LogLevel::Error => (
                "❌",
                Color::Rgb {
                    r: 244,
                    g: 67,
                    b: 54,
                },
            ),
            LogLevel::Info => (
                "ℹ️ ",
                Color::Rgb {
                    r: 33,
                    g: 150,
                    b: 243,
                },
            ),
            LogLevel::Warning => (
                "⚠️",
                Color::Rgb {
                    r: 255,
                    g: 152,
                    b: 0,
                },
            ),
            LogLevel::Watch => (
                "👀",
                Color::Rgb {
                    r: 171,
                    g: 71,
                    b: 188,
                },
            ),
            LogLevel::Debug => (
                "🛠️",
                Color::Rgb {
                    r: 121,
                    g: 134,
                    b: 203,
                },
            ),
            LogLevel::Action => (
                "🎵",
                Color::Rgb {
                    r: 0,
                    g: 188,
                    b: 212,
                },
            ),
            LogLevel::Print => ("", Color::White),
        }
    }
}

pub mod format;
pub mod layers;
pub mod sinks;
pub mod structured_error;

pub use structured_error::StructuredError;
