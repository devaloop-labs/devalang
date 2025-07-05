#![cfg(feature = "cli")]

pub mod core;
pub mod cli;
pub mod utils;
pub mod config;
pub mod audio;

use std::io;
use cli::{ Cli };
use clap::Parser;
use crate::{
    cli::{
        build::handle_build_command,
        check::handle_check_command,
        init::handle_init_command,
        play::handle_play_command,
        template::{ handle_template_info_command, handle_template_list_command },
        Commands,
        TemplateCommand,
    },
    config::{ loader::load_config, Config },
};

fn main() -> io::Result<()> {
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

        Commands::Check { entry, output, watch, compilation_mode, debug } => {
            handle_check_command(config, entry, output, watch);
        }

        Commands::Build { entry, output, watch, compilation_mode, debug, compress } => {
            handle_build_command(config, entry, output, watch);
        }

        Commands::Play { entry, output, watch, repeat } => {
            handle_play_command(config, entry, output, watch, repeat);
        }

        _ => {}
    }

    Ok(())
}
