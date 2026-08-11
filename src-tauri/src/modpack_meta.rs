//! Fetch CurseForge modpack `manifest.json` → `minecraft.recommendedRam` (MB).

use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::OnceLock;

use lighty_launcher::core::hosts::HTTP_CLIENT;
use lighty_launcher::prelude::AppState;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config;
use crate::modpack_profile::{self, ModpackProfile, PackProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedRecommendedRam {
    file_id: u32,
    /// Value from `minecraft.recommendedRam` (megabytes).
    recommended_ram_mb: u32,
}

#[derive(Debug, Deserialize)]
struct CfManifest {
    minecraft: CfMinecraft,
}

#[derive(Debug, Deserialize)]
struct CfMinecraft {
    #[serde(rename = "recommendedRam")]
    recommended_ram: Option<u32>,
}

static FETCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn cache_path_for(pack_id: &str) -> PathBuf {
    AppState::cache_dir().join(format!("modpack-recommended-ram-{pack_id}.json"))
}

fn mb_to_gb(mb: u32) -> u32 {
    // CurseForge stores MB (e.g. 8196 ≈ 8 GiB).
    ((mb as f64) / 1024.0).round().max(1.0) as u32
}

fn profile_for(pack_id: &str) -> &'static ModpackProfile {
    modpack_profile::get_by_id(pack_id).unwrap_or_else(|| modpack_profile::get())
}

fn pack_file_id(profile: &ModpackProfile) -> Option<u32> {
    match profile.pack.as_ref()?.provider {
        PackProvider::Curseforge => profile.pack.as_ref()?.file_id,
        PackProvider::Modrinth => None,
    }
}

pub fn has_cache() -> bool {
    read_cache(&modpack_profile::active_id()).is_some()
}

/// Cached / fallback recommended RAM in GiB for the active pack (no network).
pub fn recommended_gb_cached() -> u32 {
    recommended_gb_cached_for(&modpack_profile::active_id())
}

pub fn recommended_gb_cached_for(pack_id: &str) -> u32 {
    let profile = profile_for(pack_id);
    read_cache(pack_id)
        .map(|c| mb_to_gb(c.recommended_ram_mb))
        .unwrap_or(profile.fallback_ram_gb)
}

fn read_cache(pack_id: &str) -> Option<CachedRecommendedRam> {
    let profile = profile_for(pack_id);
    let file_id = pack_file_id(profile)?;
    let raw = fs::read_to_string(cache_path_for(pack_id)).ok()?;
    let cached: CachedRecommendedRam = serde_json::from_str(&raw).ok()?;
    if cached.file_id != file_id || cached.recommended_ram_mb == 0 {
        return None;
    }
    Some(cached)
}

fn write_cache(pack_id: &str, mb: u32) -> Result<(), String> {
    let profile = profile_for(pack_id);
    let file_id = pack_file_id(profile).ok_or_else(|| "not a CurseForge pack".to_string())?;
    let path = cache_path_for(pack_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let cached = CachedRecommendedRam {
        file_id,
        recommended_ram_mb: mb,
    };
    let raw = serde_json::to_string_pretty(&cached).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

fn forgecdn_url(profile: &ModpackProfile) -> Option<String> {
    let (project_id, file_id, file_name) = profile.curseforge_pack()?;
    let _ = project_id;
    let id = file_id.to_string();
    let (prefix, suffix) = if id.len() >= 4 {
        (&id[..4], &id[4..])
    } else {
        (id.as_str(), "")
    };
    let name = file_name.replace(' ', "%20");
    if name.is_empty() {
        return None;
    }
    Some(format!(
        "https://edge.forgecdn.net/files/{prefix}/{suffix}/{name}"
    ))
}

fn parse_recommended_mb_from_zip(bytes: &[u8]) -> Result<u32, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;
    let mut file = archive
        .by_name("manifest.json")
        .map_err(|e| format!("manifest.json: {e}"))?;
    let mut raw = String::new();
    file.read_to_string(&mut raw).map_err(|e| e.to_string())?;
    let manifest: CfManifest = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    manifest
        .minecraft
        .recommended_ram
        .filter(|mb| *mb > 0)
        .ok_or_else(|| "manifest.json has no minecraft.recommendedRam".into())
}

/// Ensure cache is populated from the CurseForge pack zip (once per file id).
pub async fn ensure_recommended_ram() -> u32 {
    let pack_id = modpack_profile::active_id();
    let profile = profile_for(&pack_id);
    if pack_file_id(profile).is_none() {
        return profile.fallback_ram_gb;
    }

    if let Some(cached) = read_cache(&pack_id) {
        return mb_to_gb(cached.recommended_ram_mb);
    }

    let lock = FETCH_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().await;

    if let Some(cached) = read_cache(&pack_id) {
        return mb_to_gb(cached.recommended_ram_mb);
    }

    match fetch_recommended_mb(profile).await {
        Ok(mb) => {
            let _ = write_cache(&pack_id, mb);
            mb_to_gb(mb)
        }
        Err(_) => profile.fallback_ram_gb,
    }
}

async fn fetch_recommended_mb(profile: &ModpackProfile) -> Result<u32, String> {
    let url = match resolve_download_url(profile).await {
        Ok(url) => url,
        Err(_) => forgecdn_url(profile).ok_or_else(|| "no forgecdn URL".to_string())?,
    };

    let response = HTTP_CLIENT
        .get(&url)
        .timeout(std::time::Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    parse_recommended_mb_from_zip(&bytes)
}

async fn resolve_download_url(profile: &ModpackProfile) -> Result<String, String> {
    let (project_id, file_id, _) = profile
        .curseforge_pack()
        .ok_or_else(|| "not a CurseForge pack".to_string())?;
    let key = config::curseforge_api_key()
        .ok_or_else(|| "CurseForge API key missing from this build".to_string())?;
    lighty_launcher::mods::curseforge::set_api_key(key);
    let file = lighty_launcher::mods::curseforge::fetch_pinned_file(project_id, file_id)
        .await
        .map_err(|e| e.to_string())?;
    file.download_url
        .ok_or_else(|| "CurseForge file has no downloadUrl".into())
}
