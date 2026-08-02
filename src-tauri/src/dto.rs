use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub avatar_seed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub ram_gb: u32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub jvm_args: String,
    #[serde(default = "default_server_name")]
    pub server_name: String,
    #[serde(default = "default_server_address")]
    pub server_address: String,
    /// Optional catalogue mod id → enabled. Missing keys use catalogue defaults.
    #[serde(default)]
    pub enabled_optional_mods: HashMap<String, bool>,
    /// Shader variant id → enabled. Missing keys use catalogue defaults.
    #[serde(default)]
    pub enabled_shader_variants: HashMap<String, bool>,
}

fn default_server_name() -> String {
    crate::config::DEFAULT_SERVER_NAME.to_string()
}

fn default_server_address() -> String {
    crate::config::DEFAULT_SERVER_ADDRESS.to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ram_gb: crate::config::FALLBACK_RECOMMENDED_RAM_GB,
            width: 1024,
            height: 768,
            fullscreen: false,
            jvm_args: String::new(),
            server_name: default_server_name(),
            server_address: default_server_address(),
            enabled_optional_mods: HashMap::new(),
            enabled_shader_variants: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    /// Total physical RAM on this machine, in whole GiB.
    pub total_gb: u32,
    /// Modpack-recommended allocation, in whole GiB.
    pub recommended_gb: u32,
    pub min_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub stage: String,
    pub step: String,
    pub file: String,
    pub files_done: u32,
    pub files_total: u32,
    pub percent: u32,
    pub bytes_per_sec: f64,
    pub eta_sec: u32,
}

impl Progress {
    pub fn idle() -> Self {
        Self {
            stage: "idle".into(),
            step: String::new(),
            file: String::new(),
            files_done: 0,
            files_total: 0,
            percent: 0,
            bytes_per_sec: 0.0,
            eta_sec: 0,
        }
    }

    pub fn detail(stage: &str, step: &str, detail: impl Into<String>, percent: u32) -> Self {
        Self {
            stage: stage.into(),
            step: step.into(),
            file: detail.into(),
            files_done: 0,
            files_total: 0,
            percent: percent.min(100),
            bytes_per_sec: 0.0,
            eta_sec: 0,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            stage: "error".into(),
            step: "Launch failed".into(),
            file: message.into(),
            files_done: 0,
            files_total: 0,
            percent: 0,
            bytes_per_sec: 0.0,
            eta_sec: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatus {
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePayload {
    pub code: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusPayload {
    pub step: String,
}
