use lighty_launcher::prelude::*;

use crate::config;

/// Build the ATM10 instance (CurseForge pack + optional extra mods).
pub fn build_instance() -> VersionBuilder<Loader> {
    let mut mods = VersionBuilder::new(
        config::INSTANCE_NAME,
        Loader::NeoForge,
        config::NEOFORGE_VERSION,
        config::MINECRAFT_VERSION,
    )
    .with_mod()
    .with_curseforge_modpack(config::ATM10_PROJECT_ID, config::ATM10_FILE_ID);

    if !config::EXTRA_MODRINTH_MODS.is_empty() {
        let list: Vec<(String, Option<String>)> = config::EXTRA_MODRINTH_MODS
            .iter()
            .map(|(id, ver)| ((*id).to_string(), ver.map(|v| v.to_string())))
            .collect();
        mods = mods.with_modrinth_mods(list);
    }

    if !config::EXTRA_CURSEFORGE_MODS.is_empty() {
        mods = mods.with_curseforge_mods(config::EXTRA_CURSEFORGE_MODS.to_vec());
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
