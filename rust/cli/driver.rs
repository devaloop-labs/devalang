use crate::utils::version::get_version;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "devalang")]
#[command(author = "Devaloop")]
#[command(version = get_version())]
#[command(about = "🦊 Devalang – A programming language for music and sound.")]
pub struct Cli {
    #[arg(long, global = true)]
    /// Skips loading the configuration file.
    pub no_config: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum TelemetryCommand {
    /// Enables telemetry data collection.
    Enable {},
    /// Disables telemetry data collection.
    Disable {},
}

#[derive(Subcommand)]
pub enum InstallCommand {
    /// Installs a bank.
    Bank { name: String },
    /// Installs a plugin.
    Plugin { name: String },
    /// Installs a preset.
    Preset { name: String },
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// Lists all available templates for Devalang projects.
    List,
    /// Displays information about a specific template.
    Info { name: String },
}

#[derive(Subcommand)]
pub enum BankCommand {
    /// Lists installed banks.
    List,
    /// Lists all available banks.
    Available,
    /// Displays information about a specific bank.
    Info { name: String },
    /// Removes a bank.
    Remove { name: String },
    /// Updates a specific or all banks.
    Update { name: Option<String> },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Devalang project.
    ///
    /// ### Arguments
    /// - `name` - The name of the project to create.
    /// - `template` - The template to use for the project. Defaults to "default".
    ///
    /// ### Example
    /// ```bash
    /// devalang init --name my_project --template default
    ///
    Init {
        #[arg(short, long)]
        /// The optional name (directory) of the project to create.
        name: Option<String>,

        #[arg(short, long)]
        /// The template to use for the project.
        ///
        /// ### Default value
        /// - `default`
        ///
        template: Option<String>,
    },

    /// Manage templates for Devalang projects.
    ///
    /// ### Subcommands
    /// - `list` - Lists all available templates.
    /// - `info <name>` - Displays information about a specific template.
    ///
    /// ### Example
    /// ```bash
    /// devalang template list
    /// devalang template info my_template
    /// ```bash
    ///
    Template {
        #[command(subcommand)]
        /// The template command to execute.
        command: TemplateCommand,
    },

    /// Build the program and generate output files.
    ///
    /// ### Arguments
    /// - `entry` - The entry point of the program to build. Defaults to "./src".
    /// - `output` - The directory where the output files will be generated. Defaults to "./output".
    /// - `watch` - Whether to watch for changes and rebuild. Defaults to "true".
    /// - `debug` - Whether to print debug information. Defaults to "false".
    /// - `compress` - Whether to compress the output files. Defaults to "false".
    ///
    /// ### Example
    /// ```bash
    /// devalang build --entry ./src --output ./output --watch true --debug false --compress false
    /// ```
    ///
    Build {
        #[arg(short, long)]
        /// The entry point of the program to build.
        ///
        entry: Option<String>,

        #[arg(short, long)]
        /// The directory where the output files will be generated.
        ///
        output: Option<String>,

        #[arg(long, default_value_t = false)]
        /// Whether to watch for changes and rebuild.
        ///
        /// ### Default value
        /// - `false`
        ///
        watch: bool,

        #[arg(short, long, default_value_t = false)]
        /// Whether to print debug information.
        ///
        /// ### Default value
        /// - `false`
        ///
        debug: bool,

        #[arg(short, long, default_value_t = false)]
        /// Whether to compress the output files.
        ///
        /// ### Default value
        /// - `false`
        ///
        compress: bool,
    },

    /// Analyze the program for errors and warnings.
    ///
    /// ### Arguments
    /// - `entry` - The entry point of the program to analyze. Defaults to "./src".
    /// - `output` - The directory where the output files will be generated. Defaults to "./output".
    /// - `watch` - Whether to watch for changes and re-analyze. Defaults to "true".
    /// - `debug` - Whether to print debug information. Defaults to "false".
    ///
    /// ### Example
    /// ```bash
    /// devalang check --entry ./src --output ./output --watch true --debug false
    /// ```
    ///
    Check {
        #[arg(short, long)]
        /// The entry point of the program to analyze.
        entry: Option<String>,

        #[arg(short, long)]
        /// The directory where the output files will be generated.
        output: Option<String>,

        #[arg(long, default_value_t = false)]
        /// Whether to watch for changes and re-analyze.
        ///
        /// ### Default value
        /// - `false`
        ///
        watch: bool,

        #[arg(short, long, default_value_t = false)]
        /// Whether to print debug information.
        ///
        /// ### Default value
        /// - `false`
        ///
        debug: bool,
    },

    /// Play the program and generate output files.
    ///
    /// ### Arguments
    /// - `entry` - The entry point of the program to play. Defaults to "./src".
    /// - `output` - The directory where the output files will be generated. Defaults to "./output".
    /// - `watch` - Whether to watch for changes and re-play. Defaults to "false".
    /// - `repeat` - Whether to replay the program after it finishes. Defaults to "false".
    /// - `debug` - Whether to print debug information. Defaults to "false".
    ///
    /// Note: `--repeat` and `--watch` cannot be used together. Instead use `repeat` to watch for changes and replay the program.
    ///
    /// ### Example
    /// ```bash
    /// devalang play --entry ./src --output ./output --repeat true --debug false
    /// ```
    ///
    Play {
        #[arg(short, long)]
        /// The entry point of the program to play.
        entry: Option<String>,

        #[arg(short, long)]
        /// The directory where the output files will be generated.
        output: Option<String>,

        #[arg(long, default_value_t = false)]
        /// Whether to watch for changes and re-play.
        ///
        /// ### Default value
        /// - `false`
        ///
        watch: bool,

        #[arg(long, default_value_t = false)]
        /// Whether to replay the program after it finishes.
        ///
        /// ### Default value
        /// - `false`
        ///
        repeat: bool,

        #[arg(short, long, default_value_t = false)]
        /// Whether to print debug information.
        ///
        /// ### Default value
        /// - `false`
        ///
        debug: bool,
    },

    /// Update the Devalang CLI to the latest version.
    ///
    /// ### Arguments
    /// - `only` - Selects what to update (separated by commas). Defaults to updating all components.
    ///
    Update {
        // #[arg(long, default_value_t = false)]
        /// Whether to allow updates when the working directory is dirty.
        // allow_dirty: bool,

        #[arg(long, default_value = "")]
        /// Selects what to update (separated by commas).
        only: Option<String>,
    },

    /// Install templates, banks, or plugins.
    ///
    /// ### Subcommands
    /// - `template` - Installs a template.
    /// - `bank` - Installs a bank.
    /// - `plugin` - Installs a plugin.
    ///
    Install {
        #[command(subcommand)]
        command: InstallCommand,
    },

    /// Manage banks for Devalang projects.
    ///
    /// ### Subcommands
    /// - `list` - Lists all available banks.
    /// - `info <name>` - Displays information about a specific bank.
    ///
    Bank {
        #[command(subcommand)]
        command: BankCommand,
    },

    /// Telemetry settings for Devalang.
    ///
    /// ### Subcommands
    /// - `enable` - Enables telemetry data collection.
    /// - `disable` - Disables telemetry data collection.
    ///
    Telemetry {
        #[command(subcommand)]
        command: TelemetryCommand,
    },

    /// Generate addon scaffolding for Devalang.
    ///
    /// ### Subcommands
    /// - `bank` - Generates a bank scaffold.
    /// - `plugin` - Generates a plugin scaffold.
    /// - `preset` - Generates a preset scaffold.
    ///
    // Scaffold {
    //     #[command(subcommand)]
    //     command: ScaffoldCommand,
    // },

    /// Log in to your Devaloop account.
    ///
    Login {},

    /// Log out of your Devaloop account.
    ///
    Logout {},
}
