use std::fs;
use std::path::PathBuf;

use lighty_launcher::prelude::AppState;

use crate::dto::Settings;
use crate::error::AppError;
use crate::memory;

fn settings_path() -> PathBuf {
    AppState::config_dir().join("settings.json")
}

pub fn load() -> Result<Settings, AppError> {
    let path = settings_path();
    let mut settings = if path.exists() {
        let raw = fs::read_to_string(path)?;
        serde_json::from_str(&raw)?
    } else {
        let mut defaults = Settings::default();
        defaults.ram_gb = memory::default_ram_gb();
        defaults
    };
    settings.ram_gb = memory::clamp_ram_gb(settings.ram_gb);
    Ok(settings)
}

pub fn save(settings: &Settings) -> Result<(), AppError> {
    let mut settings = settings.clone();
    settings.ram_gb = memory::clamp_ram_gb(settings.ram_gb);

    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(&settings)?;
    fs::write(path, raw)?;
    Ok(())
}
