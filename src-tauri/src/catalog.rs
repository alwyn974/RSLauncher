//! Catalogue of required extras, optional mods, and shader variants (from modpack.toml).

use serde::Serialize;

use crate::dto::Settings;
use crate::modpack_profile::{self, ModProvider, OptionalModSpec, ShaderVariant};

pub fn optional_mods() -> &'static [OptionalModSpec] {
    &modpack_profile::get().optional
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
pub struct ModpackInfoDto {
    pub name: String,
    pub version: String,
    pub minecraft: String,
    pub loader: String,
    pub loader_version: String,
    pub mod_count: Option<u32>,
    pub instance_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub modpack: ModpackInfoDto,
    pub optional_mods: Vec<OptionalModDto>,
    pub shader_variants: Vec<ShaderVariantDto>,
}

pub fn is_optional_mod_enabled(settings: &Settings, mod_id: &str) -> bool {
    if let Some(v) = settings.enabled_optional_mods.get(mod_id) {
        return *v;
    }
    optional_mods()
        .iter()
        .find(|m| m.id == mod_id)
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

pub fn catalog_dto(settings: &Settings) -> CatalogDto {
    let profile = modpack_profile::get();
    CatalogDto {
        modpack: ModpackInfoDto {
            name: profile.display_name.clone(),
            version: profile.display_version.clone(),
            minecraft: profile.minecraft.clone(),
            loader: profile.loader_label().to_string(),
            loader_version: profile.loader_version.clone(),
            mod_count: profile.mod_count,
            instance_name: profile.instance_name.clone(),
        },
        optional_mods: optional_mods()
            .iter()
            .map(|m| OptionalModDto {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                source: optional_source_label(m),
                default_enabled: m.default_enabled,
                enabled: is_optional_mod_enabled(settings, &m.id),
            })
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
        .filter_map(|m| match m.provider {
            ModProvider::Modrinth => Some((m.project.clone()?, m.version.clone())),
            ModProvider::Curseforge => None,
        })
        .collect()
}
