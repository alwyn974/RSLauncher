use tauri::{AppHandle, Emitter};

use crate::dto::{MemoryInfo, Settings};
use crate::error::AppError;
use crate::modpack_meta;

#[tauri::command]
pub async fn get_settings() -> Result<Settings, AppError> {
    crate::settings::load()
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<Settings, AppError> {
    crate::settings::save(&settings)?;
    crate::settings::load()
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
