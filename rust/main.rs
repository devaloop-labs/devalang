pub mod core;
pub mod pulse;
pub mod cli;
pub mod runner;
pub mod audio;
pub mod utils;

use std::{ fs, io };
use clap::Parser;

use crate::{
    cli::{ check::handle_check_command, new::handle_new_command },
    core::{
        debugger::Debugger,
        preprocessor::{ module::load_all_modules, resolver::resolve_statement },
        types::{
            cli::{ Cli, CliCommands, CliTemplateCommand },
            module::Module,
            statement::{ Statement, StatementKind, StatementResolved },
            store::{ ExportTable, GlobalStore, ImportTable },
            variable::VariableValue,
        },
    },
    runner::executer::execute_statements,
};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::New { name, template } => {
            handle_new_command(&name, &template);
        }

        CliCommands::Template { command } =>
            match command {
                CliTemplateCommand::List => {
                    // handle_template_list();
                }
                CliTemplateCommand::Info { name } => {
                    // handle_template_info(&name);
                }
            }

        CliCommands::Build { entry, output, watch, compilation_mode, debug, compress } => {
            // handle_build(entry, output, watch, compilation_mode, debug, compress);
        }

        CliCommands::Check { entry, output, watch, compilation_mode, debug } => {
            handle_check_command(entry, output);
        }

        // TODO Play command
    }

    Ok(())
}
