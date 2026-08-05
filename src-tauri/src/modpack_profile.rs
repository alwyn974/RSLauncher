//! Modpack profile (`src-tauri/modpack/modpack.toml`).
//!
//! At startup we fetch the live TOML from GitHub (`config::MODPACK_TOML_URL`) and
//! only fall back to the compile-time embedded copy if the fetch or parse fails.

use std::sync::OnceLock;
use std::time::Duration;

use include_dir::{include_dir, Dir};
use lighty_launcher::core::hosts::HTTP_CLIENT;
use lighty_launcher::prelude::Loader;
use serde::Deserialize;

use crate::config;
use crate::error::AppError;

static SHADERPACKS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/modpack/shaderpacks");
static PROFILE: OnceLock<ModpackProfile> = OnceLock::new();

const EMBEDDED_TOML: &str = include_str!("../modpack/modpack.toml");
const REMOTE_FETCH_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackProvider {
    Curseforge,
    Modrinth,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackSpec {
    pub provider: PackProvider,
    pub project_id: Option<u32>,
    pub file_id: Option<u32>,
    pub file_name: Option<String>,
    pub project: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerDefaults {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredCurseForge {
    pub project_id: u32,
    pub file_id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredModrinth {
    pub project: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModProvider {
    Curseforge,
    Modrinth,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionalModSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default_enabled: bool,
    pub provider: ModProvider,
    pub project_id: Option<u32>,
    pub file_id: Option<u32>,
    pub project: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ShaderSpecRaw {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pack_name: String,
    pub config_file: String,
    #[serde(default)]
    pub default_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ShaderVariant {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pack_name: String,
    pub config_txt: String,
    pub default_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ProfileToml {
    pub instance_name: String,
    pub display_name: Option<String>,
    pub display_version: Option<String>,
    pub minecraft: String,
    pub loader: String,
    pub loader_version: String,
    pub fallback_ram_gb: u32,
    #[serde(default = "default_min_ram")]
    pub min_ram_gb: u32,
    pub pack: PackSpec,
    pub server: ServerDefaults,
    #[serde(default)]
    pub required_curseforge: Vec<RequiredCurseForge>,
    #[serde(default)]
    pub required_modrinth: Vec<RequiredModrinth>,
    #[serde(default)]
    pub optional: Vec<OptionalModSpec>,
    #[serde(default)]
    pub shaders: Vec<ShaderSpecRaw>,
}

fn default_min_ram() -> u32 {
    2
}

#[derive(Debug, Clone)]
pub struct ModpackProfile {
    pub instance_name: String,
    pub display_name: String,
    pub display_version: String,
    pub minecraft: String,
    pub loader: Loader,
    pub loader_version: String,
    pub fallback_ram_gb: u32,
    pub min_ram_gb: u32,
    pub pack: PackSpec,
    pub server: ServerDefaults,
    pub required_curseforge: Vec<RequiredCurseForge>,
    pub required_modrinth: Vec<RequiredModrinth>,
    pub optional: Vec<OptionalModSpec>,
    pub shaders: Vec<ShaderVariant>,
}

impl ModpackProfile {
    pub fn curseforge_pack(&self) -> Option<(u32, u32, &str)> {
        match self.pack.provider {
            PackProvider::Curseforge => Some((
                self.pack.project_id?,
                self.pack.file_id?,
                self.pack.file_name.as_deref().unwrap_or(""),
            )),
            PackProvider::Modrinth => None,
        }
    }

    pub fn loader_label(&self) -> &'static str {
        loader_label(&self.loader)
    }

    pub fn loader_display(&self) -> String {
        format!("{} {}", self.loader_label(), self.loader_version)
    }
}

pub fn parse_loader(name: &str) -> Result<Loader, AppError> {
    match name.trim().to_ascii_lowercase().as_str() {
        "neoforge" => Ok(Loader::NeoForge),
        "forge" => Ok(Loader::Forge),
        "fabric" => Ok(Loader::Fabric),
        "quilt" => Ok(Loader::Quilt),
        "vanilla" => Ok(Loader::Vanilla),
        "optifine" => Ok(Loader::Optifine),
        other => Err(AppError::msg(format!(
            "modpack.toml: unsupported loader {other:?} \
             (fabric, quilt, forge, neoforge, vanilla, optifine)"
        ))),
    }
}

pub fn loader_label(loader: &Loader) -> &'static str {
    match loader {
        Loader::NeoForge => "NeoForge",
        Loader::Forge => "Forge",
        Loader::Fabric => "Fabric",
        Loader::Quilt => "Quilt",
        Loader::Vanilla => "Vanilla",
        Loader::Optifine => "OptiFine",
        Loader::LightyUpdater => "LightyUpdater",
    }
}

/// Load profile from GitHub when possible; otherwise use the embedded TOML.
/// Panics only if the embedded fallback itself is invalid.
pub fn init() {
    let profile = PROFILE.get_or_init(resolve_profile);
    log::info!(
        target: "rslauncher",
        "[modpack] {} · {} · MC {}",
        profile.display_name,
        profile.loader_display(),
        profile.minecraft
    );
}

pub fn get() -> &'static ModpackProfile {
    PROFILE.get().expect("modpack_profile::init() not called")
}

fn resolve_profile() -> ModpackProfile {
    match fetch_remote_blocking() {
        Ok(profile) => {
            log::info!(
                target: "rslauncher",
                "[modpack] loaded profile from GitHub"
            );
            profile
        }
        Err(err) => {
            log::warn!(
                target: "rslauncher",
                "[modpack] remote fetch failed ({err}); using embedded fallback"
            );
            load_embedded().expect("invalid embedded modpack.toml")
        }
    }
}

fn fetch_remote_blocking() -> Result<ModpackProfile, AppError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| AppError::msg(format!("tokio runtime: {e}")))?;
    rt.block_on(fetch_remote())
}

async fn fetch_remote() -> Result<ModpackProfile, AppError> {
    let response = HTTP_CLIENT
        .get(config::MODPACK_TOML_URL)
        .timeout(REMOTE_FETCH_TIMEOUT)
        .header(
            "User-Agent",
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| AppError::msg(format!("GET modpack.toml: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::msg(format!("GET modpack.toml: {e}")))?;

    let body = response
        .text()
        .await
        .map_err(|e| AppError::msg(format!("read modpack.toml: {e}")))?;

    parse_profile(&body)
}

fn load_embedded() -> Result<ModpackProfile, AppError> {
    parse_profile(EMBEDDED_TOML)
}

fn parse_profile(toml_str: &str) -> Result<ModpackProfile, AppError> {
    let raw: ProfileToml = toml::from_str(toml_str)
        .map_err(|e| AppError::msg(format!("modpack.toml: {e}")))?;

    let loader = parse_loader(&raw.loader)?;

    match raw.pack.provider {
        PackProvider::Curseforge => {
            if raw.pack.project_id.is_none() || raw.pack.file_id.is_none() {
                return Err(AppError::msg(
                    "modpack.toml: curseforge pack needs project_id and file_id",
                ));
            }
        }
        PackProvider::Modrinth => {
            if raw.pack.project.as_deref().unwrap_or("").is_empty() {
                return Err(AppError::msg(
                    "modpack.toml: modrinth pack needs project",
                ));
            }
        }
    }

    let mut shaders = Vec::with_capacity(raw.shaders.len());
    for spec in raw.shaders {
        let file = SHADERPACKS.get_file(&spec.config_file).ok_or_else(|| {
            AppError::msg(format!(
                "modpack.toml: shader config_file {:?} not found in modpack/shaderpacks/",
                spec.config_file
            ))
        })?;
        let config_txt = file
            .contents_utf8()
            .ok_or_else(|| {
                AppError::msg(format!(
                    "modpack.toml: shader config_file {:?} is not UTF-8",
                    spec.config_file
                ))
            })?
            .to_string();
        shaders.push(ShaderVariant {
            id: spec.id,
            name: spec.name,
            description: spec.description,
            pack_name: spec.pack_name,
            config_txt,
            default_enabled: spec.default_enabled,
        });
    }

    for opt in &raw.optional {
        match opt.provider {
            ModProvider::Curseforge => {
                if opt.project_id.is_none() {
                    return Err(AppError::msg(format!(
                        "modpack.toml: optional {} needs project_id",
                        opt.id
                    )));
                }
            }
            ModProvider::Modrinth => {
                if opt.project.as_deref().unwrap_or("").is_empty() {
                    return Err(AppError::msg(format!(
                        "modpack.toml: optional {} needs project",
                        opt.id
                    )));
                }
            }
        }
    }

    let display_name = raw
        .display_name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| raw.instance_name.clone());
    let display_version = raw.display_version.unwrap_or_default();

    Ok(ModpackProfile {
        instance_name: raw.instance_name,
        display_name,
        display_version,
        minecraft: raw.minecraft,
        loader,
        loader_version: raw.loader_version,
        fallback_ram_gb: raw.fallback_ram_gb,
        min_ram_gb: raw.min_ram_gb,
        pack: raw.pack,
        server: raw.server,
        required_curseforge: raw.required_curseforge,
        required_modrinth: raw.required_modrinth,
        optional: raw.optional,
        shaders,
    })
}
