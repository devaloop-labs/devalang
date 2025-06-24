pub mod core;
pub mod cli;
pub mod runner;
pub mod audio;
pub mod utils;

use std::{ io };
use clap::Parser;
use crate::{
    cli::{ build::handle_build_command, check::handle_check_command },
    core::types::cli::{ Cli, CliCommands, CliTemplateCommand },
};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::New { name, template } => {
            // handle_new_command(&name, &template);
            panic!("Command 'new project' is not implemented yet.");
        }

        CliCommands::Template { command } =>
            match command {
                CliTemplateCommand::List => {
                    // handle_template_list();
                    panic!("Command 'template list' is not implemented yet.");
                }
                CliTemplateCommand::Info { name } => {
                    // handle_template_info(&name);
                    panic!("Command 'template info' is not implemented yet.");
                }
            }

        CliCommands::Build { entry, output, watch, compilation_mode, debug, compress } => {
            handle_build_command(entry, output);
        }

        CliCommands::Check { entry, output, watch, compilation_mode, debug } => {
            handle_check_command(entry, output);
        }

        CliCommands::Play {} => {
            panic!("Command 'play' is not implemented yet.");
        }
    }

    Ok(())
}
