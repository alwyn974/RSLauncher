use std::sync::Arc;

use lighty_launcher::prelude::*;
use tauri::{AppHandle, Emitter, State};

use crate::accounts;
use crate::config;
use crate::dto::{AccountDto, DeviceCodePayload, Progress};
use crate::error::AppError;
use crate::state::LaunchState;

fn require_azure_client_id() -> Result<&'static str, AppError> {
    config::azure_client_id().ok_or_else(|| {
        AppError::msg(
            "This build has no Azure client ID. Rebuild with AZURE_CLIENT_ID set \
             (or src-tauri/azure_client_id).",
        )
    })
}

#[tauri::command]
pub async fn login_with_microsoft(
    app: AppHandle,
    state: State<'_, Arc<LaunchState>>,
) -> Result<AccountDto, AppError> {
    let client_id = require_azure_client_id()?;
    let mut auth = MicrosoftAuth::new(client_id).with_keyring(config::LAUNCHER_NAME);

    let app_cb = app.clone();
    auth.set_device_code_callback(move |code, url| {
        let _ = app_cb.emit(
            "auth://device_code",
            DeviceCodePayload {
                code: code.to_string(),
                url: url.to_string(),
            },
        );
    });

    let bus = EventBus::new(256);
    crate::events::spawn_bridge(app.clone(), bus.clone(), Arc::clone(&state));

    let profile = match auth.authenticate(Some(&bus)).await {
        Ok(profile) => profile,
        Err(e) => {
            let _ = app.emit("launch://progress", Progress::idle());
            return Err(AppError::msg(e.to_string()));
        }
    };

    // Ensure Play is idle after sign-in (auth must not leave launch progress busy).
    let _ = app.emit("launch://progress", Progress::idle());

    accounts::upsert_from_profile(&profile)
}

#[tauri::command]
pub async fn list_accounts() -> Result<Vec<AccountDto>, AppError> {
    accounts::list_accounts()
}

#[tauri::command]
pub async fn get_active_account() -> Result<Option<AccountDto>, AppError> {
    accounts::get_active_account()
}

#[tauri::command]
pub async fn set_active_account(id: String) -> Result<(), AppError> {
    accounts::set_active_account(&id)
}

#[tauri::command]
pub async fn remove_account(id: String) -> Result<(), AppError> {
    accounts::remove_account(&id)
}
