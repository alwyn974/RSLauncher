//! Catalogue of required extras, optional mods, bundles, and shader variants (active pack).

use std::collections::HashSet;

use serde::Serialize;

use crate::dto::Settings;
use crate::modpack_profile::{self, BundleSpec, ModProvider, OptionalModSpec, ShaderVariant};

pub fn optional_mods() -> &'static [OptionalModSpec] {
    &modpack_profile::get().optional
}

pub fn bundles() -> &'static [BundleSpec] {
    &modpack_profile::get().bundles
}

pub fn shader_variants() -> &'static [ShaderVariant] {
    &modpack_profile::get().shaders
}

pub fn required_curseforge() -> Vec<(u32, Option<u32>)> {
    modpack_profile::get()
        .required_curseforge
        .iter()
        .map(|m| (m.project_id, m.file_id))
        .collect()
}

pub fn required_modrinth() -> Vec<(String, Option<String>)> {
    modpack_profile::get()
        .required_modrinth
        .iter()
        .map(|m| (m.project.clone(), m.version.clone()))
        .collect()
}

fn bundled_mod_ids() -> HashSet<&'static str> {
    bundles()
        .iter()
        .flat_map(|b| b.mods.iter().map(|id| id.as_str()))
        .collect()
}

fn find_optional(mod_id: &str) -> Option<&'static OptionalModSpec> {
    optional_mods().iter().find(|m| m.id == mod_id)
}

fn find_bundle_for_mod(mod_id: &str) -> Option<&'static BundleSpec> {
    bundles()
        .iter()
        .find(|b| b.mods.iter().any(|id| id == mod_id))
}

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
pub struct BundleMemberDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub default_enabled: bool,
    pub required: bool,
    pub enabled: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalBundleDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_enabled: bool,
    pub enabled: bool,
    pub mods: Vec<BundleMemberDto>,
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
pub struct ModpackInfoDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub minecraft: String,
    pub loader: String,
    pub loader_version: String,
    pub mod_count: u32,
    pub instance_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub modpack: ModpackInfoDto,
    pub optional_mods: Vec<OptionalModDto>,
    pub optional_bundles: Vec<OptionalBundleDto>,
    pub shader_variants: Vec<ShaderVariantDto>,
}

/// Bundle is ON when every required member is enabled.
pub fn is_bundle_enabled(settings: &Settings, bundle: &BundleSpec) -> bool {
    let required = if bundle.required.is_empty() {
        bundle.mods.as_slice()
    } else {
        bundle.required.as_slice()
    };
    required
        .iter()
        .all(|id| is_optional_mod_enabled(settings, id))
}

/// Required members are locked while the bundle is ON.
pub fn bundle_member_locked(settings: &Settings, bundle: &BundleSpec, mod_id: &str) -> bool {
    let is_required = if bundle.required.is_empty() {
        bundle.mods.iter().any(|id| id == mod_id)
    } else {
        bundle.required.iter().any(|id| id == mod_id)
    };
    is_required && is_bundle_enabled(settings, bundle)
}

pub fn is_optional_mod_enabled(settings: &Settings, mod_id: &str) -> bool {
    if let Some(v) = settings.enabled_optional_mods.get(mod_id) {
        return *v;
    }
    if let Some(bundle) = find_bundle_for_mod(mod_id) {
        if bundle.default_enabled {
            return true;
        }
    }
    find_optional(mod_id)
        .map(|m| m.default_enabled)
        .unwrap_or(false)
}

pub fn is_shader_variant_enabled(settings: &Settings, variant_id: &str) -> bool {
    if let Some(v) = settings.enabled_shader_variants.get(variant_id) {
        return *v;
    }
    shader_variants()
        .iter()
        .find(|s| s.id == variant_id)
        .map(|s| s.default_enabled)
        .unwrap_or(false)
}

fn optional_source_label(m: &OptionalModSpec) -> String {
    match m.provider {
        ModProvider::Curseforge => match (m.project_id, m.file_id) {
            (Some(pid), Some(fid)) => format!("curseforge:{pid}/{fid}"),
            (Some(pid), None) => format!("curseforge:{pid}"),
            _ => "curseforge:?".into(),
        },
        ModProvider::Modrinth => match (&m.project, &m.version) {
            (Some(p), Some(v)) => format!("modrinth:{p}@{v}"),
            (Some(p), None) => format!("modrinth:{p}"),
            _ => "modrinth:?".into(),
        },
    }
}

fn optional_mod_dto(m: &OptionalModSpec, settings: &Settings) -> OptionalModDto {
    OptionalModDto {
        id: m.id.clone(),
        name: m.name.clone(),
        description: m.description.clone(),
        source: optional_source_label(m),
        default_enabled: m.default_enabled,
        enabled: is_optional_mod_enabled(settings, &m.id),
    }
}

fn optional_bundle_dto(bundle: &BundleSpec, settings: &Settings) -> OptionalBundleDto {
    let enabled = is_bundle_enabled(settings, bundle);
    let mods = bundle
        .mods
        .iter()
        .filter_map(|mod_id| {
            let m = find_optional(mod_id)?;
            let required = if bundle.required.is_empty() {
                true
            } else {
                bundle.required.iter().any(|id| id == mod_id)
            };
            Some(BundleMemberDto {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                source: optional_source_label(m),
                default_enabled: if bundle.default_enabled {
                    true
                } else {
                    m.default_enabled
                },
                required,
                enabled: is_optional_mod_enabled(settings, &m.id),
                locked: bundle_member_locked(settings, bundle, &m.id),
            })
        })
        .collect();

    OptionalBundleDto {
        id: bundle.id.clone(),
        name: bundle.name.clone(),
        description: bundle.description.clone(),
        default_enabled: bundle.default_enabled,
        enabled,
        mods,
    }
}

pub fn catalog_dto(settings: &Settings) -> CatalogDto {
    let profile = modpack_profile::get();
    let in_bundle = bundled_mod_ids();
    CatalogDto {
        modpack: ModpackInfoDto {
            id: profile.id.clone(),
            name: profile.display_name.clone(),
            version: profile.display_version.clone(),
            minecraft: profile.minecraft.clone(),
            loader: profile.loader_label().to_string(),
            loader_version: profile.loader_version.clone(),
            mod_count: crate::modpack::active_mod_count(),
            instance_name: profile.instance_name.clone(),
        },
        optional_mods: optional_mods()
            .iter()
            .filter(|m| !in_bundle.contains(m.id.as_str()))
            .map(|m| optional_mod_dto(m, settings))
            .collect(),
        optional_bundles: bundles()
            .iter()
            .map(|b| optional_bundle_dto(b, settings))
            .collect(),
        shader_variants: shader_variants()
            .iter()
            .map(|s| ShaderVariantDto {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                pack_name: s.pack_name.clone(),
                default_enabled: s.default_enabled,
                enabled: is_shader_variant_enabled(settings, &s.id),
            })
            .collect(),
    }
}

pub fn enabled_curseforge_optionals(settings: &Settings) -> Vec<(u32, Option<u32>)> {
    optional_mods()
        .iter()
        .filter(|m| is_optional_mod_enabled(settings, &m.id))
        .filter_map(|m| match m.provider {
            ModProvider::Curseforge => Some((m.project_id?, m.file_id)),
            ModProvider::Modrinth => None,
        })
        .collect()
}

pub fn enabled_modrinth_optionals(settings: &Settings) -> Vec<(String, Option<String>)> {
    optional_mods()
        .iter()
        .filter(|m| is_optional_mod_enabled(settings, &m.id))
        .filter(|m| !m.via_connector)
        .filter_map(|m| match m.provider {
            ModProvider::Modrinth => Some((m.project.clone()?, m.version.clone())),
            ModProvider::Curseforge => None,
        })
        .collect()
}

/// Modrinth optionals marked `via_connector` (Fabric-on-NeoForge via Sinytra).
pub fn enabled_modrinth_connector_optionals(
    settings: &Settings,
) -> Vec<(String, Option<String>)> {
    optional_mods()
        .iter()
        .filter(|m| is_optional_mod_enabled(settings, &m.id))
        .filter(|m| m.via_connector)
        .filter_map(|m| match m.provider {
            ModProvider::Modrinth => Some((m.project.clone()?, m.version.clone())),
            ModProvider::Curseforge => None,
        })
        .collect()
}
