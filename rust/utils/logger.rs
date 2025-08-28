#[cfg(feature = "cli")]
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Success,
    Error,
    Info,
    Print,
    Warning,
    Watcher,
    Debug,
}

#[derive(Debug, Clone)]
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Logger
    }

    // --- log_message ---

    #[cfg(feature = "cli")]
    pub fn log_message(&self, level: LogLevel, message: &str) {
        let formatted_status = self.format_status(level);
        println!(
            "🦊 {} {} {}",
            self.language_signature(),
            formatted_status,
            message
        );
    }

    #[cfg(not(feature = "cli"))]
    pub fn log_message(&self, _level: LogLevel, _message: &str) {
        // no-op for WASM
    }

    // --- log_message_with_trace ---

    #[cfg(feature = "cli")]
    pub fn log_message_with_trace(&self, level: LogLevel, message: &str, trace: Vec<&str>) {
        let formatted_status = self.format_status(level);
        println!(
            "🦊 {} {} {}",
            self.language_signature(),
            formatted_status,
            message
        );
        for t in trace {
            println!("     ↳ {}", t);
        }
    }

    #[cfg(not(feature = "cli"))]
    pub fn log_message_with_trace(&self, _level: LogLevel, _message: &str, _trace: Vec<&str>) {
        // no-op for WASM
    }

    // --- log_error_with_stacktrace ---

    #[cfg(feature = "cli")]
    pub fn log_error_with_stacktrace(&self, message: &str, stacktrace: &str) {
        let formatted_status = self.format_status(LogLevel::Error);
        println!(
            "🦊 {} {} {}",
            self.language_signature(),
            formatted_status,
            message
        );
        println!("     ↳ {}", stacktrace);
    }

    #[cfg(not(feature = "cli"))]
    pub fn log_error_with_stacktrace(&self, _message: &str, _stacktrace: &str) {
        // no-op for WASM
    }

    // --- language_signature ---

    #[cfg(feature = "cli")]
    fn language_signature(&self) -> String {
        let mut s = String::new();

        write!(&mut s, "{}", SetForegroundColor(Color::Grey)).unwrap();
        s.push('[');

        write!(
            &mut s,
            "{}",
            SetForegroundColor(Color::Rgb {
                r: 29,
                g: 211,
                b: 176
            })
        )
        .unwrap();
        write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
        s.push_str("Devalang");
        write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();

        write!(&mut s, "{}", SetForegroundColor(Color::Grey)).unwrap();
        s.push(']');
        write!(&mut s, "{}", ResetColor).unwrap();

        s
    }

    #[cfg(not(feature = "cli"))]
    fn language_signature(&self) -> String {
        "[Devalang]".to_string()
    }

    // --- format_status ---

    #[cfg(feature = "cli")]
    fn format_status(&self, level: LogLevel) -> String {
        let mut s = String::new();

        let color = match level {
            LogLevel::Success => Color::Rgb {
                r: 76,
                g: 175,
                b: 80,
            },
            LogLevel::Error => Color::Rgb {
                r: 244,
                g: 67,
                b: 54,
            },
            LogLevel::Info => Color::Rgb {
                r: 33,
                g: 150,
                b: 243,
            },
            LogLevel::Warning => Color::Rgb {
                r: 255,
                g: 152,
                b: 0,
            },
            LogLevel::Watcher => Color::Rgb {
                r: 156,
                g: 39,
                b: 176,
            },
            LogLevel::Debug => Color::Rgb {
                r: 103,
                g: 58,
                b: 183,
            },
            LogLevel::Print => Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        };

        let status = match level {
            LogLevel::Success => "SUCCESS",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Watcher => "WATCHER",
            LogLevel::Debug => "DEBUG",
            LogLevel::Print => "PRINT",
        };

        s.push('[');
        write!(&mut s, "{}", SetForegroundColor(color)).unwrap();
        write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
        s.push_str(status);
        write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();
        s.push(']');
        write!(&mut s, "{}", ResetColor).unwrap();

        s
    }

    #[cfg(not(feature = "cli"))]
    fn format_status(&self, level: LogLevel) -> String {
        match level {
            LogLevel::Success => "[SUCCESS]",
            LogLevel::Error => "[ERROR]",
            LogLevel::Info => "[INFO]",
            LogLevel::Warning => "[WARNING]",
            LogLevel::Watcher => "[WATCHER]",
            LogLevel::Debug => "[DEBUG]",
            LogLevel::Print => "[PRINT]",
        }
        .to_string()
    }
}
