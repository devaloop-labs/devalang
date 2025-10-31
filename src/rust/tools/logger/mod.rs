#[cfg(feature = "cli")]
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
#[cfg(feature = "cli")]
use std::fmt::Write;

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
        write!(&mut s, "{}", SetForegroundColor(Color::Grey)).unwrap();
        s.push('[');
        write!(
            &mut s,
            "{}",
            SetForegroundColor(Color::Rgb {
                r: 36,
                g: 199,
                b: 181,
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

    #[cfg(feature = "cli")]
    fn render_status(&self, level: LogLevel, color: Color) -> String {
        let mut s = String::new();
        write!(&mut s, "{}", SetForegroundColor(color)).unwrap();
        write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
        s.push('[');
        s.push_str(level.as_label());
        s.push(']');
        write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();
        write!(&mut s, "{}", ResetColor).unwrap();
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
