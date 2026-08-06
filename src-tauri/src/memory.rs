use sysinfo::System;

use crate::dto::MemoryInfo;
use crate::modpack_meta;
use crate::modpack_profile;

fn min_ram_gb_for(pack_id: &str) -> u32 {
    modpack_profile::get_by_id(pack_id)
        .unwrap_or_else(|| modpack_profile::get())
        .min_ram_gb
}

fn min_ram_gb() -> u32 {
    min_ram_gb_for(&modpack_profile::active_id())
}

/// Total physical RAM in whole GiB (floored).
pub fn total_gb() -> u32 {
    let mut sys = System::new();
    sys.refresh_memory();
    let bytes = sys.total_memory();
    let gib = (bytes / (1024 * 1024 * 1024)) as u32;
    gib.max(min_ram_gb())
}

fn build_info(pack_id: &str, recommended_gb: u32) -> MemoryInfo {
    let total_gb = total_gb();
    let min = min_ram_gb_for(pack_id);
    MemoryInfo {
        total_gb,
        recommended_gb: recommended_gb.min(total_gb).max(min),
        min_gb: min,
    }
}

/// Fast path: cache or CurseForge fallback (no network).
pub fn info_cached() -> MemoryInfo {
    let id = modpack_profile::active_id();
    build_info(&id, modpack_meta::recommended_gb_cached())
}

/// Resolve recommended RAM from CurseForge pack manifest (network if uncached).
pub async fn info_resolved() -> MemoryInfo {
    let id = modpack_profile::active_id();
    build_info(&id, modpack_meta::ensure_recommended_ram().await)
}

pub fn clamp_ram_gb_for(pack_id: &str, ram_gb: u32) -> u32 {
    let max = total_gb();
    ram_gb.clamp(min_ram_gb_for(pack_id), max)
}

pub fn default_ram_gb_for(pack_id: &str) -> u32 {
    clamp_ram_gb_for(pack_id, modpack_meta::recommended_gb_cached_for(pack_id))
}
