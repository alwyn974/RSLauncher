use sysinfo::System;

use crate::config;
use crate::dto::MemoryInfo;
use crate::modpack_meta;

/// Total physical RAM in whole GiB (floored).
pub fn total_gb() -> u32 {
    let mut sys = System::new();
    sys.refresh_memory();
    let bytes = sys.total_memory();
    let gib = (bytes / (1024 * 1024 * 1024)) as u32;
    gib.max(config::MIN_RAM_GB)
}

fn build_info(recommended_gb: u32) -> MemoryInfo {
    let total_gb = total_gb();
    MemoryInfo {
        total_gb,
        recommended_gb: recommended_gb.min(total_gb).max(config::MIN_RAM_GB),
        min_gb: config::MIN_RAM_GB,
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
    ram_gb.clamp(config::MIN_RAM_GB, max)
}

pub fn default_ram_gb() -> u32 {
    clamp_ram_gb(modpack_meta::recommended_gb_cached())
}
