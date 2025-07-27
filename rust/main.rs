#![cfg(feature = "cli")]

pub mod core;
pub mod cli;
pub mod utils;
pub mod config;
pub mod common;
pub mod installer;

use std::io;
use clap::Parser;
use crate::{
    cli::{
        bank::{
            handle_bank_available_command,
            handle_bank_info_command,
            handle_bank_list_command,
            handle_remove_bank_command,
            handle_update_bank_command,
        },
        build::handle_build_command,
        check::handle_check_command,
        driver::{ BankCommand, Cli, Commands, InstallCommand, TemplateCommand },
        init::handle_init_command,
        install::handle_install_command,
        play::handle_play_command,
        template::{ handle_template_info_command, handle_template_list_command },
        update::handle_update_command,
    },
    config::{ driver::Config, loader::load_config },
    installer::addon::AddonType,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();
    let mut config: Option<Config> = None;

    if !cli.no_config {
        config = load_config(None);
    } else {
        println!("No configuration file loaded. Running with arguments only.");
    }

    match cli.command {
        Commands::Init { name, template } => {
            handle_init_command(name, template);
        }

        Commands::Template { command } =>
            match command {
                TemplateCommand::List => {
                    handle_template_list_command();
                }
                TemplateCommand::Info { name } => {
                    handle_template_info_command(name);
                }
            }

        Commands::Check { entry, output, watch, debug } => {
            handle_check_command(config, entry, output, watch, debug);
        }

        Commands::Build { entry, output, watch, debug, compress } => {
            handle_build_command(config, entry, output, watch, debug, compress);
        }

        Commands::Play { entry, output, watch, repeat, debug } => {
            handle_play_command(config, entry, output, watch, repeat, debug);
        }

        Commands::Install { command } =>
            match command {
                InstallCommand::Bank { name } => {
                    if let Err(err) = handle_install_command(name, AddonType::Bank).await {
                        eprintln!("❌ Failed to install bank: {}", err);
                    }
                }
                InstallCommand::Plugin { name } => {
                    if let Err(err) = handle_install_command(name, AddonType::Plugin).await {
                        eprintln!("❌ Failed to install plugin: {}", err);
                    }
                }
                InstallCommand::Preset { name } => {
                    if let Err(err) = handle_install_command(name, AddonType::Preset).await {
                        eprintln!("❌ Failed to install preset: {}", err);
                    }
                }
            }

        Commands::Bank { command } =>
            match command {
                BankCommand::List => {
                    if let Err(err) = handle_bank_list_command().await {
                        eprintln!("❌ Failed to list local banks: {}", err);
                    }
                }

                BankCommand::Available => {
                    if let Err(err) = handle_bank_available_command().await {
                        eprintln!("❌ Failed to list available banks: {}", err);
                    }
                }

                BankCommand::Info { name } => {
                    if let Err(err) = handle_bank_info_command(name).await {
                        eprintln!("❌ Failed to get bank info: {}", err);
                    }
                }

                BankCommand::Remove { name } => {
                    if let Err(err) = handle_remove_bank_command(name).await {
                        eprintln!("❌ Failed to remove bank: {}", err);
                    }
                }

                BankCommand::Update { name } => {
                    if let Err(err) = handle_update_bank_command(name).await {
                        eprintln!("❌ Failed to update bank: {}", err);
                    }
                }
            }

        Commands::Update { only } => {
            if let Err(err) = handle_update_command(only).await {
                eprintln!("❌ Update failed: {}", err);
            }
        }

        _ => {}
    }

    Ok(())
}
