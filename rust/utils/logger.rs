use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use std::{ fmt::Write };

pub fn log_message(message: &str, status: &str) {
    let formatted_status = format_status(status);
    println!("🦊 {} {} {}", language_signature(), formatted_status, message);
}

fn language_signature() -> String {
    let mut s = String::new();

    write!(&mut s, "{}", SetForegroundColor(Color::Grey)).unwrap();
    s.push('[');

    write!(&mut s, "{}", SetForegroundColor(Color::Rgb { r: 29, g: 211, b: 176 })).unwrap();
    write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
    s.push_str("Devalang");
    write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();

    write!(&mut s, "{}", SetForegroundColor(Color::Grey)).unwrap();
    s.push(']');

    write!(&mut s, "{}", ResetColor).unwrap();

    s
}

fn format_status(status: &str) -> String {
    let mut s = String::new();

    let color = match status {
        "SUCCESS" => Color::Rgb { r: 76, g: 175, b: 80 },
        "ERROR" => Color::Rgb { r: 244, g: 67, b: 54 },
        "INFO" => Color::Rgb { r: 33, g: 150, b: 243 },
        "WARNING" => Color::Rgb { r: 255, g: 152, b: 0 },
        _ => Color::Grey,
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
