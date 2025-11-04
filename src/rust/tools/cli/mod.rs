// Parent `tools` module controls `cli` gating; avoid duplicating crate-level cfg here.
mod commands;
pub mod config;
pub mod io;
pub mod rules_reporter;
pub mod state;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::devices::DevicesListCommand;
use commands::play::PlayCommand;
use state::CliContext;

#[derive(Parser, Debug)]
#[command(name = "devalang")]
#[command(
    version,
    about = "🦊 Devalang – A programming language for music and sound."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build and play deva file(s)
    Play(PlayCommand),
    /// Initialize a new project
    Init(commands::init::InitCommand),
    /// Builds deva file(s)
    Build(commands::build::BuildCommand),
    /// Check syntax without building
    Check(commands::check::CheckCommand),
    /// Manages addons (install, update, remove, list, discover)
    Addon(commands::addon::AddonCommand),
    /// Login to Devalang (authenticate with token)
    Login {
        /// Authentication token (optional, will prompt if not provided)
        token: Option<String>,
    },
    /// Logout from Devalang
    Logout,
    /// Check authentication status
    Me,
    /// Manage telemetry settings
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
    },
    /// Manage MIDI devices
    Devices {
        #[command(subcommand)]
        action: DevicesCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum DevicesCommands {
    /// List MIDI devices
    List(DevicesListCommand),
    /// Preview incoming/outgoing notes (non-writing)
    Preview(commands::devices::DevicesLiveCommand),
    /// Record incoming notes and write to a file
    Write(commands::devices::DevicesWriteCommand),
}

#[derive(Subcommand, Debug)]
pub enum TelemetryAction {
    /// Enable telemetry
    Enable,
    /// Disable telemetry
    Disable,
    /// Show telemetry status
    Status,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let ctx = CliContext::new();
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async move {
        match cli.command {
            Commands::Play(command) => commands::play::execute(command, &ctx).await?,
            Commands::Init(command) => command.execute(&ctx).await?,
            Commands::Build(command) => command.execute(&ctx).await?,
            Commands::Check(command) => command.execute(&ctx).await?,
            Commands::Addon(command) => command.execute(&ctx).await?,
            Commands::Login { token } => commands::auth::login(token).await?,
            Commands::Logout => commands::auth::logout().await?,
            Commands::Me => commands::auth::check_auth_status().await?,
            Commands::Telemetry { action } => {
                let logger = ctx.logger();
                match action {
                    TelemetryAction::Enable => {
                        config::telemetry::enable_telemetry()?;
                        logger.success("Telemetry enabled");
                        logger.info("Thank you for helping us improve Devalang!");
                    }
                    TelemetryAction::Disable => {
                        config::telemetry::disable_telemetry()?;
                        logger.success("Telemetry disabled");
                    }
                    TelemetryAction::Status => {
                        let status = config::telemetry::get_telemetry_status();
                        logger.info(format!("Telemetry is currently: {}", status));
                    }
                }
            }
            Commands::Devices { action } => match action {
                DevicesCommands::List(cmd) => {
                    commands::devices::execute_list(cmd, &ctx)?;
                }
                DevicesCommands::Preview(cmd) => {
                    commands::devices::execute_preview(cmd, &ctx).await?;
                }
                DevicesCommands::Write(cmd) => {
                    commands::devices::execute_write(cmd, &ctx).await?;
                }
            },
        }
        Ok(())
    })
}
