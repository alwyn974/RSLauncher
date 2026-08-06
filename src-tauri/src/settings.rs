use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use lighty_launcher::prelude::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::Settings;
use crate::error::AppError;
use crate::memory;
use crate::modpack_profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    active_pack_id: String,
    packs: HashMap<String, Settings>,
}

fn settings_path() -> PathBuf {
    AppState::config_dir().join("settings.json")
}

/// Read `activePackId` from disk without full settings validation (used at boot).
pub fn peek_active_pack_id() -> Option<String> {
    let raw = fs::read_to_string(settings_path()).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value
        .get("activePackId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn defaults_for_pack(pack_id: &str) -> Settings {
    let profile = modpack_profile::get_by_id(pack_id)
        .unwrap_or_else(|| modpack_profile::get());
    let mut settings = Settings::from_profile(profile);
    settings.ram_gb = memory::clamp_ram_gb_for(pack_id, memory::default_ram_gb_for(pack_id));
    settings
}

fn load_file() -> Result<SettingsFile, AppError> {
    let path = settings_path();
    if !path.exists() {
        let id = modpack_profile::active_id();
        let mut packs = HashMap::new();
        packs.insert(id.clone(), defaults_for_pack(&id));
        return Ok(SettingsFile {
            active_pack_id: id,
            packs,
        });
    }

    let raw = fs::read_to_string(&path)?;
    let value: Value = serde_json::from_str(&raw)?;

    if value.get("packs").is_some() {
        let mut file: SettingsFile = serde_json::from_value(value)?;
        normalize_file(&mut file);
        return Ok(file);
    }

    // Legacy flat settings.json → migrate under the default / active pack.
    let legacy: Settings = serde_json::from_value(value)?;
    let id = modpack_profile::default_id().to_string();
    let mut packs = HashMap::new();
    let mut migrated = legacy;
    migrated.ram_gb = memory::clamp_ram_gb_for(&id, migrated.ram_gb);
    packs.insert(id.clone(), migrated);
    let file = SettingsFile {
        active_pack_id: id,
        packs,
    };
    save_file(&file)?;
    log::info!(
        target: "rslauncher",
        "[settings] migrated flat settings.json to per-pack format"
    );
    Ok(file)
}

fn normalize_file(file: &mut SettingsFile) {
    if modpack_profile::get_by_id(&file.active_pack_id).is_none() {
        file.active_pack_id = modpack_profile::default_id().to_string();
    }
    for id in modpack_profile::ids() {
        file.packs
            .entry(id.clone())
            .or_insert_with(|| defaults_for_pack(id));
    }
    if let Some(settings) = file.packs.get_mut(&file.active_pack_id) {
        settings.ram_gb =
            memory::clamp_ram_gb_for(&file.active_pack_id, settings.ram_gb);
    }
}

fn save_file(file: &SettingsFile) -> Result<(), AppError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(file)?;
    fs::write(path, raw)?;
    Ok(())
}

/// Settings for the active modpack.
pub fn load() -> Result<Settings, AppError> {
    let file = load_file()?;
    let id = file.active_pack_id;
    Ok(file
        .packs
        .get(&id)
        .cloned()
        .unwrap_or_else(|| defaults_for_pack(&id)))
}

pub fn save(settings: &Settings) -> Result<(), AppError> {
    let mut file = load_file()?;
    let id = file.active_pack_id.clone();
    let mut settings = settings.clone();
    settings.ram_gb = memory::clamp_ram_gb_for(&id, settings.ram_gb);
    file.packs.insert(id, settings);
    save_file(&file)
}

pub fn set_active_pack_id(id: &str) -> Result<(), AppError> {
    if modpack_profile::get_by_id(id).is_none() {
        return Err(AppError::msg(format!("Unknown modpack: {id}")));
    }
    let mut file = load_file()?;
    file.active_pack_id = id.to_string();
    file.packs
        .entry(id.to_string())
        .or_insert_with(|| defaults_for_pack(id));
    save_file(&file)
}
