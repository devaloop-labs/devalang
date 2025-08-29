use crossterm::style::{Attribute, SetAttribute};
use std::fmt::Write;

use crate::{
    config::settings::{get_devalang_homedir, set_user_config_bool, write_user_config_file},
    utils::{
        logger::{LogLevel, Logger},
        signature::get_signature,
    },
};

pub fn check_is_first_usage() {
    if get_devalang_homedir().exists() == true {
        // Do nothing
    } else {
        first_usage_welcome();
        write_user_config_file();
    }
}

pub fn first_usage_welcome() {
    std::fs::create_dir_all(get_devalang_homedir()).ok();

    let version = env!("CARGO_PKG_VERSION");
    print!("{}", get_signature(version));

    let homedir = get_devalang_homedir().display().to_string();

    let welcome_msg = format!(
        "Welcome to Devalang ! \n\
        It looks like this is your first time using the tool.\n\
        A configuration file will be created in your home directory.\n\
        (location: '{}')",
        homedir
    );

    let mut s = String::new();
    write!(&mut s, "{}", SetAttribute(Attribute::Bold)).unwrap();
    write!(&mut s, "{}", welcome_msg).unwrap();
    write!(&mut s, "{}", SetAttribute(Attribute::Reset)).unwrap();

    println!("");
    println!("{}", s);
    println!("");

    first_usage_ask_for_telemetry();
}

pub fn first_usage_ask_for_telemetry() {
    let telemetry_msg = "Would you like to enable anonymous telemetry ?";
    let telemetry_desc = "This data helps us improve the tool. You can opt-out at any time.";

    let telemetry_prompt = inquire::Confirm::new(telemetry_msg)
        .with_help_message(telemetry_desc)
        .with_default(false)
        .prompt();

    let telemetry_response = telemetry_prompt.unwrap_or(false);

    write_user_config_file();

    let logger = Logger::new();

    if telemetry_response == true {
        println!("");
        logger.log_message(
            LogLevel::Info,
            "Telemetry enabled. You can opt-out at any time by using 'devalang telemetry disable'",
        );
        println!("");

        set_user_config_bool("telemetry", true);
    } else {
        println!("");
        logger.log_message(
            LogLevel::Info,
            "Telemetry disabled. You can enable it at any time by using 'devalang telemetry enable'"
        );
        println!("");

        set_user_config_bool("telemetry", false);
    }
}
