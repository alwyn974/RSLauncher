//! Launcher branding + secrets. Pack-specific data lives under `modpack/`.

pub const LAUNCHER_NAME: &str = "RSLauncher";

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

/// Azure AD app (client) ID baked in at compile time via `build.rs`
/// (`AZURE_CLIENT_ID` env or `src-tauri/azure_client_id` file).
pub fn azure_client_id() -> Option<&'static str> {
    option_env!("RSLAUNCHER_AZURE_CLIENT_ID").filter(|id| !id.is_empty())
}
