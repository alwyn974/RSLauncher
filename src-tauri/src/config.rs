pub const LAUNCHER_NAME: &str = "RSLauncher";
pub const INSTANCE_NAME: &str = "ATM10";

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

/// All the Mods 10 — CurseForge project + pinned file (7.2).
/// https://www.curseforge.com/minecraft/modpacks/all-the-mods-10/files/8469481
pub const ATM10_PROJECT_ID: u32 = 925_200;
pub const ATM10_FILE_ID: u32 = 8_469_481;
pub const ATM10_FILE_NAME: &str = "All the Mods 10-7.2.zip";
pub const MINECRAFT_VERSION: &str = "1.21.1";
pub const NEOFORGE_VERSION: &str = "21.1.241";

/// Fallback if CurseForge `minecraft.recommendedRam` can't be fetched yet.
pub const FALLBACK_RECOMMENDED_RAM_GB: u32 = 8;
pub const MIN_RAM_GB: u32 = 2;

/// Default multiplayer entry written into `servers.dat` when missing.
pub const DEFAULT_SERVER_NAME: &str = "Eh Zebi";
pub const DEFAULT_SERVER_ADDRESS: &str = "mc.alwyn974.re";
