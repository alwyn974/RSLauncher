//! Ensure a default multiplayer server exists in the instance `servers.dat`.
//!
//! Format: gzip-compressed NBT compound with a `servers` list (vanilla Minecraft).

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(rename = "acceptTextures", default, skip_serializing_if = "Option::is_none")]
    accept_textures: Option<i8>,
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
    } else {
        root.servers.insert(
            0,
            ServerEntry {
                name: display_name,
                ip: address.to_string(),
                icon: None,
                accept_textures: Some(1),
            },
        );
    }

    write_servers(&path, &root)
}

fn read_servers(path: &Path) -> Result<ServersRoot, AppError> {
    let bytes = fs::read(path)?;
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut inflated = Vec::new();
    decoder
        .read_to_end(&mut inflated)
        .map_err(|e| AppError::msg(format!("servers.dat gunzip: {e}")))?;
    fastnbt::from_bytes(&inflated).map_err(|e| AppError::msg(format!("servers.dat nbt: {e}")))
}

fn write_servers(path: &Path, root: &ServersRoot) -> Result<(), AppError> {
    let nbt = fastnbt::to_bytes(root).map_err(|e| AppError::msg(format!("servers.dat nbt: {e}")))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&nbt)
        .map_err(|e| AppError::msg(format!("servers.dat gzip: {e}")))?;
    let gz = encoder
        .finish()
        .map_err(|e| AppError::msg(format!("servers.dat gzip finish: {e}")))?;
    fs::write(path, gz)?;
    Ok(())
}
