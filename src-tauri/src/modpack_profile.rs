//! Curated modpack registry (`src-tauri/modpack/modpacks.toml` + `packs/*.toml`).
//!
//! At startup we fetch the live index (and each pack TOML) from GitHub and only
//! fall back to the compile-time embedded copies if fetch/parse fails.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

use include_dir::{include_dir, Dir};
use lighty_launcher::core::hosts::HTTP_CLIENT;
use lighty_launcher::prelude::Loader;
use serde::Deserialize;

use crate::config;
use crate::error::AppError;

static EMBEDDED_PACKS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/modpack/packs");
static SHADERPACKS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/modpack/shaderpacks");
const EMBEDDED_INDEX: &str = include_str!("../modpack/modpacks.toml");

static REGISTRY: OnceLock<ModpackRegistry> = OnceLock::new();
static ACTIVE_ID: RwLock<String> = RwLock::new(String::new());

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
    /// Fabric mod that runs on Forge/NeoForge through Sinytra Connector.
    /// Resolved against the Fabric loader; Fabric API deps are skipped.
    #[serde(default)]
    pub via_connector: bool,
    pub provider: ModProvider,
    pub project_id: Option<u32>,
    pub file_id: Option<u32>,
    pub project: Option<String>,
    pub version: Option<String>,
}

/// Group of optional mods toggled together in Settings.
///
/// Members stay as `[[optional]]` entries (download source of truth). Bundle ON
/// means every id in `required` is enabled; non-required members stay free.
#[derive(Debug, Clone, Deserialize)]
pub struct BundleSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default_enabled: bool,
    pub mods: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
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
    /// Hosted CurseForge / Modrinth pack zip. Omit for manifest-only packs
    /// (mods listed under `[[required_curseforge]]` / `[[required_modrinth]]`).
    #[serde(default)]
    pub pack: Option<PackSpec>,
    pub server: ServerDefaults,
    #[serde(default)]
    pub required_curseforge: Vec<RequiredCurseForge>,
    #[serde(default)]
    pub required_modrinth: Vec<RequiredModrinth>,
    #[serde(default)]
    pub optional: Vec<OptionalModSpec>,
    #[serde(default)]
    pub bundle: Vec<BundleSpec>,
    #[serde(default)]
    pub shaders: Vec<ShaderSpecRaw>,
}

fn default_min_ram() -> u32 {
    2
}

#[derive(Debug, Clone, Deserialize)]
struct IndexToml {
    pub default: String,
    pub packs: Vec<IndexPackEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct IndexPackEntry {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ModpackProfile {
    /// Stable id from `modpacks.toml` (e.g. `atm10`).
    pub id: String,
    pub instance_name: String,
    pub display_name: String,
    pub display_version: String,
    pub minecraft: String,
    pub loader: Loader,
    pub loader_version: String,
    pub fallback_ram_gb: u32,
    pub min_ram_gb: u32,
    pub pack: Option<PackSpec>,
    pub server: ServerDefaults,
    pub required_curseforge: Vec<RequiredCurseForge>,
    pub required_modrinth: Vec<RequiredModrinth>,
    pub optional: Vec<OptionalModSpec>,
    pub bundles: Vec<BundleSpec>,
    pub shaders: Vec<ShaderVariant>,
}

impl ModpackProfile {
    pub fn curseforge_pack(&self) -> Option<(u32, u32, &str)> {
        let pack = self.pack.as_ref()?;
        match pack.provider {
            PackProvider::Curseforge => Some((
                pack.project_id?,
                pack.file_id?,
                pack.file_name.as_deref().unwrap_or(""),
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

struct ModpackRegistry {
    default_id: String,
    /// Insertion order from the index.
    order: Vec<String>,
    packs: HashMap<String, ModpackProfile>,
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

/// Load registry: embedded local packs in debug, GitHub (with embedded fallback) in release.
/// Panics only if the embedded fallback itself is invalid.
pub fn init() {
    let registry = REGISTRY.get_or_init(resolve_registry);
    let initial = crate::settings::peek_active_pack_id()
        .filter(|id| registry.packs.contains_key(id))
        .unwrap_or_else(|| registry.default_id.clone());
    set_active_id(&initial);
    log::info!(
        target: "rslauncher",
        "[modpack] {} pack(s) loaded · active={} · default={}",
        registry.order.len(),
        initial,
        registry.default_id
    );
    for id in &registry.order {
        if let Some(p) = registry.packs.get(id) {
            log::info!(
                target: "rslauncher",
                "[modpack]   {id}: {} · {} · MC {}",
                p.display_name,
                p.loader_display(),
                p.minecraft
            );
        }
    }
}

fn registry() -> &'static ModpackRegistry {
    REGISTRY.get().expect("modpack_profile::init() not called")
}

pub fn default_id() -> &'static str {
    &registry().default_id
}

pub fn ids() -> &'static [String] {
    &registry().order
}

pub fn get_by_id(id: &str) -> Option<&'static ModpackProfile> {
    registry().packs.get(id)
}

/// Active pack profile (follows [`active_id`]).
pub fn get() -> &'static ModpackProfile {
    let id = active_id();
    get_by_id(&id).unwrap_or_else(|| {
        get_by_id(default_id()).expect("default modpack missing from registry")
    })
}

pub fn active_id() -> String {
    ACTIVE_ID
        .read()
        .expect("ACTIVE_ID poisoned")
        .clone()
}

pub fn set_active_id(id: &str) {
    if !registry().packs.contains_key(id) {
        return;
    }
    *ACTIVE_ID.write().expect("ACTIVE_ID poisoned") = id.to_string();
}

/// Switch active pack and persist selection. Returns the new active id.
pub fn set_active(id: &str) -> Result<&'static ModpackProfile, AppError> {
    let profile = get_by_id(id).ok_or_else(|| AppError::msg(format!("Unknown modpack: {id}")))?;
    set_active_id(id);
    crate::settings::set_active_pack_id(id)?;
    Ok(profile)
}

fn resolve_registry() -> ModpackRegistry {
    // Dev builds use the local/embedded catalog so unpublished packs (e.g. a
    // freshly imported manifest) show up without waiting for a GitHub push.
    if cfg!(debug_assertions) {
        log::info!(
            target: "rslauncher",
            "[modpack] debug build — using embedded local packs"
        );
        return load_embedded_registry().expect("invalid embedded modpacks.toml / packs");
    }

    match fetch_remote_registry_blocking() {
        Ok(reg) => {
            log::info!(
                target: "rslauncher",
                "[modpack] loaded index + packs from GitHub"
            );
            reg
        }
        Err(err) => {
            log::warn!(
                target: "rslauncher",
                "[modpack] remote fetch failed ({err}); using embedded fallback"
            );
            load_embedded_registry().expect("invalid embedded modpacks.toml / packs")
        }
    }
}

fn fetch_remote_registry_blocking() -> Result<ModpackRegistry, AppError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| AppError::msg(format!("tokio runtime: {e}")))?;
    rt.block_on(fetch_remote_registry())
}

async fn fetch_remote_registry() -> Result<ModpackRegistry, AppError> {
    let index_body = http_get_text(config::MODPACKS_TOML_URL).await?;
    let index = parse_index(&index_body)?;

    let mut packs = HashMap::new();
    let mut order = Vec::new();

    for entry in &index.packs {
        let url = format!("{}/{}", config::MODPACK_BASE_URL, entry.path);
        let body = match http_get_text(&url).await {
            Ok(b) => b,
            Err(err) => {
                log::warn!(
                    target: "rslauncher",
                    "[modpack] remote pack {} failed ({err}); trying embedded",
                    entry.id
                );
                match load_embedded_pack_toml(&entry.path) {
                    Ok(b) => b,
                    Err(e) => {
                        return Err(AppError::msg(format!(
                            "pack {}: remote and embedded both failed ({e})",
                            entry.id
                        )));
                    }
                }
            }
        };
        let profile = parse_profile(&entry.id, &body)?;
        order.push(entry.id.clone());
        packs.insert(entry.id.clone(), profile);
    }

    if packs.is_empty() {
        return Err(AppError::msg("modpacks.toml: no packs listed"));
    }
    if !packs.contains_key(&index.default) {
        return Err(AppError::msg(format!(
            "modpacks.toml: default {:?} not in packs list",
            index.default
        )));
    }

    Ok(ModpackRegistry {
        default_id: index.default,
        order,
        packs,
    })
}

fn load_embedded_registry() -> Result<ModpackRegistry, AppError> {
    let index = parse_index(EMBEDDED_INDEX)?;
    let mut packs = HashMap::new();
    let mut order = Vec::new();

    for entry in &index.packs {
        let body = load_embedded_pack_toml(&entry.path)?;
        let profile = parse_profile(&entry.id, &body)?;
        order.push(entry.id.clone());
        packs.insert(entry.id.clone(), profile);
    }

    if packs.is_empty() {
        return Err(AppError::msg("embedded modpacks.toml: no packs"));
    }
    if !packs.contains_key(&index.default) {
        return Err(AppError::msg(format!(
            "embedded modpacks.toml: default {:?} missing",
            index.default
        )));
    }

    Ok(ModpackRegistry {
        default_id: index.default,
        order,
        packs,
    })
}

fn load_embedded_pack_toml(path: &str) -> Result<String, AppError> {
    let relative = path.strip_prefix("packs/").unwrap_or(path);
    let file = EMBEDDED_PACKS.get_file(relative).ok_or_else(|| {
        AppError::msg(format!("embedded pack file missing: {path}"))
    })?;
    file.contents_utf8()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::msg(format!("embedded pack {path} is not UTF-8")))
}

async fn http_get_text(url: &str) -> Result<String, AppError> {
    let response = HTTP_CLIENT
        .get(url)
        .timeout(REMOTE_FETCH_TIMEOUT)
        .header(
            "User-Agent",
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| AppError::msg(format!("GET {url}: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::msg(format!("GET {url}: {e}")))?;

    response
        .text()
        .await
        .map_err(|e| AppError::msg(format!("read {url}: {e}")))
}

fn parse_index(toml_str: &str) -> Result<IndexToml, AppError> {
    toml::from_str(toml_str).map_err(|e| AppError::msg(format!("modpacks.toml: {e}")))
}

/// Resolve a shader Iris config: pack-local first, then shared `common/`.
///
/// Layout:
/// - `modpack/shaderpacks/{pack_id}/{config_file}` (override)
/// - `modpack/shaderpacks/common/{config_file}` (shared)
fn resolve_shader_config(
    pack_id: &str,
    config_file: &str,
) -> Result<(String, &'static include_dir::File<'static>), AppError> {
    let local = format!("{pack_id}/{config_file}");
    if let Some(file) = SHADERPACKS.get_file(&local) {
        return Ok((local, file));
    }
    let common = format!("common/{config_file}");
    if let Some(file) = SHADERPACKS.get_file(&common) {
        return Ok((common, file));
    }
    Err(AppError::msg(format!(
        "pack {pack_id}: shader config_file {config_file:?} not found in \
         modpack/shaderpacks/{pack_id}/ or modpack/shaderpacks/common/"
    )))
}

fn parse_profile(pack_id: &str, toml_str: &str) -> Result<ModpackProfile, AppError> {
    let raw: ProfileToml = toml::from_str(toml_str)
        .map_err(|e| AppError::msg(format!("pack {pack_id}: {e}")))?;

    let loader = parse_loader(&raw.loader)?;

    if let Some(pack) = &raw.pack {
        match pack.provider {
            PackProvider::Curseforge => {
                if pack.project_id.is_none() || pack.file_id.is_none() {
                    return Err(AppError::msg(format!(
                        "pack {pack_id}: curseforge pack needs project_id and file_id"
                    )));
                }
            }
            PackProvider::Modrinth => {
                if pack.project.as_deref().unwrap_or("").is_empty() {
                    return Err(AppError::msg(format!(
                        "pack {pack_id}: modrinth pack needs project"
                    )));
                }
            }
        }
    } else if raw.required_curseforge.is_empty() && raw.required_modrinth.is_empty() {
        return Err(AppError::msg(format!(
            "pack {pack_id}: missing [pack] and has no required_curseforge / required_modrinth mods"
        )));
    }

    let mut shaders = Vec::with_capacity(raw.shaders.len());
    for spec in raw.shaders {
        let (relative, file) = resolve_shader_config(pack_id, &spec.config_file)?;
        let config_txt = file
            .contents_utf8()
            .ok_or_else(|| {
                AppError::msg(format!(
                    "pack {pack_id}: shader config_file {relative:?} is not UTF-8"
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

    let optional_ids: std::collections::HashSet<&str> =
        raw.optional.iter().map(|m| m.id.as_str()).collect();

    for opt in &raw.optional {
        match opt.provider {
            ModProvider::Curseforge => {
                if opt.project_id.is_none() {
                    return Err(AppError::msg(format!(
                        "pack {pack_id}: optional {} needs project_id",
                        opt.id
                    )));
                }
            }
            ModProvider::Modrinth => {
                if opt.project.as_deref().unwrap_or("").is_empty() {
                    return Err(AppError::msg(format!(
                        "pack {pack_id}: optional {} needs project",
                        opt.id
                    )));
                }
            }
        }
    }

    let mut claimed_mods: HashMap<&str, &str> = HashMap::new();
    for bundle in &raw.bundle {
        if bundle.mods.is_empty() {
            return Err(AppError::msg(format!(
                "pack {pack_id}: bundle {} needs at least one mod",
                bundle.id
            )));
        }
        for mod_id in &bundle.mods {
            if !optional_ids.contains(mod_id.as_str()) {
                return Err(AppError::msg(format!(
                    "pack {pack_id}: bundle {} references unknown optional {mod_id}",
                    bundle.id
                )));
            }
            if let Some(other) = claimed_mods.insert(mod_id.as_str(), bundle.id.as_str()) {
                return Err(AppError::msg(format!(
                    "pack {pack_id}: optional {mod_id} is in bundles {other} and {}",
                    bundle.id
                )));
            }
        }
        for req in &bundle.required {
            if !bundle.mods.iter().any(|m| m == req) {
                return Err(AppError::msg(format!(
                    "pack {pack_id}: bundle {} required {req} is not in mods",
                    bundle.id
                )));
            }
        }
    }

    let display_name = raw
        .display_name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| raw.instance_name.clone());
    let display_version = raw.display_version.unwrap_or_default();

    Ok(ModpackProfile {
        id: pack_id.to_string(),
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
        bundles: raw.bundle,
        shaders,
    })
}
