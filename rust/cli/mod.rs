pub mod check;
pub mod build;
pub mod init;
pub mod template;
pub mod play;

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
pub enum TemplateCommand {
    /// Lists all available templates for Devalang projects.
    List,
    /// Displays information about a specific template.
    Info {
        name: String,
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
    /// - `watch` - Whether to watch for changes and re-analyze. Defaults to "true".
    ///
    /// ### Example
    /// ```bash
    /// devalang check --entry ./src --watch true --compilation-mode real-time
    /// ```
    Check {
        #[arg(short, long)]
        /// The entry point of the program to analyze.
        ///
        entry: Option<String>,

        #[arg(short, long)]
        /// The directory where the output files will be generated.
        ///
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

    Play {
        #[arg(short, long)]
        /// The entry point of the program to play.
        ///
        entry: Option<String>,

        #[arg(short, long)]
        /// The directory where the output files will be generated.
        ///
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
}
