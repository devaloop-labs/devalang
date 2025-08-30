use crate::cli::template::commands::get_available_templates;
use devalang_utils::file::copy_dir_recursive;
use include_dir::{Dir, include_dir};
use std::{fs, path::Path};

#[cfg(feature = "cli")]
use inquire::{Confirm, Select};

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[cfg(feature = "cli")]
pub fn handle_init_command(name: Option<String>, template: Option<String>) {
    let current_dir = std::env::current_dir().unwrap();
    let project_name = name.clone().unwrap_or_else(|| {
        current_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    // Select a template if not provided
    let selected_template = template.unwrap_or_else(|| {
        Select::new(
            "Select a template for your project:",
            get_available_templates(),
        )
        .prompt()
        .unwrap_or_else(|_| {
            eprintln!("No template selected. Exiting...");
            std::process::exit(1);
        })
    });

    if selected_template.is_empty() {
        eprintln!("Template cannot be empty.");
        std::process::exit(1);
    }

    if name.is_none() {
        // Case of initialization in the current directory
        if fs::read_dir(&current_dir).unwrap().next().is_some() {
            let confirm =
                Confirm::new("The current directory is not empty. Do you want to continue?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false);

            if !confirm {
                eprintln!("Operation cancelled by the user.");
                std::process::exit(0);
            }
        }

        scaffold_project_current_dir(current_dir.as_path(), selected_template);
        println!(
            "✅ Initialized '{}' project in current directory: {}",
            project_name,
            current_dir.display()
        );
    } else {
        // Case of initialization in a new directory
        let target_path = current_dir.join(&project_name);

        if target_path.exists() {
            eprintln!("❌ A folder named '{}' already exists.", project_name);
            std::process::exit(1);
        }

        fs::create_dir_all(&target_path).expect("Error creating project directory");

        scaffold_project_current_dir(&target_path, selected_template);
        println!(
            "✅ Initialized '{}' project in: {}",
            project_name,
            target_path.display()
        );
    }
}

fn scaffold_project_current_dir(path: &Path, template: String) {
    let template_dir = TEMPLATES_DIR.get_dir(&template).unwrap_or_else(|| {
        eprintln!("❌ The template '{}' doesn't exist.", template);
        std::process::exit(1);
    });

    copy_dir_recursive(template_dir, path, template_dir.path());
}
