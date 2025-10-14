#![cfg(feature = "cli")]

use anyhow::Result;
use clap::Args;
use std::fs;
use std::path::Path;

use crate::tools::cli::state::CliContext;

#[derive(Debug, Clone, Args)]
pub struct InitCommand {
    /// Project name (creates new directory) or use current directory if not specified
    #[arg(short, long)]
    pub name: Option<String>,

    /// Project template (default, basic, advanced)
    #[arg(short, long, default_value = "default")]
    pub template: String,
}

impl InitCommand {
    pub async fn execute(&self, ctx: &CliContext) -> Result<()> {
        let logger = ctx.logger();

        let current_dir = std::env::current_dir()?;

        let (target_path, project_name) = if let Some(name) = &self.name {
            let path = current_dir.join(name);
            (path, name.clone())
        } else {
            let name = current_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (current_dir.clone(), name)
        };

        // Check if target exists and is not empty
        if target_path.exists() && self.name.is_some() {
            logger.error(format!("Directory '{}' already exists", project_name));
            anyhow::bail!("Target directory already exists");
        }

        logger.action(format!("Initializing '{}' project...", project_name));

        // Create directory if needed
        if self.name.is_some() {
            fs::create_dir_all(&target_path)?;
        }

        // Scaffold project
        self.scaffold_project(&target_path, &self.template)?;

        logger.success(format!(
            "Project '{}' initialized successfully at '{}'",
            project_name,
            target_path.display()
        ));

        logger.info("Next steps:");
        if self.name.is_some() {
            logger.info(format!("  cd {}", project_name));
        }
        logger.info("  devalang build");
        logger.info("  devalang play");

        Ok(())
    }

    fn scaffold_project(&self, path: &Path, template: &str) -> Result<()> {
        // Create basic project structure
        let examples_dir = path.join("examples");
        let output_dir = path.join("output");
        let deva_dir = path.join(".deva");

        fs::create_dir_all(&examples_dir)?;
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(deva_dir.join("banks"))?;
        fs::create_dir_all(deva_dir.join("plugins"))?;
        fs::create_dir_all(deva_dir.join("presets"))?;
        fs::create_dir_all(deva_dir.join("templates"))?;

        // Get project name
        let project_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Create devalang.json config (recommended format)
        let config_content = self.get_config_template(template, &project_name);
        fs::write(path.join("devalang.json"), config_content)?;

        // Create .gitignore
        let gitignore_content = self.get_gitignore_template();
        fs::write(path.join(".gitignore"), gitignore_content)?;

        // Create example file based on template
        let example_content = self.get_example_template(template);
        fs::write(examples_dir.join("index.deva"), example_content)?;

        Ok(())
    }

    fn get_config_template(&self, _template: &str, project_name: &str) -> String {
        format!(
            r#"{{
  "project": {{
    "name": "{}"
  }},
  "paths": {{
    "entry": "examples/index.deva",
    "output": "output"
  }},
  "audio": {{
    "format": ["wav", "mid"],
    "bit_depth": 16,
    "channels": 2,
    "sample_rate": 44100,
    "resample_quality": "sinc24",
    "bpm": 120
  }},
  "live": {{
    "crossfade_ms": 500
  }}
}}
"#,
            project_name
        )
    }

    fn get_gitignore_template(&self) -> String {
        r#"# Devalang
output/
.deva/
*.wav
*.mid
*.mp3
*.flac

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo
"#
        .to_string()
    }

    fn get_example_template(&self, template: &str) -> String {
        match template {
            "basic" => self.get_basic_example(),
            "advanced" => self.get_advanced_example(),
            _ => self.get_default_example(),
        }
    }

    fn get_default_example(&self) -> String {
        r#"# Welcome to Devalang!
# This is a simple example to get you started.

# Set tempo
bpm 120

# Load a bank
bank devaloop.808 as drums

# Play some simple beats
drums.kick 1/4
sleep 250
drums.kick 1/4
sleep 250
drums.snare 1/4
sleep 250
drums.kick 1/4
"#
        .to_string()
    }

    fn get_basic_example(&self) -> String {
        r#"# Basic Devalang Example with patterns

bpm 120
bank devaloop.808 as drums

# Define a kick pattern
pattern kickPattern with drums.kick = "x--- x--- x--- x---"

# Define a snare pattern with options
pattern snarePattern with drums.snare {
    swing: 0.1,
    humanize: 0.02,
    velocity: 0.8
} = "---- x--- ---- x---"

# Define a hihat pattern
pattern hihatPattern with drums.hihat = "x-x- x-x- x-x- x-x-"

# Play the patterns
call kickPattern
call snarePattern
call hihatPattern
"#
        .to_string()
    }

    fn get_advanced_example(&self) -> String {
        r#"# Advanced Devalang Example with synths and automation

bpm 120
bank devaloop.808 as drums

# Variables
let kickVol = 1.0
let snareVol = 0.8

# Create a filtered synth
let bass = synth sine {
    filters: [
        {
            type: "lowpass",
            cutoff: 800.0
        }
    ]
}

# Define drum patterns
pattern kickPattern with drums.kick = "x--- x--- x--- x---"
pattern snarePattern with drums.snare {
    velocity: snareVol
} = "---- x--- ---- x---"
pattern hihatPattern with drums.hihat {
    velocity: 0.6
} = "x-x- x-x- x-x- x-x-"

# Group for drums
group drumLoop:
    call kickPattern
    call snarePattern
    call hihatPattern

# Group for bass melody
group bassLine:
    bass -> note(C2) -> velocity(80) -> duration(1000)
    sleep 250
    bass -> note(E2) -> velocity(80) -> duration(1000)
    sleep 250
    bass -> note(G2) -> velocity(80) -> duration(1000)
    sleep 250

# Play groups in parallel
spawn drumLoop
spawn bassLine
"#
        .to_string()
    }
}
