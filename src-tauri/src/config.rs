//! Launcher branding + secrets. Pack-specific data lives under `modpack/`.

pub const LAUNCHER_NAME: &str = "RSLauncher";

/// Official Microsoft Entra application (client) ID for RSLauncher.
pub const AZURE_CLIENT_ID: &str = "fa6b7191-d9b5-4d14-9f2b-cd402ef5070f";

/// Base URL for live modpack files on `main` (trailing slash omitted).
pub const MODPACK_BASE_URL: &str =
    "https://raw.githubusercontent.com/alwyn974/RSLauncher/main/src-tauri/modpack";

/// Live modpack index - fetched at startup; embedded TOML is fallback only.
pub const MODPACKS_TOML_URL: &str =
    "https://raw.githubusercontent.com/alwyn974/RSLauncher/main/src-tauri/modpack/modpacks.toml";

/// CurseForge API key baked in at compile time via `build.rs`
/// (`CURSEFORGE_API_KEY` env or `src-tauri/curseforge_api_key` file).
pub fn curseforge_api_key() -> Option<&'static str> {
    option_env!("RSLAUNCHER_CF_API_KEY").filter(|key| !key.is_empty())
}
