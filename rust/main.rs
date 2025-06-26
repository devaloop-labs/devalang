pub mod core;
pub mod cli;
pub mod runner;
pub mod audio;
pub mod utils;

use std::{ io };
use clap::Parser;
use crate::{
    cli::{
        build::handle_build_command,
        check::handle_check_command,
        init::handle_init_command,
        template::{ handle_template_info_command, handle_template_list_command },
    },
    core::types::cli::{ Cli, CliCommands, CliTemplateCommand },
};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::Init { name, template } => {
            handle_init_command(name, template);
        }

        CliCommands::Template { command } =>
            match command {
                CliTemplateCommand::List => {
                    handle_template_list_command();
                }
                CliTemplateCommand::Info { name } => {
                    handle_template_info_command(name);
                }
            }

        CliCommands::Build { entry, output, watch, compilation_mode, debug, compress } => {
            handle_build_command(entry, output, watch);
        }

        CliCommands::Check { entry, output, watch, compilation_mode, debug } => {
            handle_check_command(entry, output, watch);
        }

        // TODO - Implement the play command
        // CliCommands::Play {} => {
        //     log_message("Command 'play' is not implemented yet.", "WARNING");
        // }
    }

    Ok(())
}
