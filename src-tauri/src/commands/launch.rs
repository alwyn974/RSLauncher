use std::sync::Arc;

use lighty_launcher::mods::curseforge;
use lighty_launcher::prelude::*;
use tauri::{AppHandle, Emitter, State};

use crate::accounts;
use crate::config;
use crate::dto::{InstanceStatus, Progress};
use crate::error::AppError;
use crate::modpack;
use crate::servers;
use crate::settings;
use crate::state::LaunchState;

fn require_curseforge_key() -> Result<(), AppError> {
    let key = config::curseforge_api_key().ok_or_else(|| {
        AppError::msg(
            "This build has no CurseForge API key. Rebuild with CURSEFORGE_API_KEY set (or src-tauri/curseforge_api_key).",
        )
    })?;
    curseforge::set_api_key(key);
    Ok(())
}

fn emit_log(level: &str, source: &str, message: impl Into<String>) {
    let message = message.into();
    match level {
        "ERROR" => log::error!(target: "rslauncher", "[{source}] {message}"),
        "WARN" => log::warn!(target: "rslauncher", "[{source}] {message}"),
        _ => log::info!(target: "rslauncher", "[{source}] {message}"),
    }
}

fn emit_launch_error(app: &AppHandle, err: &AppError) {
    let message = err.to_string();
    let _ = app.emit("launch://progress", Progress::error(message.clone()));
    emit_log("ERROR", "launcher", format!("Launch failed: {message}"));
}

async fn resolve_profile() -> Result<UserProfile, AppError> {
    let uuid = accounts::active_uuid()?
        .ok_or_else(|| AppError::msg("No active account - sign in first"))?;

    let mut auth = MicrosoftAuth::new(config::AZURE_CLIENT_ID).with_keyring(config::LAUNCHER_NAME);

    if let Some(rt) = accounts::load_refresh_token(&uuid) {
        match auth.authenticate_with_refresh_token(&rt, None).await {
            Ok(profile) => {
                if let AuthProvider::Microsoft {
                    refresh_token: Some(new_rt),
                    ..
                } = &profile.provider
                {
                    let _ = accounts::save_refresh_token(&profile.uuid, new_rt);
                }
                return Ok(profile);
            }
            Err(err) => {
                emit_log(
                    "WARN",
                    "auth",
                    format!("Silent refresh failed ({err}), re-login required"),
                );
            }
        }
    }

    Err(AppError::msg(
        "Session expired - sign in with Microsoft again",
    ))
}

#[tauri::command]
pub async fn get_instance_status() -> InstanceStatus {
    InstanceStatus::current()
}

#[tauri::command]
pub async fn play(
    app: AppHandle,
    state: State<'_, Arc<LaunchState>>,
    quick_play: bool,
) -> Result<(), AppError> {
    if state.is_busy() {
        return Err(AppError::msg("A launch is already in progress"));
    }

    require_curseforge_key()?;
    state.reset_for_play();
    state.set_busy(true);

    let _ = app.emit(
        "launch://progress",
        Progress::detail(
            "preparing",
            "Preparing launch",
            if quick_play {
                "Quick Play - refreshing session…"
            } else {
                "Refreshing Microsoft session…"
            },
            4,
        ),
    );
    emit_log(
        "INFO",
        "launcher",
        if quick_play {
            "Starting Quick Play…"
        } else {
            "Starting Play…"
        },
    );

    let launch_state = Arc::clone(&state);
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let result = run_launch(app_handle.clone(), launch_state.clone(), quick_play).await;
        launch_state.set_busy(false);
        if let Err(err) = result {
            if launch_state.is_cancelled() {
                emit_log("WARN", "launcher", "Launch cancelled");
                let _ = app_handle.emit(
                    "launch://progress",
                    Progress::detail(
                        "error",
                        "Cancelled",
                        "Launch cancelled",
                        0,
                    ),
                );
            } else {
                emit_launch_error(&app_handle, &err);
            }
        } else {
            let _ = app_handle.emit("instance://status", InstanceStatus::current());
        }
    });

    Ok(())
}

async fn run_launch(
    app: AppHandle,
    launch_state: Arc<LaunchState>,
    quick_play: bool,
) -> Result<(), AppError> {
    let _ = app.emit(
        "launch://progress",
        Progress::detail(
            "preparing",
            "Refreshing session",
            "Validating Microsoft account…",
            6,
        ),
    );
    let profile = resolve_profile().await?;
    let _ = app.emit(
        "launch://progress",
        Progress::detail(
            "preparing",
            "Session ready",
            format!("Playing as {}", profile.username),
            12,
        ),
    );

    let settings = settings::load()?;
    let server_address = settings.server_address.trim().to_string();

    if quick_play && server_address.is_empty() {
        return Err(AppError::msg(
            "Quick Play needs a server address - set one in Settings",
        ));
    }

    let bus = EventBus::new(1000);
    crate::events::spawn_bridge(app.clone(), bus.clone(), launch_state.clone());

    let _ = app.emit(
        "launch://progress",
        Progress::detail(
            "preparing",
            "Building instance",
            {
                let p = crate::modpack_profile::get();
                format!("{} · {}", p.instance_name, p.loader_display())
            },
            15,
        ),
    );

    let mut instance = modpack::build_instance_with(&settings);
    // game_dirs() is the vanilla run directory (servers.dat lives here).
    servers::ensure_default_server(
        instance.game_dirs(),
        &settings.server_name,
        &server_address,
    )?;

    emit_log(
        "INFO",
        "launcher",
        {
            let p = crate::modpack_profile::get();
            format!(
                "Launching {} ({}) — install will run if needed",
                p.instance_name,
                p.loader_display()
            )
        },
    );
    let _ = app.emit(
        "launch://progress",
        Progress::detail(
            "preparing",
            "Starting install pipeline",
            "Java → loader → assets → launch",
            16,
        ),
    );

    let xms = settings.ram_gb.min(2);
    let mut jvm = instance
        .launch(&profile, JavaDistribution::Temurin)
        .with_event_bus(&bus)
        .with_jvm_options()
        .set("Xmx", format!("{}G", settings.ram_gb))
        .set("Xms", format!("{xms}G"));

    for token in settings.jvm_args.split_whitespace() {
        let trimmed = token.trim_start_matches('-');
        if let Some((key, value)) = trimmed.split_once('=') {
            jvm = jvm.set(key, value);
        } else if trimmed.starts_with("Xmx") || trimmed.starts_with("Xms") {
            continue;
        } else {
            jvm = jvm.set(trimmed, "");
        }
    }

    let mut args = jvm
        .done()
        .with_arguments()
        .set("width", settings.width.to_string())
        .set("height", settings.height.to_string());

    if settings.fullscreen {
        args = args.set("fullscreen", "");
    }

    if quick_play {
        emit_log(
            "INFO",
            "launcher",
            format!("Quick Play → {server_address}"),
        );
        args = args.set("quickPlayMultiplayer", &server_address);
    }

    args.done()
        .run()
        .await
        .map_err(|e| AppError::msg(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn cancel(state: State<'_, Arc<LaunchState>>) -> Result<(), AppError> {
    state.request_cancel();
    if let Some(pid) = state.take_pid() {
        let instance = modpack::build_instance();
        instance
            .close_instance(pid)
            .await
            .map_err(|e| AppError::msg(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn stop(state: State<'_, Arc<LaunchState>>) -> Result<(), AppError> {
    if let Some(pid) = state.take_pid() {
        let instance = modpack::build_instance();
        instance
            .close_instance(pid)
            .await
            .map_err(|e| AppError::msg(e.to_string()))?;
    }
    Ok(())
}
