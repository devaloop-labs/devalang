#[cfg(feature = "cli")]
use crossterm::style::{Attribute, SetAttribute};

#[cfg(feature = "cli")]
use std::fmt::Write;

use crate::logger::{LogLevel, Logger};
use crate::signature::get_signature;
use crate::version::get_version;
use std::env;
use std::path::PathBuf;

fn get_devalang_homedir() -> PathBuf {
    // Prefer explicit env var, then HOME/USERPROFILE, fallback to current dir
    if let Ok(p) = env::var("DEVALANG_HOME") {
        return PathBuf::from(p);
    }

    if let Ok(p) = env::var("HOME") {
        return PathBuf::from(p).join(".devalang");
    }

    if let Ok(p) = env::var("USERPROFILE") {
        return PathBuf::from(p).join(".devalang");
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".devalang")
}

pub fn check_is_first_usage() -> bool {
    if get_devalang_homedir().exists() == true {
        false
    } else {
        first_usage_welcome();
        true
    }
}

pub fn first_usage_welcome() {
    std::fs::create_dir_all(get_devalang_homedir()).ok();

    let version = get_version();
    print!("{}", get_signature(&version));

    let homedir = get_devalang_homedir().display().to_string();

    let welcome_msg = format!(
        "Welcome to Devalang ! \n\
        It looks like this is your first time using the tool.\n\
        A configuration file will be created in your home directory.\n\
        (location: '{}')",
        homedir
    );

    #[cfg(feature = "cli")]
    let mut s = String::new();
    #[cfg(feature = "cli")]
    {
        write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
        write!(&mut s, "{}", welcome_msg).unwrap();
        write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();

        println!("");
        println!("{}", s);
        println!("");
    }

    #[cfg(not(feature = "cli"))]
    {
        // Fallback: plain output on non-cli (wasm) builds
        println!("");
        println!("{}", welcome_msg);
        println!("");
    }

    first_usage_ask_for_telemetry();
}

pub fn first_usage_ask_for_telemetry() {
    let telemetry_msg = "Would you like to enable anonymous telemetry ?";
    let telemetry_desc = "This data helps us improve the tool. You can opt-out at any time.";

    // Non-interactive fallback for first usage: default to telemetry disabled.
    let _ = telemetry_msg;
    let _ = telemetry_desc;

    let logger = Logger::new();

    println!("");
    logger.log_message(
        LogLevel::Info,
        "Telemetry disabled by default. You can enable it at any time by using 'devalang telemetry enable'"
    );
    println!("");
}
