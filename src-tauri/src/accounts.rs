use std::fs;
use std::path::PathBuf;

use lighty_launcher::auth::{ExposeSecret, SecretString};
use lighty_launcher::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::dto::AccountDto;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAccount {
    uuid: String,
    username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AccountsFile {
    accounts: Vec<StoredAccount>,
    active_uuid: Option<String>,
}

fn accounts_path() -> PathBuf {
    AppState::config_dir().join("accounts.json")
}

fn refresh_keyring_user(uuid: &str) -> String {
    format!("microsoft_refresh:{uuid}")
}

fn load_file() -> Result<AccountsFile, AppError> {
    let path = accounts_path();
    if !path.exists() {
        return Ok(AccountsFile::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn save_file(file: &AccountsFile) -> Result<(), AppError> {
    let path = accounts_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(file)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn avatar_seed(uuid: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for byte in uuid.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn to_dto(account: &StoredAccount) -> AccountDto {
    AccountDto {
        id: account.uuid.clone(),
        username: account.username.clone(),
        uuid: account.uuid.clone(),
        avatar_seed: avatar_seed(&account.uuid),
    }
}

pub fn list_accounts() -> Result<Vec<AccountDto>, AppError> {
    let file = load_file()?;
    Ok(file.accounts.iter().map(to_dto).collect())
}

pub fn get_active_account() -> Result<Option<AccountDto>, AppError> {
    let file = load_file()?;
    let Some(active) = file.active_uuid.as_ref() else {
        return Ok(None);
    };
    Ok(file
        .accounts
        .iter()
        .find(|a| &a.uuid == active)
        .map(to_dto))
}

pub fn set_active_account(id: &str) -> Result<(), AppError> {
    let mut file = load_file()?;
    if !file.accounts.iter().any(|a| a.uuid == id) {
        return Err(AppError::msg("Account not found"));
    }
    file.active_uuid = Some(id.to_string());
    save_file(&file)
}

pub fn remove_account(id: &str) -> Result<(), AppError> {
    let mut file = load_file()?;
    file.accounts.retain(|a| a.uuid != id);
    if file.active_uuid.as_deref() == Some(id) {
        file.active_uuid = file.accounts.first().map(|a| a.uuid.clone());
    }
    save_file(&file)?;

    if let Ok(entry) = keyring::Entry::new(config::LAUNCHER_NAME, &refresh_keyring_user(id)) {
        let _ = entry.delete_credential();
    }
    if let Ok(entry) = keyring::Entry::new(config::LAUNCHER_NAME, &format!("microsoft:{id}")) {
        let _ = entry.delete_credential();
    }
    Ok(())
}

pub fn upsert_from_profile(profile: &UserProfile) -> Result<AccountDto, AppError> {
    let mut file = load_file()?;
    if let Some(existing) = file.accounts.iter_mut().find(|a| a.uuid == profile.uuid) {
        existing.username = profile.username.clone();
    } else {
        file.accounts.push(StoredAccount {
            uuid: profile.uuid.clone(),
            username: profile.username.clone(),
        });
    }
    file.active_uuid = Some(profile.uuid.clone());

    if let AuthProvider::Microsoft {
        refresh_token: Some(rt),
        ..
    } = &profile.provider
    {
        save_refresh_token(&profile.uuid, rt)?;
    }

    save_file(&file)?;
    Ok(AccountDto {
        id: profile.uuid.clone(),
        username: profile.username.clone(),
        uuid: profile.uuid.clone(),
        avatar_seed: avatar_seed(&profile.uuid),
    })
}

pub fn save_refresh_token(uuid: &str, token: &SecretString) -> Result<(), AppError> {
    let entry = keyring::Entry::new(config::LAUNCHER_NAME, &refresh_keyring_user(uuid))
        .map_err(|e| AppError::msg(e.to_string()))?;
    entry
        .set_password(token.expose_secret())
        .map_err(|e| AppError::msg(e.to_string()))?;
    Ok(())
}

pub fn load_refresh_token(uuid: &str) -> Option<SecretString> {
    let entry = keyring::Entry::new(config::LAUNCHER_NAME, &refresh_keyring_user(uuid)).ok()?;
    let token = entry.get_password().ok()?;
    Some(SecretString::from(token))
}

pub fn active_uuid() -> Result<Option<String>, AppError> {
    Ok(load_file()?.active_uuid)
}
