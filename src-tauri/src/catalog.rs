//! Baked-in catalogue of required extras, optional mods, and shader variants.

use serde::Serialize;

use crate::dto::Settings;

#[derive(Debug, Clone, Copy)]
pub enum ModSource {
    CurseForge {
        project_id: u32,
        file_id: Option<u32>,
    },
    Modrinth {
        project: &'static str,
        version: Option<&'static str>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct OptionalMod {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub source: ModSource,
    pub default_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderVariant {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Existing pack in `shaderpacks/` (folder or zip stem). Never created by us —
    /// Euphoria Patches (or the user) must already have produced it.
    pub pack_name: &'static str,
    pub config_txt: &'static str,
    pub default_enabled: bool,
}

/// Always installed extras (Create Aeronautics stack, etc.).
pub const REQUIRED_CURSEFORGE_MODS: &[(u32, Option<u32>)] = &[
    (676721, Some(8240058)),   // create-aeronautics
    (1312371, Some(8263584)),  // sable
    (1528764, Some(8464450)),  // create-aeronautics-climbable-ropes
    (1524471, Some(8404634)),  // waystones-sable
    (1532334, Some(8249093)),  // create-aeronautics-x-curios-api-compat
    (1529882, Some(8368780)),  // create-aeronautics-throwable-rope-connector
    (1521213, Some(8002670)),  // create-aeronautics-portable-engine-liquid-fuel
    (1519765, Some(7968280)),  // create-tracks
    (1514529, Some(8493955)),  // aeronautics-claims
];

pub const REQUIRED_MODRINTH_MODS: &[(&str, Option<&str>)] = &[];

pub const OPTIONAL_MODS: &[OptionalMod] = &[
    OptionalMod {
        id: "discord-chat-connect",
        name: "Discord Chat Connect",
        description: "Bridge in-game chat with a Discord channel.",
        source: ModSource::CurseForge {
            project_id: 1_198_238,
            file_id: Some(8_150_396),
        },
        default_enabled: true,
    },
    OptionalMod {
        id: "ping-wheel",
        name: "Ping Wheel",
        description: "Ping locations for your teammates.",
        source: ModSource::CurseForge {
            project_id: 734_339,
            file_id: Some(7_996_932),
        },
        default_enabled: true,
    },
    OptionalMod {
        id: "ping-to-map",
        name: "Ping to Map",
        description: "Show pings on the map.",
        source: ModSource::CurseForge {
            project_id: 1_537_977,
            file_id: Some(8_189_365),
        },
        default_enabled: true,
    },
    OptionalMod {
        id: "jade-sable-compat",
        name: "Jade × Sable Compat",
        description: "Jade tooltips for Sable content.",
        source: ModSource::CurseForge {
            project_id: 1_530_988,
            file_id: Some(8_269_260),
        },
        default_enabled: true,
    },
    OptionalMod {
        id: "obscraft",
        name: "OBSCraft",
        description: "Control OBS Studio from Minecraft (commands / datapacks).",
        source: ModSource::Modrinth {
            project: "obscraft",
            // NeoForge 1.21–1.21.4 — https://modrinth.com/mod/obscraft/version/Dsw8aSfW
            version: Some("Dsw8aSfW"),
        },
        default_enabled: false,
    },
];

/// Write an Iris/Oculus `.txt` next to a pack that already exists on disk.
/// We never create or duplicate shaderpacks (Euphoria must create its folders).
pub const SHADER_VARIANTS: &[ShaderVariant] = &[
    ShaderVariant {
        id: "complementary-unbound-euphoria",
        name: "Complementary Unbound + Euphoria",
        description: "Applies Iris settings once the Euphoria folder exists.",
        pack_name: "ComplementaryUnbound_r5.8.1 + EuphoriaPatches_1.9.3",
        config_txt: include_str!(
            "../resources/shaderpacks/ComplementaryUnbound_r5.8.1 + EuphoriaPatches_1.9.3.txt"
        ),
        default_enabled: false,
    },
    ShaderVariant {
        id: "complementary-reimagined-euphoria",
        name: "Complementary Reimagined + Euphoria",
        description: "Applies Iris settings once the Euphoria folder exists.",
        pack_name: "ComplementaryReimagined_r5.8.1 + EuphoriaPatches_1.9.3",
        config_txt: include_str!(
            "../resources/shaderpacks/ComplementaryReimagined_r5.8.1 + EuphoriaPatches_1.9.3.txt"
        ),
        default_enabled: false,
    },
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalModDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub default_enabled: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderVariantDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pack_name: String,
    pub default_enabled: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub optional_mods: Vec<OptionalModDto>,
    pub shader_variants: Vec<ShaderVariantDto>,
}

pub fn is_optional_mod_enabled(settings: &Settings, mod_id: &str) -> bool {
    if let Some(v) = settings.enabled_optional_mods.get(mod_id) {
        return *v;
    }
    OPTIONAL_MODS
        .iter()
        .find(|m| m.id == mod_id)
        .map(|m| m.default_enabled)
        .unwrap_or(false)
}

pub fn is_shader_variant_enabled(settings: &Settings, variant_id: &str) -> bool {
    if let Some(v) = settings.enabled_shader_variants.get(variant_id) {
        return *v;
    }
    SHADER_VARIANTS
        .iter()
        .find(|s| s.id == variant_id)
        .map(|s| s.default_enabled)
        .unwrap_or(false)
}

pub fn catalog_dto(settings: &Settings) -> CatalogDto {
    CatalogDto {
        optional_mods: OPTIONAL_MODS
            .iter()
            .map(|m| OptionalModDto {
                id: m.id.to_string(),
                name: m.name.to_string(),
                description: m.description.to_string(),
                source: match m.source {
                    ModSource::CurseForge { project_id, file_id } => match file_id {
                        Some(fid) => format!("curseforge:{project_id}/{fid}"),
                        None => format!("curseforge:{project_id}"),
                    },
                    ModSource::Modrinth { project, version } => match version {
                        Some(v) => format!("modrinth:{project}@{v}"),
                        None => format!("modrinth:{project}"),
                    },
                },
                default_enabled: m.default_enabled,
                enabled: is_optional_mod_enabled(settings, m.id),
            })
            .collect(),
        shader_variants: SHADER_VARIANTS
            .iter()
            .map(|s| ShaderVariantDto {
                id: s.id.to_string(),
                name: s.name.to_string(),
                description: s.description.to_string(),
                pack_name: s.pack_name.to_string(),
                default_enabled: s.default_enabled,
                enabled: is_shader_variant_enabled(settings, s.id),
            })
            .collect(),
    }
}

pub fn enabled_curseforge_optionals(settings: &Settings) -> Vec<(u32, Option<u32>)> {
    OPTIONAL_MODS
        .iter()
        .filter(|m| is_optional_mod_enabled(settings, m.id))
        .filter_map(|m| match m.source {
            ModSource::CurseForge {
                project_id,
                file_id,
            } => Some((project_id, file_id)),
            _ => None,
        })
        .collect()
}

pub fn enabled_modrinth_optionals(settings: &Settings) -> Vec<(String, Option<String>)> {
    OPTIONAL_MODS
        .iter()
        .filter(|m| is_optional_mod_enabled(settings, m.id))
        .filter_map(|m| match m.source {
            ModSource::Modrinth { project, version } => {
                Some((project.to_string(), version.map(|v| v.to_string())))
            }
            _ => None,
        })
        .collect()
}
