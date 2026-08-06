mod accounts;
mod catalog;
mod commands;
mod config;
mod dto;
mod error;
mod events;
mod memory;
mod modpack;
mod modpack_meta;
mod modpack_profile;
mod mods_cleanup;
mod optional_content;
mod servers;
mod settings;
mod state;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use lighty_launcher::mods::curseforge;
use lighty_launcher::prelude::*;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, DEFAULT_FILENAME};

use crate::state::LaunchState;

/// Debounce disk writes while the user drags/resizes (HMR kills often skip `Exit`).
const WINDOW_STATE_SAVE_DEBOUNCE: Duration = Duration::from_millis(400);

fn persist_window_state(app: &tauri::AppHandle) {
    if let Err(err) = app.save_window_state(StateFlags::all()) {
        log::warn!(target: "rslauncher", "[launcher] save window state: {err}");
    }
}

/// True when window-state has a real saved size for `main` (mirrors the plugin's
/// "skip WindowState::default()" restore filter).
fn has_persisted_window_state(app: &tauri::AppHandle) -> bool {
    let Ok(dir) = app.path().app_config_dir() else {
        return false;
    };
    let Ok(raw) = std::fs::read_to_string(dir.join(DEFAULT_FILENAME)) else {
        return false;
    };
    let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw) else {
        return false;
    };
    let Some(main) = map.get("main") else {
        return false;
    };
    let width = main.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
    let height = main.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
    width > 0 && height > 0
}

/// `center: true` in tauri.conf fights restore on every launch — only center once.
fn center_main_window_on_first_launch(app: &tauri::App) {
    if has_persisted_window_state(app.handle()) {
        return;
    }
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    if let Err(err) = win.center() {
        log::warn!(target: "rslauncher", "[launcher] center window on first launch: {err}");
    }
}

fn install_window_state_autosave(app: &tauri::App) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let handle = app.handle().clone();
    let generation = Arc::new(AtomicU64::new(0));

    win.on_window_event(move |event| {
        match event {
            WindowEvent::Moved(_) | WindowEvent::Resized(_) => {}
            _ => return,
        }
        let gen = generation.fetch_add(1, Ordering::SeqCst) + 1;
        let handle = handle.clone();
        let generation = Arc::clone(&generation);
        std::thread::spawn(move || {
            std::thread::sleep(WINDOW_STATE_SAVE_DEBOUNCE);
            if generation.load(Ordering::SeqCst) == gen {
                persist_window_state(&handle);
            }
        });
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    AppState::init(config::LAUNCHER_NAME).expect("failed to init Lighty AppState");
    modpack_profile::init();

    // After install / NeoForge processors, before JVM spawn: reconcile overrides
    // + orphan jars + optional toggles. Must not run on the event bridge (that
    // raced the launcher and left the UI on "Downloading" while the game started).
    lighty_launcher::_launch::set_pre_launch_hook(|| async {
        let Ok(settings) = crate::settings::load() else {
            return;
        };
        let instance = crate::modpack::build_instance_with(&settings);
        if let Err(err) = crate::optional_content::sync(instance.game_dirs(), &settings).await {
            log::warn!(
                target: "rslauncher",
                "[launcher] pre-launch sync failed: {err}"
            );
        }
    });

    // Flaky connections: fewer parallel streams, more retries, longer backoff.
    init_downloader_config(DownloaderConfig {
        max_concurrent_downloads: 16,
        max_retries: 8,
        initial_delay_ms: 250,
    });

    if let Some(key) = config::curseforge_api_key() {
        curseforge::set_api_key(key);
    }

    let launch_state = Arc::new(LaunchState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("rslauncher".into()),
                    }),
                    Target::new(TargetKind::Webview),
                ])
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .level_for("hyper", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("reqwest", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tao", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("wry", tauri_plugin_log::log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // window-state restores on window-ready; we only add debounced saves
            // so HMR / kill-restarts keep size + monitor. Center only when there
            // is no saved state yet (keeps `center: false` in tauri.conf).
            center_main_window_on_first_launch(app);
            install_window_state_autosave(app);
            Ok(())
        })
        .manage(launch_state)
        .invoke_handler(tauri::generate_handler![
            commands::auth::login_with_microsoft,
            commands::auth::list_accounts,
            commands::auth::remove_account,
            commands::auth::set_active_account,
            commands::auth::get_active_account,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_catalog,
            commands::settings::list_modpacks,
            commands::settings::set_active_modpack,
            commands::settings::get_active_modpack,
            commands::settings::get_memory_info,
            commands::launch::get_instance_status,
            commands::launch::play,
            commands::launch::cancel,
            commands::launch::stop,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                persist_window_state(app);
                // Drop our PID tracking without killing the JVM - Minecraft is
                // intentionally detached (see lighty-java patch).
                if let Some(state) = app.try_state::<Arc<LaunchState>>() {
                    if let Some(pid) = state.take_pid() {
                        log::info!(
                            target: "rslauncher",
                            "[launcher] exiting - Minecraft keeps running (pid {pid})"
                        );
                    }
                }
            }
        });
}
