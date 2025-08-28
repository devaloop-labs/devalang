#![cfg(feature = "cli")]

pub mod cli;
pub mod common;
pub mod config;
pub mod core;
pub mod installer;
pub mod utils;

use crate::{
    cli::{
        bank::{
            handle_bank_available_command, handle_bank_info_command, handle_bank_list_command,
            handle_remove_bank_command, handle_update_bank_command,
        },
        build::handle_build_command,
        check::handle_check_command,
        driver::{BankCommand, Cli, Commands, InstallCommand, TelemetryCommand, TemplateCommand},
        init::handle_init_command,
        install::handle_install_command,
        login::handle_login_command,
        play::handle_play_command,
        telemetry::{handle_telemetry_disable_command, handle_telemetry_enable_command},
        template::{handle_template_info_command, handle_template_list_command},
        update::handle_update_command,
    },
    config::{driver::ProjectConfig, loader::load_config},
    installer::addon::AddonType,
    utils::{first_usage::check_is_first_usage, telemetry::TelemetryEventCreator},
};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();
    let mut config: Option<ProjectConfig> = None;

    let duration = std::time::Instant::now();

    check_is_first_usage();

    let telemetry_event_creator = TelemetryEventCreator::new();
    let mut event = telemetry_event_creator.get_base_event();
    let mut had_error: bool = false;
    let mut last_error_message: Option<String> = None;
    let mut exit_code: Option<i32> = None;

    if !cli.no_config {
        config = load_config(None);
    } else {
        println!("No configuration file loaded. Running with arguments only.");
    }

    match cli.command {
        Commands::Init { name, template } => {
            handle_init_command(name, template);
        }

        Commands::Template { command } => match command {
            TemplateCommand::List => {
                handle_template_list_command();
            }
            TemplateCommand::Info { name } => {
                handle_template_info_command(name);
            }
        },

        Commands::Check {
            entry,
            output,
            watch,
            debug,
        } => {
            if let Err(err) = handle_check_command(config, entry, output, watch, debug) {
                eprintln!("❌ Check failed: {}", err);
                had_error = true;
                last_error_message = Some(format!("check failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Build {
            entry,
            output,
            watch,
            debug,
            compress,
        } => {
            if let Err(err) = handle_build_command(config, entry, output, watch, debug, compress) {
                eprintln!("❌ Build failed: {}", err);
                had_error = true;
                last_error_message = Some(format!("build failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Play {
            entry,
            output,
            watch,
            repeat,
            debug,
        } => {
            if let Err(err) = handle_play_command(config, entry, output, watch, repeat, debug) {
                eprintln!("❌ Play failed: {}", err);
                had_error = true;
                last_error_message = Some(format!("play failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Install { command } => match command {
            InstallCommand::Bank { name } => {
                if let Err(err) = handle_install_command(name, AddonType::Bank).await {
                    eprintln!("❌ Failed to install bank: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("install bank failed: {}", err));
                    exit_code = Some(1);
                }
            }
            InstallCommand::Plugin { name } => {
                if let Err(err) = handle_install_command(name, AddonType::Plugin).await {
                    eprintln!("❌ Failed to install plugin: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("install plugin failed: {}", err));
                    exit_code = Some(1);
                }
            }
            InstallCommand::Preset { name } => {
                if let Err(err) = handle_install_command(name, AddonType::Preset).await {
                    eprintln!("❌ Failed to install preset: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("install preset failed: {}", err));
                    exit_code = Some(1);
                }
            }
        },

        Commands::Bank { command } => match command {
            BankCommand::List => {
                if let Err(err) = handle_bank_list_command().await {
                    eprintln!("❌ Failed to list local banks: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("bank list failed: {}", err));
                    exit_code = Some(1);
                }
            }

            BankCommand::Available => {
                if let Err(err) = handle_bank_available_command().await {
                    eprintln!("❌ Failed to list available banks: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("bank available failed: {}", err));
                    exit_code = Some(1);
                }
            }

            BankCommand::Info { name } => {
                if let Err(err) = handle_bank_info_command(name).await {
                    eprintln!("❌ Failed to get bank info: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("bank info failed: {}", err));
                    exit_code = Some(1);
                }
            }

            BankCommand::Remove { name } => {
                if let Err(err) = handle_remove_bank_command(name).await {
                    eprintln!("❌ Failed to remove bank: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("bank remove failed: {}", err));
                    exit_code = Some(1);
                }
            }

            BankCommand::Update { name } => {
                if let Err(err) = handle_update_bank_command(name).await {
                    eprintln!("❌ Failed to update bank: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("bank update failed: {}", err));
                    exit_code = Some(1);
                }
            }
        },

        Commands::Update { only } => {
            if let Err(err) = handle_update_command(only).await {
                eprintln!("❌ Update failed: {}", err);
                had_error = true;
                last_error_message = Some(format!("update failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Telemetry { command } => match command {
            TelemetryCommand::Enable { .. } => {
                if let Err(err) = handle_telemetry_enable_command().await {
                    eprintln!("❌ Failed to enable telemetry: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("telemetry enable failed: {}", err));
                    exit_code = Some(1);
                }
            }
            TelemetryCommand::Disable { .. } => {
                if let Err(err) = handle_telemetry_disable_command().await {
                    eprintln!("❌ Failed to disable telemetry: {}", err);
                    had_error = true;
                    last_error_message = Some(format!("telemetry disable failed: {}", err));
                    exit_code = Some(1);
                }
            }
        },

        Commands::Login { .. } => {
            if let Err(err) = handle_login_command().await {
                eprintln!("❌ Login failed: {}", err);
                had_error = true;
                last_error_message = Some(format!("login failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Logout { .. } => {
            eprintln!("❌ Logout command is not implemented yet.");
            had_error = true;
            last_error_message = Some("logout not implemented".to_string());
            exit_code = Some(1);
        }
    }

    // SECTION Telemetry
    event.set_timestamp(chrono::Utc::now().to_string());
    event.set_duration(duration.elapsed().as_millis() as u64);
    event.set_success(!had_error);

    if had_error {
        event.set_error(
            utils::telemetry::TelemetryErrorLevel::Critical,
            last_error_message,
            exit_code,
        );
    }

    utils::telemetry::refresh_event_project_info(&mut event);

    let _ = utils::telemetry::send_telemetry_event(&event).await;

    Ok(())
}
