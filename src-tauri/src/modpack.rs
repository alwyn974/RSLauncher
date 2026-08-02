use lighty_launcher::prelude::*;

use crate::catalog::{
    self, REQUIRED_CURSEFORGE_MODS, REQUIRED_MODRINTH_MODS,
};
use crate::config;
use crate::dto::Settings;
use crate::settings;

/// Build the ATM10 instance (CurseForge pack + required/optional extras).
pub fn build_instance() -> VersionBuilder<Loader> {
    let settings = settings::load().unwrap_or_default();
    build_instance_with(&settings)
}

pub fn build_instance_with(settings: &Settings) -> VersionBuilder<Loader> {
    let mut mods = VersionBuilder::new(
        config::INSTANCE_NAME,
        Loader::NeoForge,
        config::NEOFORGE_VERSION,
        config::MINECRAFT_VERSION,
    )
    .with_mod()
    .with_curseforge_modpack(config::ATM10_PROJECT_ID, config::ATM10_FILE_ID);

    let mut cf = REQUIRED_CURSEFORGE_MODS.to_vec();
    cf.extend(catalog::enabled_curseforge_optionals(settings));
    if !cf.is_empty() {
        mods = mods.with_curseforge_mods(cf);
    }

    let mut mr: Vec<(String, Option<String>)> = REQUIRED_MODRINTH_MODS
        .iter()
        .map(|(id, ver)| ((*id).to_string(), ver.map(|v| v.to_string())))
        .collect();
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

fn dir_has_entries(path: &std::path::Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}
