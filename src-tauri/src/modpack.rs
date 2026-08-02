use lighty_launcher::mods::ModpackSource;
use lighty_launcher::prelude::*;

use crate::catalog;
use crate::dto::Settings;
use crate::modpack_profile::{self, PackProvider};
use crate::settings;

/// Build the configured instance (modpack + required/optional extras).
pub fn build_instance() -> VersionBuilder<Loader> {
    let settings = settings::load().unwrap_or_default();
    build_instance_with(&settings)
}

pub fn build_instance_with(settings: &Settings) -> VersionBuilder<Loader> {
    let profile = modpack_profile::get();

    let mut mods = VersionBuilder::new(
        &profile.instance_name,
        profile.loader.clone(),
        &profile.loader_version,
        &profile.minecraft,
    )
    .with_mod();

    match profile.pack.provider {
        PackProvider::Curseforge => {
            let project_id = profile.pack.project_id.expect("validated at init");
            let file_id = profile.pack.file_id.expect("validated at init");
            mods = mods.with_curseforge_modpack(project_id, file_id);
        }
        PackProvider::Modrinth => {
            let project = profile.pack.project.clone().expect("validated at init");
            mods = mods.with_modrinth_modpack(ModpackSource::ModrinthPinned {
                project,
                version: profile.pack.version.clone(),
            });
        }
    }

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

    mods.done()
}

/// True when the pack looks actually installed (not just an empty/partial dir).
/// `servers.dat` alone must not count as installed.
pub fn is_pack_installed() -> bool {
    let instance = build_instance();
    let root = instance.game_dirs();
    let mods_dir = root.join("mods");
    let versions_dir = root.join("versions");

    dir_has_entries(&mods_dir) || dir_has_entries(&versions_dir)
}

/// Count enabled mod jars in `mods/` (excludes `*.jar.disabled`).
pub fn active_mod_count() -> u32 {
    let instance = build_instance();
    let mods_dir = instance.game_dirs().join("mods");
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
