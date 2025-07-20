use clap::{ Parser, Subcommand };
use crate::utils::version::get_version;

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
pub enum InstallCommand {
    /// Installs a bank.
    Bank {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// Lists all available templates for Devalang projects.
    List,
    /// Displays information about a specific template.
    Info {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum BankCommand {
    /// Lists installed banks.
    List,
    /// Lists all available banks.
    Available,
    /// Displays information about a specific bank.
    Info {
        name: String,
    },
    /// Removes a bank.
    Remove {
        name: String,
    },
    /// Updates a specific or all banks.
    Update {
        name: Option<String>,
    },
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
    ///
    /// ### Example
    /// ```bash
    /// devalang build --entry ./src --output ./output --watch true
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

        #[arg(long, default_value = "real-time")]
        /// The mode of compilation.
        ///
        /// ### Default value
        /// - `real-time`
        ///
        /// ### Possible values
        /// - `real-time` - Compiles files as soon as possible.
        /// - `batch` - Compiles files one by one.
        /// - `check` - Analyzes the code without compiling it.
        ///
        compilation_mode: String,

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
    /// - `compilation_mode` - The mode of compilation. Defaults to "real-time".
    /// - `debug` - Whether to print debug information. Defaults to "false".
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

        #[arg(short, long, default_value = "real-time")]
        /// The mode of compilation.
        ///
        /// ### Default value
        /// - `real-time`
        ///
        /// ### Possible values
        /// - `real-time` - Analyzes files as soon as possible.
        /// - `batch` - Analyzes files one by one.
        /// - `check` - Analyzes the code without compiling it.
        ///
        compilation_mode: String,

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
}
