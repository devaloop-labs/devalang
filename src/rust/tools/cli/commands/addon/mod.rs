#![cfg(feature = "cli")]

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::tools::cli::state::CliContext;

mod discover;
mod download;
mod install;
mod list;
mod metadata;
mod remove;
mod update;
mod utils;

#[derive(Debug, Clone, Args)]
pub struct AddonCommand {
    #[command(subcommand)]
    pub action: Option<AddonAction>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AddonAction {
    Install {
        name: String,
    },
    Remove {
        name: String,
    },
    List,
    Discover {
        /// Search term (optional)
        search: Option<String>,
        /// Filter by addon type (bank, plugin, preset, template)
        #[arg(short = 't', long)]
        addon_type: Option<String>,
        /// Filter by author
        #[arg(short, long)]
        author: Option<String>,
    },
    Update {
        name: String,
    },
    Metadata {
        name: String,
    },
}

impl AddonCommand {
    pub async fn execute(&self, ctx: &CliContext) -> Result<()> {
        let logger = ctx.logger();

        match &self.action {
            Some(AddonAction::Install { name }) => {
                logger.action(format!("Installing addon '{}'...", name));
                match install::install_addon(name.clone(), false).await {
                    Ok(_) => {
                        logger.success(format!("Addon '{}' installed successfully", name));
                    }
                    Err(e) => {
                        logger.error(format!("Failed to install addon '{}': {}", name, e));
                        return Err(e);
                    }
                }
            }
            Some(AddonAction::Remove { name }) => {
                logger.action(format!("Removing addon '{}'...", name));
                match remove::remove_addon(name.clone()).await {
                    Ok(_) => {
                        logger.success(format!("Addon '{}' removed successfully", name));
                    }
                    Err(e) => {
                        logger.error(format!("Failed to remove addon '{}': {}", name, e));
                        return Err(e);
                    }
                }
            }
            Some(AddonAction::List) | None => {
                logger.info("Listing installed addons...");
                match list::list_addons().await {
                    Ok(_) => {}
                    Err(e) => {
                        logger.error(format!("Failed to list addons: {}", e));
                        return Err(e);
                    }
                }
            }
            Some(AddonAction::Discover {
                search,
                addon_type,
                author,
            }) => {
                logger.action("Discovering addons...");
                match discover::discover_addons(search.clone(), addon_type.clone(), author.clone())
                    .await
                {
                    Ok(addons) => {
                        discover::display_addon_results(&addons);
                    }
                    Err(e) => {
                        logger.error(format!("Failed to discover addons: {}", e));
                        logger.info("Visit https://devalang.com/forge to browse addons manually.");
                        return Err(e);
                    }
                }
            }
            Some(AddonAction::Update { name }) => {
                logger.action(format!("Updating addon '{}'...", name));
                match update::update_addon(name.clone()).await {
                    Ok(_) => {
                        logger.success(format!("Addon '{}' updated successfully", name));
                    }
                    Err(e) => {
                        logger.error(format!("Failed to update addon '{}': {}", name, e));
                        return Err(e);
                    }
                }
            }
            Some(AddonAction::Metadata { name }) => {
                logger.action(format!("Fetching metadata for addon '{}'...", name));
                match metadata::get_addon_from_api(name).await {
                    Ok(metadata) => {
                        logger.info(format!("Name: {}", metadata.name));
                        logger.info(format!("Publisher: {}", metadata.publisher));
                        logger.info(format!("Type: {}", metadata.addon_type));
                    }
                    Err(e) => {
                        logger.error(format!("Failed to fetch metadata for '{}': {}", name, e));
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}
