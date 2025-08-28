use crossterm::style::{Attribute, SetAttribute};
use std::fmt::Write;

use crate::{
    config::settings::{get_devalang_homedir, set_user_config_bool, write_user_config_file},
    utils::signature::get_signature,
};

pub fn check_is_first_usage() {
    if get_devalang_homedir()
        .join("config.json")
        .try_exists()
        .is_ok()
    {
        // Do nothing
    } else {
        first_usage_welcome();
        write_user_config_file();
    }
}

pub fn first_usage_welcome() {
    let version = env!("CARGO_PKG_VERSION");
    print!("{}", get_signature(version));

    let mut s = String::new();
    let welcome_msg = format!(
        "Welcome to Devalang ! \n\
        It looks like this is your first time using the tool.\n\
        A configuration file will be created in your home directory.\n\
        ({})",
        get_devalang_homedir().display()
    );

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

    if telemetry_response == true {
        println!("");
        println!(
            "Telemetry enabled. You can opt-out at any time by using 'devalang telemetry disable'"
        );
        println!("");

        set_user_config_bool("telemetry", true);
    } else {
        println!("");
        println!(
            "Telemetry disabled. You can enable it at any time by using 'devalang telemetry enable'"
        );
        println!("");

        set_user_config_bool("telemetry", false);
    }
}
