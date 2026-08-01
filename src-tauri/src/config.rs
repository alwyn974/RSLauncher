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

/// Modrinth: `(slug_or_id, optional_version_id)`
pub const EXTRA_MODRINTH_MODS: &[(&str, Option<&str>)] = &[

];
/// CurseForge: `(mod_id, optional_file_id)`
pub const EXTRA_CURSEFORGE_MODS: &[(u32, Option<u32>)] = &[
     (676721,  Some(8240058)),  // create-aeronautics
     (1312371, Some(8263584)), // sable
     (1528764, Some(8464450)), // create-aeronautics-climbable-ropes
     (1524471, Some(8404634)), // waystones-sable
     (1532334, Some(8249093)), // create-aeronautics-x-curios-api-compat
     (1529882, Some(8368780)), // create-aeronautics-throwable-rope-connector
     (1521213, Some(8002670)), // create-aeronautics-portable-engine-liquid-fuel
     (1519765, Some(7968280)), // create-tracks
     (1514529, Some(8493955)), // aeronautics-claims
     // Optional
     (1198238, Some(8150396)),  // discord-chat-connect
     (734339,  Some(7996932)),  // ping-wheel
     (1537977, Some(8189365)),  // ping-to-map
     (1530988, Some(8269260)),  // jade-sable-compat
];

/// Default multiplayer entry written into `servers.dat` when missing.
pub const DEFAULT_SERVER_NAME: &str = "Eh Zebi";
pub const DEFAULT_SERVER_ADDRESS: &str = "mc.alwyn974.re";
