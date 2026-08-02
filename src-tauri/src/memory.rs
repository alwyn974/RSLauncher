use sysinfo::System;

use crate::dto::MemoryInfo;
use crate::modpack_meta;
use crate::modpack_profile;

fn min_ram_gb() -> u32 {
    modpack_profile::get().min_ram_gb
}

/// Total physical RAM in whole GiB (floored).
pub fn total_gb() -> u32 {
    let mut sys = System::new();
    sys.refresh_memory();
    let bytes = sys.total_memory();
    let gib = (bytes / (1024 * 1024 * 1024)) as u32;
    gib.max(min_ram_gb())
}

fn build_info(recommended_gb: u32) -> MemoryInfo {
    let total_gb = total_gb();
    let min = min_ram_gb();
    MemoryInfo {
        total_gb,
        recommended_gb: recommended_gb.min(total_gb).max(min),
        min_gb: min,
    }
}

/// Fast path: cache or CurseForge fallback (no network).
pub fn info_cached() -> MemoryInfo {
    build_info(modpack_meta::recommended_gb_cached())
}

/// Resolve recommended RAM from CurseForge pack manifest (network if uncached).
pub async fn info_resolved() -> MemoryInfo {
    build_info(modpack_meta::ensure_recommended_ram().await)
}

pub fn clamp_ram_gb(ram_gb: u32) -> u32 {
    let max = total_gb();
    ram_gb.clamp(min_ram_gb(), max)
}

pub fn default_ram_gb() -> u32 {
    clamp_ram_gb(modpack_meta::recommended_gb_cached())
}
