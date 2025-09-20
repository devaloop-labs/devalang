use clap::{Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Wav,
    Mid,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum AudioFormat {
    Wav16,
    Wav24,
    Wav32,
}

#[derive(Parser)]
#[command(name = "devalang")]
#[command(author = "Devaloop")]
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
pub enum TemplateCommand {
    /// Lists all available templates for Devalang projects.
    List,
    /// Displays information about a specific template.
    Info { name: String },
}

#[derive(Subcommand)]
pub enum AddonCommand {
    /// Installs an addon.
    Install {
        name: String,
        #[arg(long, default_value_t = false)]
        no_clear_tmp: bool,
    },

    /// Updates an addon.
    Update { name: String },

    /// Lists installed addons.
    List {},

    /// Removes an addon.
    Remove { name: String },
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

        #[arg(long, value_enum, value_delimiter = ',', default_value = "wav,mid")]
        /// Which output formats to generate. Comma-separated list is accepted.
        ///
        /// ### Default value
        /// - `wav,mid`
        ///
        output_format: Vec<OutputFormat>,

        #[arg(long, default_value = "wav16")]
        /// Audio format to use for file outputs (affects WAV export).
        ///
        /// ### Default value
        /// - `wav16`
        ///
        audio_format: AudioFormat,

        #[arg(long, default_value_t = 44100u32)]
        /// Sample rate to use for audio export and playback (e.g. 44100, 48000)
        sample_rate: u32,

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

        #[arg(long, default_value_t = 44100u32)]
        /// Sample rate to use for playback (e.g. 44100, 48000)
        sample_rate: u32,

        #[arg(long, default_value = "wav16")]
        /// Audio format to use for file outputs (affects WAV export).
        ///
        /// ### Default value
        /// - `wav16`
        ///
        audio_format: AudioFormat,

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

    /// Discover available local addons for Devalang.
    ///
    Discover {
        #[arg(long, default_value_t = false)]
        /// Do not clear the temporary extraction folder after installation.
        no_clear_tmp: bool,
    },

    /// Manage addons for Devalang projects.
    ///
    /// ### Subcommands
    /// - `install <publisher>.<name>` - Installs a new addon.
    /// - `list` - Lists installed addons.
    /// - `remove <publisher>.<name>` - Removes an existing addon.
    ///
    Addon {
        #[command(subcommand)]
        command: AddonCommand,
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

    /// Log in to your Devaloop account.
    ///
    Login {},

    /// Display information about the current Devalang user.
    ///
    Me {},

    /// Log out of your Devaloop account.
    ///
    Logout {},
}
