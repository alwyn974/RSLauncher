//! Reconcile instance files after Lighty install.
//!
//! Lighty gaps this covers:
//! 1. Version bumps leave old jars on disk (orphan cleanup).
//! 2. Pack `overrides/mods/*.jar` are extracted *before* manifest downloads and
//!    with SkipWarn — ATM ships a fixed CC-Tweaked in overrides while the
//!    manifest still lists an older file → duplicate modId / wrong version wins.
//! 3. SkipWarn also keeps stale override configs (e.g. `bcc-common.toml` still
//!    saying 7.2 after a 7.3 bump). On pack file_id change we re-apply the
//!    whole overrides tree with overwrite.
//!
//! Flow: re-apply overrides → drop conflicting modIds → orphan jar cleanup.

use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use lighty_launcher::core::calculate_sha1_bytes;
use lighty_launcher::mods::curseforge;
use lighty_launcher::mods::modrinth;
use lighty_launcher::mods::ModpackSource;
use lighty_launcher::prelude::{AppState, ModRequest};
use serde::Deserialize;
use zip::ZipArchive;

use crate::catalog::optional_mods;
use crate::dto::Settings;
use crate::error::AppError;
use crate::modpack_profile::{self, PackProvider};
use crate::optional_content;

#[derive(Debug, Deserialize)]
struct CachedMod {
    path: Option<String>,
}

/// Apply pack overrides, dedupe modIds, then remove orphan jars.
pub async fn remove_orphans(game_dir: &Path, _settings: &Settings) -> Result<(), AppError> {
    let mods_dir = game_dir.join("mods");
    fs::create_dir_all(&mods_dir)?;

    let pack_key = current_pack_key();
    let stamp_path = game_dir.join(".rslauncher-pack-id");
    let pack_changed = fs::read_to_string(&stamp_path)
        .map(|s| s.trim() != pack_key)
        .unwrap_or(true);

    let archive = match resolve_pack_archive_path().await {
        Ok(Some(path)) => Some(path),
        Ok(None) => {
            if modpack_profile::get().pack.is_some() {
                log::warn!(
                    target: "rslauncher",
                    "[mods] reconcile skipped — pack archive missing \
                     (launch once so Lighty can download it)"
                );
                return Ok(());
            }
            None
        }
        Err(err) => {
            log::warn!(target: "rslauncher", "[mods] resolve pack archive: {err}");
            return Ok(());
        }
    };

    let override_jars = if let Some(archive) = archive.as_ref() {
        let jars = tokio::task::spawn_blocking({
            let archive = archive.clone();
            let game_dir = game_dir.to_path_buf();
            move || apply_pack_overrides(&archive, &game_dir, pack_changed)
        })
        .await
        .map_err(|e| AppError::msg(format!("override apply task panicked: {e}")))??;

        if pack_changed {
            if let Err(err) = fs::write(&stamp_path, &pack_key) {
                log::warn!(
                    target: "rslauncher",
                    "[mods] could not write pack stamp {}: {err}",
                    stamp_path.display()
                );
            } else {
                log::info!(
                    target: "rslauncher",
                    "[mods] pack overrides refreshed for {pack_key}"
                );
            }
        } else if !jars.is_empty() {
            log::info!(
                target: "rslauncher",
                "[mods] applied {} override jar(s)",
                jars.len()
            );
        }
        jars
    } else {
        // Manifest-only pack: no zip overrides to re-apply.
        if pack_changed {
            if let Err(err) = fs::write(&stamp_path, &pack_key) {
                log::warn!(
                    target: "rslauncher",
                    "[mods] could not write pack stamp {}: {err}",
                    stamp_path.display()
                );
            }
        }
        HashSet::new()
    };

    let override_mod_ids = tokio::task::spawn_blocking({
        let mods_dir = mods_dir.clone();
        let override_jars = override_jars.clone();
        move || mod_ids_for_jars(&mods_dir, &override_jars)
    })
    .await
    .map_err(|e| AppError::msg(format!("modId scan panicked: {e}")))??;

    // Manifest jars that share a modId with an override must go.
    let dropped_conflicts = drop_modid_conflicts(&mods_dir, &override_jars, &override_mod_ids)?;

    let mut keep = if let Some(archive) = archive.as_ref() {
        load_pack_mod_jars_from_cache(archive)?
    } else {
        HashSet::new()
    };
    if keep.is_empty() && override_jars.is_empty() && archive.is_some() {
        log::warn!(
            target: "rslauncher",
            "[mods] orphan cleanup skipped — modpack mod list cache missing/empty"
        );
        return Ok(());
    }

    for name in &override_jars {
        keep.insert(name.clone());
    }
    for name in &dropped_conflicts {
        keep.remove(name);
    }
    extend_with_extra_jars(&mut keep).await;

    if keep.is_empty() {
        log::warn!(
            target: "rslauncher",
            "[mods] orphan cleanup skipped — no managed jar names resolved"
        );
        return Ok(());
    }

    let mut removed = 0u32;
    for entry in fs::read_dir(&mods_dir)? {
        let entry = entry?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_managed_jar_name(name) {
            continue;
        }
        if keep.contains(name) {
            continue;
        }
        match fs::remove_file(entry.path()) {
            Ok(()) => {
                removed += 1;
                log::info!(target: "rslauncher", "[mods] removed orphan {name}");
            }
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[mods] could not remove orphan {name}: {err}"
                );
            }
        }
    }

    if removed > 0 || !dropped_conflicts.is_empty() {
        log::info!(
            target: "rslauncher",
            "[mods] reconcile done — conflicts={}, orphans={}, keep={}",
            dropped_conflicts.len(),
            removed,
            keep.len()
        );
    } else {
        log::debug!(
            target: "rslauncher",
            "[mods] reconcile — nothing to remove (keep={})",
            keep.len()
        );
    }

    Ok(())
}

fn is_managed_jar_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".jar") || lower.ends_with(".jar.disabled")
}

fn mods_json_path(archive: &Path) -> PathBuf {
    // `….archive` → `….mods.json` (stem strips only the final extension).
    archive.with_extension("mods.json")
}

async fn resolve_pack_archive_url() -> Result<Option<String>, AppError> {
    let profile = modpack_profile::get();
    let Some(pack) = profile.pack.as_ref() else {
        return Ok(None);
    };
    match pack.provider {
        PackProvider::Curseforge => {
            let project_id = pack.project_id.ok_or_else(|| {
                AppError::msg("modpack.toml: curseforge pack missing project_id")
            })?;
            let file_id = pack.file_id.ok_or_else(|| {
                AppError::msg("modpack.toml: curseforge pack missing file_id")
            })?;
            curseforge::modpack::resolve_cf_modpack_url(&ModpackSource::CurseForgePinned {
                project_id,
                file_id,
            })
            .await
            .map(Some)
            .map_err(|e| AppError::msg(format!("resolve CurseForge pack URL: {e}")))
        }
        PackProvider::Modrinth => {
            let project = pack.project.clone().ok_or_else(|| {
                AppError::msg("modpack.toml: modrinth pack missing project")
            })?;
            modrinth::modpack::resolve_mrpack_url(&ModpackSource::ModrinthPinned {
                project,
                version: pack.version.clone(),
            })
            .await
            .map(Some)
            .map_err(|e| AppError::msg(format!("resolve Modrinth pack URL: {e}")))
        }
    }
}

async fn resolve_pack_archive_path() -> Result<Option<PathBuf>, AppError> {
    let Some(url) = resolve_pack_archive_url().await? else {
        return Ok(None);
    };
    let sha1 = calculate_sha1_bytes(url.as_bytes());
    let path = AppState::cache_dir()
        .join("modpacks")
        .join(format!("{sha1}.archive"));
    if path.is_file() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

fn load_pack_mod_jars_from_cache(archive: &Path) -> Result<HashSet<String>, AppError> {
    let path = mods_json_path(archive);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Ok(HashSet::new()),
    };
    let cached: Vec<CachedMod> = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(err) => {
            log::warn!(
                target: "rslauncher",
                "[mods] could not parse {}: {err}",
                path.display()
            );
            return Ok(HashSet::new());
        }
    };

    let mut jars = HashSet::new();
    for entry in cached {
        if let Some(name) = jar_name_under_mods(&entry) {
            jars.insert(name);
        }
    }
    Ok(jars)
}

fn jar_name_under_mods(entry: &CachedMod) -> Option<String> {
    let path = entry.path.as_deref()?;
    let normalized = path.replace('\\', "/");
    let rest = normalized.strip_prefix("mods/")?;
    if rest.is_empty() || rest.contains('/') {
        return None;
    }
    let lower = rest.to_ascii_lowercase();
    if lower.ends_with(".jar") {
        Some(rest.to_string())
    } else {
        None
    }
}

fn current_pack_key() -> String {
    let profile = modpack_profile::get();
    match profile.pack.as_ref() {
        Some(pack) => match pack.provider {
            PackProvider::Curseforge => format!(
                "curseforge:{}:{}",
                pack.project_id.unwrap_or(0),
                pack.file_id.unwrap_or(0)
            ),
            PackProvider::Modrinth => format!(
                "modrinth:{}:{}",
                pack.project.as_deref().unwrap_or(""),
                pack.version.as_deref().unwrap_or("latest")
            ),
        },
        None => {
            // Fingerprint of required CF pins so orphan cleanup refreshes on re-import.
            let mut parts: Vec<String> = profile
                .required_curseforge
                .iter()
                .map(|m| format!("{}:{}", m.project_id, m.file_id.unwrap_or(0)))
                .collect();
            parts.sort();
            format!(
                "manifest:{}:{}:{}",
                profile.id,
                profile.display_version,
                parts.join(",")
            )
        }
    }
}

/// When `full` is true (pack file_id changed), overwrite the whole overrides
/// tree. Otherwise only refresh `overrides/mods/*.jar` (cheap path).
fn apply_pack_overrides(
    archive: &Path,
    game_dir: &Path,
    full: bool,
) -> Result<HashSet<String>, AppError> {
    let file = fs::File::open(archive)
        .map_err(|e| AppError::msg(format!("open pack archive {}: {e}", archive.display())))?;
    let mut zip = ZipArchive::new(file)
        .map_err(|e| AppError::msg(format!("read pack archive {}: {e}", archive.display())))?;

    let mut override_jars = HashSet::new();
    let mut written = 0u32;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| AppError::msg(format!("zip entry: {e}")))?;
        if entry.is_dir() {
            continue;
        }
        let Some(rel) = override_rel_path(entry.name()) else {
            continue;
        };

        let is_mod_jar = rel.starts_with("mods/")
            && !rel[5..].contains('/')
            && rel.to_ascii_lowercase().ends_with(".jar");

        if !full && !is_mod_jar {
            continue;
        }

        let dest = game_dir.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::msg(format!("mkdir {}: {e}", parent.display()))
            })?;
        }
        let mut out = fs::File::create(&dest).map_err(|e| {
            AppError::msg(format!("write override {}: {e}", dest.display()))
        })?;
        std::io::copy(&mut entry, &mut out).map_err(|e| {
            AppError::msg(format!("extract override {}: {e}", dest.display()))
        })?;
        out.flush().ok();
        written += 1;

        if is_mod_jar {
            let jar_name = rel.trim_start_matches("mods/").to_string();
            log::info!(target: "rslauncher", "[mods] override jar → {jar_name}");
            override_jars.insert(jar_name);
        }
    }

    if full {
        log::info!(
            target: "rslauncher",
            "[mods] wrote {written} override file(s) from pack archive"
        );
    }

    Ok(override_jars)
}

fn override_rel_path(entry_name: &str) -> Option<String> {
    let normalized = entry_name.replace('\\', "/");
    for prefix in ["overrides/", "client-overrides/"] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            if rest.is_empty() || rest.ends_with('/') {
                return None;
            }
            // Zip-slip guard.
            if rest.split('/').any(|p| p == "..") {
                return None;
            }
            return Some(rest.to_string());
        }
    }
    None
}

fn mod_ids_for_jars(mods_dir: &Path, jars: &HashSet<String>) -> Result<HashSet<String>, AppError> {
    let mut ids = HashSet::new();
    for name in jars {
        if let Some(id) = read_primary_mod_id(&mods_dir.join(name)) {
            ids.insert(id);
        }
    }
    Ok(ids)
}

fn drop_modid_conflicts(
    mods_dir: &Path,
    override_jars: &HashSet<String>,
    override_mod_ids: &HashSet<String>,
) -> Result<HashSet<String>, AppError> {
    if override_mod_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let mut dropped = HashSet::new();
    let entries: Vec<_> = fs::read_dir(mods_dir)?.flatten().collect();
    for entry in entries {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.to_ascii_lowercase().ends_with(".jar") {
            continue;
        }
        if override_jars.contains(name) {
            continue;
        }
        let Some(id) = read_primary_mod_id(&entry.path()) else {
            continue;
        };
        if !override_mod_ids.contains(&id) {
            continue;
        }
        match fs::remove_file(entry.path()) {
            Ok(()) => {
                log::info!(
                    target: "rslauncher",
                    "[mods] removed {name} (replaced by override modId {id})"
                );
                dropped.insert(name.to_string());
            }
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[mods] could not remove conflicting {name}: {err}"
                );
            }
        }
    }
    Ok(dropped)
}

fn read_primary_mod_id(jar: &Path) -> Option<String> {
    let file = fs::File::open(jar).ok()?;
    let mut zip = ZipArchive::new(file).ok()?;

    for candidate in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(mut entry) = zip.by_name(candidate) {
            let mut text = String::new();
            entry.read_to_string(&mut text).ok()?;
            if let Some(id) = first_toml_mod_id(&text) {
                return Some(id);
            }
        }
    }

    if let Ok(mut entry) = zip.by_name("fabric.mod.json") {
        let mut text = String::new();
        entry.read_to_string(&mut text).ok()?;
        let meta: serde_json::Value = serde_json::from_str(&text).ok()?;
        return meta.get("id")?.as_str().map(|s| s.to_string());
    }

    None
}

fn first_toml_mod_id(text: &str) -> Option<String> {
    // Prefer the first [[mods]] block's modId.
    let mut in_mods = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_mods = trimmed == "[[mods]]";
            continue;
        }
        if !in_mods {
            // Fallback: any top-level modId= before blocks (rare).
            if let Some(id) = parse_mod_id_assignment(trimmed) {
                return Some(id);
            }
            continue;
        }
        if let Some(id) = parse_mod_id_assignment(trimmed) {
            return Some(id);
        }
    }
    None
}

fn parse_mod_id_assignment(line: &str) -> Option<String> {
    let rest = line.strip_prefix("modId")?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim();
    let rest = rest.strip_prefix('"')?.strip_suffix('"')?;
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

async fn extend_with_extra_jars(keep: &mut HashSet<String>) {
    let profile = modpack_profile::get();

    for req in &profile.required_curseforge {
        match resolve_curseforge_jar(req.project_id, req.file_id).await {
            Ok(name) => {
                keep.insert(name);
            }
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[mods] required CF {}/{:?}: {err}",
                    req.project_id,
                    req.file_id
                );
            }
        }
    }

    for req in &profile.required_modrinth {
        match resolve_modrinth_jar(&req.project, req.version.as_deref()).await {
            Ok(name) => {
                keep.insert(name);
            }
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[mods] required Modrinth {}: {err}",
                    req.project
                );
            }
        }
    }

    for opt in optional_mods() {
        match optional_content::resolve_jar_filename(opt).await {
            Ok(name) => {
                keep.insert(name.clone());
                keep.insert(format!("{name}.disabled"));
            }
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[mods] optional {}: {err}",
                    opt.id
                );
            }
        }
    }
}

async fn resolve_curseforge_jar(project_id: u32, file_id: Option<u32>) -> Result<String, AppError> {
    let file_id = file_id.ok_or_else(|| {
        AppError::msg(format!(
            "required CurseForge mod {project_id} needs a pinned file_id"
        ))
    })?;
    let file = curseforge::fetch_pinned_file(project_id, file_id)
        .await
        .map_err(|e| AppError::msg(format!("CurseForge {project_id}/{file_id}: {e}")))?;
    Ok(file.file_name)
}

async fn resolve_modrinth_jar(project: &str, version: Option<&str>) -> Result<String, AppError> {
    let mc = &modpack_profile::get().minecraft;
    let via_connector = optional_mods()
        .iter()
        .find(|m| m.project.as_deref() == Some(project))
        .map(|m| m.via_connector)
        .unwrap_or(false);
    let request = if via_connector {
        ModRequest::modrinth_via_connector(project.to_string(), version.map(|s| s.to_string()))
    } else {
        ModRequest::modrinth(project.to_string(), version.map(|s| s.to_string()))
    };
    let (resolved, _deps) = modrinth::fetch(
        &request,
        mc,
        &modpack_profile::get().loader,
        std::time::Duration::from_secs(6 * 60 * 60),
    )
    .await
    .map_err(|e| AppError::msg(format!("Modrinth {project}: {e}")))?;

    let path = resolved
        .path
        .ok_or_else(|| AppError::msg(format!("Modrinth {project}: resolved mod has no path")))?;
    Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::msg(format!("Modrinth {project}: invalid file path {path}")))
}
