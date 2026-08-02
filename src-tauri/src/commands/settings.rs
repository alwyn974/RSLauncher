use lighty_launcher::prelude::VersionInfo;
use tauri::{AppHandle, Emitter};

use crate::catalog::{self, CatalogDto};
use crate::dto::{MemoryInfo, Settings};
use crate::error::AppError;
use crate::modpack;
use crate::modpack_meta;
use crate::optional_content;

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
