use std::fs;

use lighty_launcher::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::catalog::{self, CatalogDto};
use crate::dto::{MemoryInfo, ModpackListEntry, Settings};
use crate::error::AppError;
use crate::modpack;
use crate::modpack_meta;
use crate::modpack_profile;
use crate::optional_content;

fn open_dir(path: &std::path::Path) -> Result<(), AppError> {
    fs::create_dir_all(path)?;
    tauri_plugin_opener::open_path(path, None::<&str>)
        .map_err(|err| AppError::msg(err.to_string()))
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, AppError> {
    crate::settings::load()
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<Settings, AppError> {
    crate::settings::save(&settings)?;
    let saved = crate::settings::load()?;
    // Apply jar/shader toggles immediately when the instance exists.
    let instance = modpack::build_instance_with(&saved);
    if let Err(err) = optional_content::sync(instance.game_dirs(), &saved).await {
        log::warn!(target: "rslauncher", "[optional] sync after save_settings: {err}");
    }
    Ok(saved)
}

#[tauri::command]
pub async fn get_catalog() -> Result<CatalogDto, AppError> {
    let settings = crate::settings::load()?;
    Ok(catalog::catalog_dto(&settings))
}

#[tauri::command]
pub async fn list_modpacks() -> Result<Vec<ModpackListEntry>, AppError> {
    let mut entries = Vec::new();
    for id in modpack_profile::ids() {
        let Some(profile) = modpack_profile::get_by_id(id) else {
            continue;
        };
        let builder = modpack::build_instance_for(id);
        let root = builder.game_dirs().to_path_buf();
        entries.push(ModpackListEntry {
            id: profile.id.clone(),
            name: profile.display_name.clone(),
            version: profile.display_version.clone(),
            minecraft: profile.minecraft.clone(),
            loader: profile.loader_label().to_string(),
            loader_version: profile.loader_version.clone(),
            instance_name: profile.instance_name.clone(),
            installed: modpack::is_pack_installed_at(&root),
        });
    }
    Ok(entries)
}

#[tauri::command]
pub async fn set_active_modpack(id: String) -> Result<ModpackListEntry, AppError> {
    let profile = modpack_profile::set_active(&id)?;
    let builder = modpack::build_instance_for(&profile.id);
    let root = builder.game_dirs().to_path_buf();
    Ok(ModpackListEntry {
        id: profile.id.clone(),
        name: profile.display_name.clone(),
        version: profile.display_version.clone(),
        minecraft: profile.minecraft.clone(),
        loader: profile.loader_label().to_string(),
        loader_version: profile.loader_version.clone(),
        instance_name: profile.instance_name.clone(),
        installed: modpack::is_pack_installed_at(&root),
    })
}

#[tauri::command]
pub async fn get_active_modpack() -> Result<String, AppError> {
    Ok(modpack_profile::active_id())
}

#[tauri::command]
pub async fn get_memory_info(app: AppHandle) -> MemoryInfo {
    let info = crate::memory::info_cached();

    if !modpack_meta::has_cache() {
        tauri::async_runtime::spawn(async move {
            let updated = crate::memory::info_resolved().await;
            let _ = app.emit("memory://updated", updated);
        });
    }

    info
}

/// Open the active instance game directory in the system file manager.
#[tauri::command]
pub async fn open_instance_folder() -> Result<(), AppError> {
    open_dir(modpack::build_instance().game_dirs())
}

/// Open the RSLauncher data directory (parent of all instances).
#[tauri::command]
pub async fn open_launcher_folder() -> Result<(), AppError> {
    open_dir(crate::storage::root_dir())
}
