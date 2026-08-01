//! Ensure a default multiplayer server exists in the instance `servers.dat`.
//!
//! Modern Minecraft (1.21+) writes **uncompressed** NBT. Older builds used
//! gzip — we read both, and always write uncompressed to match vanilla.

use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ServersRoot {
    #[serde(default)]
    servers: Vec<ServerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerEntry {
    name: String,
    ip: String,
    #[serde(default)]
    hidden: bool,
    /// NeoForge / modern vanilla (chat reporting toggle).
    #[serde(rename = "preventsChatReports", default)]
    prevents_chat_reports: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

impl ServerEntry {
    fn new(name: String, ip: String) -> Self {
        Self {
            name,
            ip,
            hidden: false,
            prevents_chat_reports: false,
            icon: None,
        }
    }
}

/// Upsert the given server into `{game_dir}/servers.dat`.
/// Existing servers are preserved; matched by IP then updated, else inserted first.
pub fn ensure_default_server(
    game_dir: &Path,
    name: &str,
    address: &str,
) -> Result<(), AppError> {
    let name = name.trim();
    let address = address.trim();
    if address.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(game_dir)?;
    let path = game_dir.join("servers.dat");

    let mut root = if path.exists() {
        read_servers(&path).unwrap_or_default()
    } else {
        ServersRoot::default()
    };

    let display_name = if name.is_empty() {
        address.to_string()
    } else {
        name.to_string()
    };

    if let Some(entry) = root.servers.iter_mut().find(|s| s.ip == address) {
        entry.name = display_name;
        entry.hidden = false;
    } else {
        root.servers.insert(0, ServerEntry::new(display_name, address.to_string()));
    }

    write_servers(&path, &root)?;
    log::info!(
        target: "rslauncher",
        "[launcher] servers.dat ready at {} ({} entries)",
        path.display(),
        root.servers.len()
    );
    Ok(())
}

fn read_servers(path: &Path) -> Result<ServersRoot, AppError> {
    let bytes = fs::read(path)?;
    let nbt = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut inflated = Vec::new();
        decoder
            .read_to_end(&mut inflated)
            .map_err(|e| AppError::msg(format!("servers.dat gunzip: {e}")))?;
        inflated
    } else {
        bytes
    };
    fastnbt::from_bytes(&nbt).map_err(|e| AppError::msg(format!("servers.dat nbt: {e}")))
}

fn write_servers(path: &Path, root: &ServersRoot) -> Result<(), AppError> {
    // Vanilla 1.21 writes uncompressed NBT (not gzip).
    let nbt = fastnbt::to_bytes(root).map_err(|e| AppError::msg(format!("servers.dat nbt: {e}")))?;
    fs::write(path, nbt)?;
    Ok(())
}
