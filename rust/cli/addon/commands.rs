use crate::cli::addon::{
    install::install_addon, list::list_addons, remove::remove_addon, update::update_addon,
};

pub async fn handle_install_addon_command(name: String, no_clear_tmp: bool) -> Result<(), String> {
    if let Err(e) = install_addon(name, no_clear_tmp).await {
        return Err(format!("Failed to install addon: {}", e));
    }

    Ok(())
}

pub async fn handle_list_addon_command() -> Result<(), String> {
    if let Err(e) = list_addons().await {
        return Err(format!("Failed to list addons: {}", e));
    }

    Ok(())
}

pub async fn handle_remove_addon_command(name: String) -> Result<(), String> {
    if let Err(e) = remove_addon(name).await {
        return Err(format!("Failed to remove addon: {}", e));
    }

    Ok(())
}

pub async fn handle_update_addon_command(name: String) -> Result<(), String> {
    if let Err(e) = update_addon(name).await {
        return Err(format!("Failed to update addon: {}", e));
    }

    Ok(())
}
