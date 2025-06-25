pub mod core;
pub mod cli;
pub mod runner;
pub mod audio;
pub mod utils;

use std::{ io };
use clap::Parser;
use crate::{
    cli::{ build::handle_build_command, check::handle_check_command },
    core::types::cli::{ Cli, CliCommands },
};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // TODO - Implement the new command
        // CliCommands::New { name, template } => {
        //     log_message("Command 'new project' is not implemented yet.", "WARNING");
        // }

        // TODO - Implement the template command
        // CliCommands::Template { command } =>
        //     match command {
        //         CliTemplateCommand::List => {
        //             log_message("Command 'template list' is not implemented yet.", "WARNING");
        //         }
        //         CliTemplateCommand::Info { name } => {
        //             log_message("Command 'template info' is not implemented yet.", "WARNING");
        //         }
        //     }

        CliCommands::Build { entry, output, watch, compilation_mode, debug, compress } => {
            handle_build_command(entry, output);
        }

        CliCommands::Check { entry, output, watch, compilation_mode, debug } => {
            handle_check_command(entry, output);
        }

        // TODO - Implement the play command
        // CliCommands::Play {} => {
        //     log_message("Command 'play' is not implemented yet.", "WARNING");
        // }
    }

    Ok(())
}
