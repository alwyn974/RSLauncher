//! Sync optional mod jars (.disabled) and shaderpack Iris configs on disk.
//!
//! Jar filenames are resolved from CurseForge / Modrinth (same APIs Lighty uses).
//! Shader presets only write/remove `.txt` next to packs that already exist -
//! we never create or duplicate shaderpacks (Euphoria Patches owns that).

use std::fs;
use std::path::Path;
use std::time::Duration;

use lighty_launcher::mods::curseforge;
use lighty_launcher::mods::modrinth;
use lighty_launcher::prelude::ModRequest;

use crate::catalog::{
    is_optional_mod_enabled, is_shader_variant_enabled, optional_mods, shader_variants,
};
use crate::dto::Settings;
use crate::error::AppError;
use crate::modpack_profile::{self, ModProvider, OptionalModSpec};

/// Enable/disable optional jars and apply/remove shader Iris configs.
pub async fn sync(game_dir: &Path, settings: &Settings) -> Result<(), AppError> {
    // Drop jars that left the pack before toggling optionals.
    if let Err(err) = crate::mods_cleanup::remove_orphans(game_dir, settings).await {
        log::warn!(
            target: "rslauncher",
            "[mods] orphan cleanup failed: {err}"
        );
    }
    sync_optional_mod_jars(game_dir, settings).await?;
    ensure_shader_configs(game_dir, settings)?;
    Ok(())
}

async fn sync_optional_mod_jars(game_dir: &Path, settings: &Settings) -> Result<(), AppError> {
    let mods_dir = game_dir.join("mods");
    if !mods_dir.is_dir() {
        return Ok(());
    }

    for entry in optional_mods() {
        let enabled = is_optional_mod_enabled(settings, &entry.id);
        let jar_name = match resolve_jar_filename(entry).await {
            Ok(name) => name,
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[optional] could not resolve jar name for {}: {err}",
                    entry.id
                );
                continue;
            }
        };

        let active = mods_dir.join(&jar_name);
        let disabled = mods_dir.join(format!("{jar_name}.disabled"));

        if enabled {
            if disabled.is_file() && !active.is_file() {
                log::info!(
                    target: "rslauncher",
                    "[optional] enabling {} → {}",
                    entry.id,
                    jar_name
                );
                fs::rename(&disabled, &active)?;
            } else if !active.is_file() && !disabled.is_file() {
                log::debug!(
                    target: "rslauncher",
                    "[optional] no jar yet for {} ({}) - install will fetch it",
                    entry.id,
                    jar_name
                );
            }
        } else if active.is_file() {
            log::info!(
                target: "rslauncher",
                "[optional] disabling {} → {}.disabled",
                entry.id,
                jar_name
            );
            fs::rename(&active, &disabled)?;
        }
    }

    Ok(())
}

pub(crate) async fn resolve_jar_filename(entry: &OptionalModSpec) -> Result<String, AppError> {
    let mc = &modpack_profile::get().minecraft;
    match entry.provider {
        ModProvider::Curseforge => {
            let project_id = entry.project_id.ok_or_else(|| {
                AppError::msg(format!(
                    "optional mod {} needs a CurseForge project_id",
                    entry.id
                ))
            })?;
            let file_id = entry.file_id.ok_or_else(|| {
                AppError::msg(format!(
                    "optional mod {} needs a pinned CurseForge file_id",
                    entry.id
                ))
            })?;
            let file = curseforge::fetch_pinned_file(project_id, file_id)
                .await
                .map_err(|e| AppError::msg(format!("CurseForge {project_id}/{file_id}: {e}")))?;
            Ok(file.file_name)
        }
        ModProvider::Modrinth => {
            let project = entry.project.as_deref().ok_or_else(|| {
                AppError::msg(format!("optional mod {} needs a Modrinth project", entry.id))
            })?;
            let request = ModRequest::Modrinth {
                id_or_slug: project.to_string(),
                version: entry.version.clone(),
            };
            let (resolved, _deps) = modrinth::fetch(
                &request,
                mc,
                &modpack_profile::get().loader,
                Duration::from_secs(6 * 60 * 60),
            )
            .await
            .map_err(|e| AppError::msg(format!("Modrinth {project}: {e}")))?;

            let path = resolved.path.ok_or_else(|| {
                AppError::msg(format!("Modrinth {project}: resolved mod has no path"))
            })?;
            Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    AppError::msg(format!("Modrinth {project}: invalid file path {path}"))
                })
        }
    }
}

fn ensure_shader_configs(game_dir: &Path, settings: &Settings) -> Result<(), AppError> {
    let shader_dir = game_dir.join("shaderpacks");
    let variants = shader_variants();
    if variants.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(&shader_dir)?;

    for variant in variants {
        let enabled = is_shader_variant_enabled(settings, &variant.id);
        let pack_stem = pack_stem(&variant.pack_name);
        let txt_path = shader_dir.join(format!("{pack_stem}.txt"));

        remove_bogus_euphoria_zip(&shader_dir, pack_stem);

        if enabled {
            fs::write(&txt_path, &variant.config_txt)?;
            if pack_exists_on_disk(&shader_dir, pack_stem) {
                log::info!(
                    target: "rslauncher",
                    "[shaders] wrote Iris config {} ({})",
                    txt_path.display(),
                    variant.id
                );
            } else {
                log::warn!(
                    target: "rslauncher",
                    "[shaders] wrote {} but pack {:?} is missing - apply Euphoria in-game",
                    txt_path.display(),
                    pack_stem
                );
            }
        } else if txt_path.is_file() {
            log::info!(
                target: "rslauncher",
                "[shaders] removed Iris config {}",
                txt_path.display()
            );
            let _ = fs::remove_file(&txt_path);
        }
    }

    Ok(())
}

fn pack_stem(pack_name: &str) -> &str {
    pack_name
        .strip_suffix(".zip")
        .or_else(|| pack_name.strip_suffix(".ZIP"))
        .unwrap_or(pack_name)
}

fn pack_exists_on_disk(shader_dir: &Path, stem: &str) -> bool {
    shader_dir.join(stem).is_dir() || shader_dir.join(format!("{stem}.zip")).is_file()
}

fn remove_bogus_euphoria_zip(shader_dir: &Path, stem: &str) {
    if !stem.contains("Euphoria") {
        return;
    }
    let bogus = shader_dir.join(format!("{stem}.zip"));
    if bogus.is_file() {
        log::warn!(
            target: "rslauncher",
            "[shaders] removing invalid Euphoria zip copy {}",
            bogus.display()
        );
        let _ = fs::remove_file(&bogus);
    }
}
