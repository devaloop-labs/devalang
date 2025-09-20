#![cfg(feature = "cli")]

pub mod cli;
pub mod config;
pub mod core;
pub mod web;
pub use devalang_utils as utils;

use crate::cli::addon::commands::handle_install_addon_command;
use crate::cli::addon::commands::handle_list_addon_command;
use crate::cli::addon::commands::handle_remove_addon_command;
use crate::cli::addon::commands::handle_update_addon_command;
use crate::cli::me::commands::handle_me_command;
use crate::cli::parser::AddonCommand;
use crate::cli::telemetry::send::send_telemetry_event;
use crate::config::settings::ensure_user_config_file_exists;
use crate::config::settings::write_user_config_file;
use crate::{
    cli::{
        build::commands::handle_build_command,
        check::handle_check_command,
        discover::commands::handle_discover_command,
        init::commands::handle_init_command,
        login::commands::handle_login_command,
        parser::{Cli, Commands, TelemetryCommand, TemplateCommand},
        play::commands::handle_play_command,
        telemetry::{
            commands::{handle_telemetry_disable_command, handle_telemetry_enable_command},
            event_creator::{TelemetryEventCreator, TelemetryEventExt},
        },
        template::commands::{handle_template_info_command, handle_template_list_command},
    },
    config::driver::ProjectConfig,
    utils::first_usage::check_is_first_usage,
};
use clap::CommandFactory;
use clap::FromArgMatches;
use devalang_types::TelemetryErrorLevel;
use devalang_utils::path::ensure_deva_dir;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let version = devalang_utils::version::get_version();
    let signature = devalang_utils::signature::get_signature(&version);

    let version_static: &'static str = Box::leak(format!("v{}", version).into_boxed_str());
    let signature_static: &'static str = Box::leak(signature.into_boxed_str());

    let mut cmd = Cli::command();
    cmd = cmd.version(version_static).before_help(signature_static);

    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{}", signature_static);
        return Ok(());
    }

    let matches = cmd.get_matches();
    let cli: Cli = Cli::from_arg_matches(&matches).expect("failed to parse cli args");
    let mut config: Option<ProjectConfig> = None;

    let telemetry_event_creator = TelemetryEventCreator::new();
    let mut event = telemetry_event_creator.get_base_event();

    let mut had_error: bool = false;
    let mut last_error_message: Option<String> = None;
    let mut exit_code: Option<i32> = None;

    let _ = ensure_deva_dir();

    if check_is_first_usage() == true {
        write_user_config_file();
    } else {
        ensure_user_config_file_exists();
    }

    let duration = std::time::Instant::now();

    if !cli.no_config {
        config = config::ops::load_config(None);
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
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Check failed: {}", err),
                );
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
            output_format,
            audio_format,
            sample_rate,
        } => {
            if let Err(err) = handle_build_command(
                config,
                entry,
                output,
                output_format,
                audio_format,
                sample_rate,
                watch,
                debug,
                compress,
            ) {
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Build failed: {}", err),
                );
                had_error = true;
                last_error_message = Some(format!("build failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Play {
            entry,
            output,
            sample_rate,
            watch,
            repeat,
            debug,
            audio_format,
        } => {
            if let Err(err) = handle_play_command(
                config,
                entry,
                output,
                audio_format,
                sample_rate,
                watch,
                repeat,
                debug,
            ) {
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Play failed: {}", err),
                );
                had_error = true;
                last_error_message = Some(format!("play failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Addon { command } => {
            match command {
                AddonCommand::Install { name, no_clear_tmp } => {
                    if let Err(err) = handle_install_addon_command(name, no_clear_tmp).await {
                        let logger = devalang_utils::logger::Logger::new();
                        logger.log_message(
                            devalang_utils::logger::LogLevel::Error,
                            &format!("[error] Failed to install addon: {}", err),
                        );
                        had_error = true;
                        last_error_message = Some(format!("install addon failed: {}", err));
                        exit_code = Some(1);
                    }
                }

                AddonCommand::Update { name } => {
                    if let Err(err) = handle_update_addon_command(name).await {
                        let logger = devalang_utils::logger::Logger::new();
                        logger.log_message(
                            devalang_utils::logger::LogLevel::Error,
                            &format!("[error] Failed to update addon: {}", err),
                        );
                        had_error = true;
                        last_error_message = Some(format!("update addon failed: {}", err));
                        exit_code = Some(1);
                    }
                }

                AddonCommand::List {} => {
                    if let Err(err) = handle_list_addon_command().await {
                        let logger = devalang_utils::logger::Logger::new();
                        logger.log_message(
                            devalang_utils::logger::LogLevel::Error,
                            &format!("[error] Failed to list local addons: {}", err),
                        );
                        had_error = true;
                        last_error_message = Some(format!("addon list failed: {}", err));
                        exit_code = Some(1);
                    }
                }

                AddonCommand::Remove { name } => {
                    if let Err(err) = handle_remove_addon_command(name).await {
                        let logger = devalang_utils::logger::Logger::new();
                        logger.log_message(
                            devalang_utils::logger::LogLevel::Error,
                            &format!("[error] Failed to remove addon: {}", err),
                        );
                        had_error = true;
                        last_error_message = Some(format!("remove addon failed: {}", err));
                        exit_code = Some(1);
                    }
                } // AddonCommand::Bank { command } => {
                  //     match command {
                  //         BankCommand::List => {
                  //             if let Err(err) = handle_bank_list_command().await {
                  //                 let logger = devalang_utils::logger::Logger::new();
                  //                 logger.log_message(
                  //                     devalang_utils::logger::LogLevel::Error,
                  //                     &format!("[error] Failed to list local banks: {}", err)
                  //                 );
                  //                 had_error = true;
                  //                 last_error_message = Some(format!("bank list failed: {}", err));
                  //                 exit_code = Some(1);
                  //             }
                  //         }

                  //         BankCommand::Available => {
                  //             if let Err(err) = handle_bank_available_command().await {
                  //                 let logger = devalang_utils::logger::Logger::new();
                  //                 logger.log_message(
                  //                     devalang_utils::logger::LogLevel::Error,
                  //                     &format!("[error] Failed to list available banks: {}", err)
                  //                 );
                  //                 had_error = true;
                  //                 last_error_message = Some(
                  //                     format!("bank available failed: {}", err)
                  //                 );
                  //                 exit_code = Some(1);
                  //             }
                  //         }

                  //         // BankCommand::Info { name } => {
                  //         //     if let Err(err) = handle_bank_info_command(name).await {
                  //         //         let logger = devalang_utils::logger::Logger::new();
                  //         //         logger.log_message(
                  //         //             devalang_utils::logger::LogLevel::Error,
                  //         //             &format!("[error] Failed to get bank info: {}", err)
                  //         //         );
                  //         //         had_error = true;
                  //         //         last_error_message = Some(format!("bank info failed: {}", err));
                  //         //         exit_code = Some(1);
                  //         //     }
                  //         // }

                  //         // BankCommand::Remove { name } => {
                  //         //     if let Err(err) = handle_remove_bank_command(name).await {
                  //         //         let logger = devalang_utils::logger::Logger::new();
                  //         //         logger.log_message(
                  //         //             devalang_utils::logger::LogLevel::Error,
                  //         //             &format!("[error] Failed to remove bank: {}", err)
                  //         //         );
                  //         //         had_error = true;
                  //         //         last_error_message = Some(format!("bank remove failed: {}", err));
                  //         //         exit_code = Some(1);
                  //         //     }
                  //         // }

                  //         // BankCommand::Update { name } => {
                  //         //     if let Err(err) = handle_update_bank_command(name).await {
                  //         //         let logger = devalang_utils::logger::Logger::new();
                  //         //         logger.log_message(
                  //         //             devalang_utils::logger::LogLevel::Error,
                  //         //             &format!("[error] Failed to update bank: {}", err)
                  //         //         );
                  //         //         had_error = true;
                  //         //         last_error_message = Some(format!("bank update failed: {}", err));
                  //         //         exit_code = Some(1);
                  //         //     }
                  //         // }
                  //     }
                  // }

                  // AddonCommand::Plugin { command} => {
                  //     match command {
                  //         PluginCommand::List => {
                  //             if let Err(err) = handle_plugin_list_command().await {
                  //                 let logger = devalang_utils::logger::Logger::new();
                  //                 logger.log_message(
                  //                     devalang_utils::logger::LogLevel::Error,
                  //                     &format!("[error] Failed to list local plugins: {}", err)
                  //                 );
                  //                 had_error = true;
                  //                 last_error_message = Some(format!("plugin list failed: {}", err));
                  //                 exit_code = Some(1);
                  //             }
                  //         }

                  //         PluginCommand::Available => {
                  //             if let Err(err) = handle_plugin_available_command().await {
                  //                 let logger = devalang_utils::logger::Logger::new();
                  //                 logger.log_message(
                  //                     devalang_utils::logger::LogLevel::Error,
                  //                     &format!("[error] Failed to list available plugins: {}", err)
                  //                 );
                  //                 had_error = true;
                  //                 last_error_message = Some(
                  //                     format!("plugin available failed: {}", err)
                  //                 );
                  //                 exit_code = Some(1);
                  //             }
                  //         }
                  //     }
                  // }
            }
        }

        // Commands::Install { command } => match command {
        //     InstallCommand::Template { name } => {
        //         if let Err(err) = handle_install_command(name, AddonType::Template).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to install template: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("install template failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }
        //     InstallCommand::Bank { name } => {
        //         if let Err(err) = handle_install_command(name, AddonType::Bank).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to install bank: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("install bank failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }
        //     InstallCommand::Plugin { name } => {
        //         if let Err(err) = handle_install_command(name, AddonType::Plugin).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to install plugin: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("install plugin failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }
        //     InstallCommand::Preset { name } => {
        //         if let Err(err) = handle_install_command(name, AddonType::Preset).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to install preset: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("install preset failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }
        // },

        // Commands::Bank { command } => match command {
        //     BankCommand::List => {
        //         if let Err(err) = handle_bank_list_command().await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to list local banks: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("bank list failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }

        //     BankCommand::Available => {
        //         if let Err(err) = handle_bank_available_command().await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to list available banks: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("bank available failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }

        //     BankCommand::Info { name } => {
        //         if let Err(err) = handle_bank_info_command(name).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to get bank info: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("bank info failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }

        //     BankCommand::Remove { name } => {
        //         if let Err(err) = handle_remove_bank_command(name).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to remove bank: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("bank remove failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }

        //     BankCommand::Update { name } => {
        //         if let Err(err) = handle_update_bank_command(name).await {
        //             let logger = devalang_utils::logger::Logger::new();
        //             logger.log_message(
        //                 devalang_utils::logger::LogLevel::Error,
        //                 &format!("[error] Failed to update bank: {}", err),
        //             );
        //             had_error = true;
        //             last_error_message = Some(format!("bank update failed: {}", err));
        //             exit_code = Some(1);
        //         }
        //     }
        // },

        // Commands::Update { only } => {
        //     if let Err(err) = handle_update_command(only).await {
        //         let logger = devalang_utils::logger::Logger::new();
        //         logger.log_message(
        //             devalang_utils::logger::LogLevel::Error,
        //             &format!("[error] Update failed: {}", err)
        //         );
        //         had_error = true;
        //         last_error_message = Some(format!("update failed: {}", err));
        //         exit_code = Some(1);
        //     }
        // }
        Commands::Telemetry { command } => match command {
            TelemetryCommand::Enable { .. } => {
                if let Err(err) = handle_telemetry_enable_command().await {
                    let logger = devalang_utils::logger::Logger::new();
                    logger.log_message(
                        devalang_utils::logger::LogLevel::Error,
                        &format!("[error] Failed to enable telemetry: {}", err),
                    );
                    had_error = true;
                    last_error_message = Some(format!("telemetry enable failed: {}", err));
                    exit_code = Some(1);
                }
            }
            TelemetryCommand::Disable { .. } => {
                if let Err(err) = handle_telemetry_disable_command().await {
                    let logger = devalang_utils::logger::Logger::new();
                    logger.log_message(
                        devalang_utils::logger::LogLevel::Error,
                        &format!("[error] Failed to disable telemetry: {}", err),
                    );
                    had_error = true;
                    last_error_message = Some(format!("telemetry disable failed: {}", err));
                    exit_code = Some(1);
                }
            }
        },

        Commands::Discover { no_clear_tmp } => {
            if let Err(err) = handle_discover_command(no_clear_tmp).await {
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Failed to discover: {}", err),
                );
                had_error = true;
                last_error_message = Some(format!("discover failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Login { .. } => {
            if let Err(err) = handle_login_command().await {
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Login failed: {}", err),
                );
                had_error = true;
                last_error_message = Some(format!("login failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Me {} => {
            if let Err(err) = handle_me_command().await {
                let logger = devalang_utils::logger::Logger::new();
                logger.log_message(
                    devalang_utils::logger::LogLevel::Error,
                    &format!("[error] Me command failed: {}", err),
                );
                had_error = true;
                last_error_message = Some(format!("me command failed: {}", err));
                exit_code = Some(1);
            }
        }

        Commands::Logout { .. } => {
            let logger = devalang_utils::logger::Logger::new();
            logger.log_message(
                devalang_utils::logger::LogLevel::Error,
                "[error] Logout command is not implemented yet.",
            );
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
        event.set_error(TelemetryErrorLevel::Critical, last_error_message, exit_code);
    }

    let _ = send_telemetry_event(&event).await;

    Ok(())
}
