use std::path::Path;

use lighty_launcher::mods::ModpackSource;
use lighty_launcher::prelude::*;

use crate::catalog;
use crate::dto::Settings;
use crate::modpack_profile::{self, PackProvider};
use crate::settings;

/// Build the active instance (modpack + required/optional extras).
pub fn build_instance() -> VersionBuilder<Loader> {
    let settings = settings::load().unwrap_or_default();
    build_instance_with(&settings)
}

pub fn build_instance_with(settings: &Settings) -> VersionBuilder<Loader> {
    build_instance_for_with(&modpack_profile::active_id(), settings)
}

/// Build a VersionBuilder for a specific pack id (uses that pack's saved settings when available).
pub fn build_instance_for(pack_id: &str) -> VersionBuilder<Loader> {
    let settings = if pack_id == modpack_profile::active_id() {
        settings::load().unwrap_or_else(|_| Settings::from_profile(
            modpack_profile::get_by_id(pack_id).unwrap_or_else(|| modpack_profile::get()),
        ))
    } else {
        Settings::from_profile(
            modpack_profile::get_by_id(pack_id).unwrap_or_else(|| modpack_profile::get()),
        )
    };
    build_instance_for_with(pack_id, &settings)
}

pub fn build_instance_for_with(pack_id: &str, settings: &Settings) -> VersionBuilder<Loader> {
    let profile = modpack_profile::get_by_id(pack_id).unwrap_or_else(|| modpack_profile::get());

    let mut mods = VersionBuilder::new(
        &profile.instance_name,
        profile.loader.clone(),
        &profile.loader_version,
        &profile.minecraft,
    )
    .with_mod();

    match &profile.pack {
        Some(pack) => match pack.provider {
            PackProvider::Curseforge => {
                let project_id = pack.project_id.expect("validated at init");
                let file_id = pack.file_id.expect("validated at init");
                mods = mods.with_curseforge_modpack(project_id, file_id);
            }
            PackProvider::Modrinth => {
                let project = pack.project.clone().expect("validated at init");
                mods = mods.with_modrinth_modpack(ModpackSource::ModrinthPinned {
                    project,
                    version: pack.version.clone(),
                });
            }
        },
        None => {
            // Manifest-only / extras-only pack: no hosted zip.
        }
    }

    // Catalogue helpers always read the *active* pack. When building a
    // non-active pack for status checks, skip optionals (requireds still needed
    // for a correct game dir layout only when launching the active pack).
    if pack_id == modpack_profile::active_id() {
        let mut cf = catalog::required_curseforge();
        cf.extend(catalog::enabled_curseforge_optionals(settings));
        if !cf.is_empty() {
            mods = mods.with_curseforge_mods(cf);
        }

        let mut mr = catalog::required_modrinth();
        mr.extend(catalog::enabled_modrinth_optionals(settings));
        if !mr.is_empty() {
            mods = mods.with_modrinth_mods(mr);
        }

        let mr_connector = catalog::enabled_modrinth_connector_optionals(settings);
        if !mr_connector.is_empty() {
            mods = mods.with_modrinth_connector_mods(mr_connector);
        }
    } else {
        let cf: Vec<(u32, Option<u32>)> = profile
            .required_curseforge
            .iter()
            .map(|m| (m.project_id, m.file_id))
            .collect();
        if !cf.is_empty() {
            mods = mods.with_curseforge_mods(cf);
        }
        let mr: Vec<(String, Option<String>)> = profile
            .required_modrinth
            .iter()
            .map(|m| (m.project.clone(), m.version.clone()))
            .collect();
        if !mr.is_empty() {
            mods = mods.with_modrinth_mods(mr);
        }
    }

    mods.done()
}

/// True when the active pack looks actually installed (not just an empty/partial dir).
/// `servers.dat` alone must not count as installed.
pub fn is_pack_installed() -> bool {
    is_pack_installed_at(build_instance().game_dirs())
}

pub fn is_pack_installed_at(root: &Path) -> bool {
    let mods_dir = root.join("mods");
    let versions_dir = root.join("versions");
    dir_has_entries(&mods_dir) || dir_has_entries(&versions_dir)
}

/// Count enabled mod jars in the active pack's `mods/` (excludes `*.jar.disabled`).
pub fn active_mod_count() -> u32 {
    active_mod_count_at(build_instance().game_dirs())
}

pub fn active_mod_count_at(root: &Path) -> u32 {
    let mods_dir = root.join("mods");
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            let lower = name.to_ascii_lowercase();
            lower.ends_with(".jar") && !lower.ends_with(".jar.disabled")
        })
        .count() as u32
}

fn dir_has_entries(path: &std::path::Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}
