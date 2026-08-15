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
    /// OAuth client that issued the stored refresh token. Missing on accounts
    /// created before this migration, which intentionally forces one relogin.
    #[serde(default)]
    auth_client_id: Option<String>,
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

impl StoredAccount {
    fn needs_reauth(&self) -> bool {
        self.auth_client_id.as_deref() != Some(config::AZURE_CLIENT_ID)
    }
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

fn normalize_active_account(file: &mut AccountsFile) {
    let active_is_usable = file
        .active_uuid
        .as_deref()
        .and_then(|uuid| file.accounts.iter().find(|account| account.uuid == uuid))
        .map(|account| !account.needs_reauth())
        .unwrap_or(false);

    if !active_is_usable {
        file.active_uuid = file
            .accounts
            .iter()
            .find(|account| !account.needs_reauth())
            .map(|account| account.uuid.clone());
    }
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
        needs_reauth: account.needs_reauth(),
    }
}

pub fn list_accounts() -> Result<Vec<AccountDto>, AppError> {
    let file = load_file()?;
    Ok(file.accounts.iter().map(to_dto).collect())
}

pub fn get_active_account() -> Result<Option<AccountDto>, AppError> {
    let mut file = load_file()?;
    let previous = file.active_uuid.clone();
    normalize_active_account(&mut file);
    if file.active_uuid != previous {
        save_file(&file)?;
    }
    let Some(active) = file.active_uuid.as_ref() else {
        return Ok(None);
    };
    Ok(file
        .accounts
        .iter()
        .find(|a| &a.uuid == active && !a.needs_reauth())
        .map(to_dto))
}

pub fn set_active_account(id: &str) -> Result<(), AppError> {
    let mut file = load_file()?;
    let account = file
        .accounts
        .iter()
        .find(|a| a.uuid == id)
        .ok_or_else(|| AppError::msg("Account not found"))?;
    if account.needs_reauth() {
        return Err(AppError::msg(
            "This account needs to reconnect with Microsoft",
        ));
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

    clear_credentials(id);
    Ok(())
}

pub fn upsert_from_profile(profile: &UserProfile) -> Result<AccountDto, AppError> {
    let mut file = load_file()?;
    if let Some(existing) = file.accounts.iter_mut().find(|a| a.uuid == profile.uuid) {
        existing.username = profile.username.clone();
        existing.auth_client_id = Some(config::AZURE_CLIENT_ID.to_string());
    } else {
        file.accounts.push(StoredAccount {
            uuid: profile.uuid.clone(),
            username: profile.username.clone(),
            auth_client_id: Some(config::AZURE_CLIENT_ID.to_string()),
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
        needs_reauth: false,
    })
}

fn clear_credentials(uuid: &str) {
    for user in [refresh_keyring_user(uuid), format!("microsoft:{uuid}")] {
        if let Ok(entry) = keyring::Entry::new(config::LAUNCHER_NAME, &user) {
            let _ = entry.delete_credential();
        }
    }
}

pub fn mark_reauth_required(uuid: &str) -> Result<(), AppError> {
    let mut file = load_file()?;
    if let Some(account) = file.accounts.iter_mut().find(|a| a.uuid == uuid) {
        account.auth_client_id = None;
        normalize_active_account(&mut file);
        save_file(&file)?;
    }
    clear_credentials(uuid);
    Ok(())
}

fn persist_refreshed_profile(
    expected_uuid: &str,
    profile: &UserProfile,
) -> Result<(), AppError> {
    if profile.uuid != expected_uuid {
        return Err(AppError::msg(format!(
            "Microsoft returned account {} while refreshing {expected_uuid}",
            profile.uuid
        )));
    }

    if let AuthProvider::Microsoft {
        refresh_token: Some(rt),
        ..
    } = &profile.provider
    {
        save_refresh_token(expected_uuid, rt)?;
    }

    let mut file = load_file()?;
    let account = file
        .accounts
        .iter_mut()
        .find(|a| a.uuid == expected_uuid)
        .ok_or_else(|| AppError::msg("Account disappeared during session refresh"))?;
    account.username = profile.username.clone();
    account.auth_client_id = Some(config::AZURE_CLIENT_ID.to_string());
    save_file(&file)
}

/// Refresh every reusable account when the launcher opens. Microsoft commonly
/// rotates refresh tokens, so each successful response is persisted at once.
pub async fn refresh_sessions() -> Result<(), AppError> {
    let stored = load_file()?.accounts;

    for account in stored {
        if account.needs_reauth() {
            // Tokens issued to the previous App ID cannot be reused.
            clear_credentials(&account.uuid);
            continue;
        }

        let Some(refresh_token) = load_refresh_token(&account.uuid) else {
            mark_reauth_required(&account.uuid)?;
            continue;
        };

        let mut auth =
            MicrosoftAuth::new(config::AZURE_CLIENT_ID).with_keyring(config::LAUNCHER_NAME);
        match auth.authenticate_with_refresh_token(&refresh_token, None).await {
            Ok(profile) => {
                persist_refreshed_profile(&account.uuid, &profile)?;
                log::info!(
                    target: "rslauncher",
                    "[auth] refreshed Microsoft session for {}",
                    account.username
                );
            }
            Err(AuthError::InvalidToken) => {
                mark_reauth_required(&account.uuid)?;
                log::warn!(
                    target: "rslauncher",
                    "[auth] session expired for {}; reconnect required",
                    account.username
                );
            }
            Err(err) => {
                // Do not invalidate a reusable session for a transient network
                // or Xbox/Minecraft service failure.
                log::warn!(
                    target: "rslauncher",
                    "[auth] startup refresh failed for {}: {err}",
                    account.username
                );
            }
        }
    }

    Ok(())
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
    let mut file = load_file()?;
    let previous = file.active_uuid.clone();
    normalize_active_account(&mut file);
    if file.active_uuid != previous {
        save_file(&file)?;
    }
    Ok(file.active_uuid)
}

#[cfg(test)]
mod tests {
    use super::{normalize_active_account, to_dto, AccountsFile, StoredAccount};
    use crate::config;

    #[test]
    fn legacy_account_requires_reauthentication() {
        let account: StoredAccount = serde_json::from_str(
            r#"{"uuid":"1234","username":"Steve"}"#,
        )
        .unwrap();

        assert!(to_dto(&account).needs_reauth);
    }

    #[test]
    fn current_client_account_remains_connected() {
        let account = StoredAccount {
            uuid: "1234".into(),
            username: "Alex".into(),
            auth_client_id: Some(config::AZURE_CLIENT_ID.into()),
        };

        assert!(!to_dto(&account).needs_reauth);
    }

    #[test]
    fn invalid_active_account_falls_back_to_a_valid_one() {
        let mut file = AccountsFile {
            accounts: vec![
                StoredAccount {
                    uuid: "old".into(),
                    username: "Old".into(),
                    auth_client_id: None,
                },
                StoredAccount {
                    uuid: "valid".into(),
                    username: "Valid".into(),
                    auth_client_id: Some(config::AZURE_CLIENT_ID.into()),
                },
            ],
            active_uuid: Some("old".into()),
        };

        normalize_active_account(&mut file);

        assert_eq!(file.active_uuid.as_deref(), Some("valid"));
    }
}
