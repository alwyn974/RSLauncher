mod accounts;
mod commands;
mod config;
mod dto;
mod error;
mod events;
mod memory;
mod modpack;
mod modpack_meta;
mod servers;
mod settings;
mod state;

use std::sync::Arc;

use lighty_launcher::mods::curseforge;
use lighty_launcher::prelude::*;
use tauri::{LogicalSize, Manager, RunEvent};
use tauri_plugin_log::{Target, TargetKind};

use crate::state::LaunchState;

/// Minecraft’s classic default window — also `minWidth` / `minHeight` in tauri.conf.
const WINDOW_MIN_WIDTH: f64 = 854.0;
const WINDOW_MIN_HEIGHT: f64 = 480.0;
/// Must match `app.windows[0].width/height` in tauri.conf.json (used on bad restore).
const WINDOW_DEFAULT_WIDTH: f64 = 1024.0;
const WINDOW_DEFAULT_HEIGHT: f64 = 768.0;

/// After `window-state` restore, clamp absurd sizes (e.g. a near-fullscreen
/// physical size saved with `maximized: false`) back to the default.
fn clamp_main_window_size(app: &tauri::App) -> tauri::Result<()> {
    let Some(win) = app.get_webview_window("main") else {
        return Ok(());
    };
    if win.is_maximized()? || win.is_fullscreen()? {
        return Ok(());
    }

    let scale = win.scale_factor()?;
    let physical = win.inner_size()?;
    let logical_w = physical.width as f64 / scale;
    let logical_h = physical.height as f64 / scale;

    let monitor_logical = win.current_monitor()?.map(|m| {
        let s = m.size();
        (s.width as f64 / scale, s.height as f64 / scale)
    });

    let mut width = logical_w.max(WINDOW_MIN_WIDTH);
    let mut height = logical_h.max(WINDOW_MIN_HEIGHT);
    let mut reset = false;

    if let Some((mw, mh)) = monitor_logical {
        // Physical size larger than the monitor → bad restore (common HiDPI glitch).
        if width > mw || height > mh {
            width = WINDOW_DEFAULT_WIDTH;
            height = WINDOW_DEFAULT_HEIGHT;
            reset = true;
        }
    }

    if reset || (width - logical_w).abs() > 0.5 || (height - logical_h).abs() > 0.5 {
        win.set_size(LogicalSize::new(width, height))?;
        if reset {
            log::warn!(
                target: "rslauncher",
                "[launcher] reset window size from {logical_w:.0}×{logical_h:.0} → {width:.0}×{height:.0}"
            );
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    AppState::init(config::LAUNCHER_NAME).expect("failed to init Lighty AppState");

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
        .plugin(tauri_plugin_positioner::init())
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
            if let Err(err) = clamp_main_window_size(app) {
                log::warn!(target: "rslauncher", "[launcher] window size clamp: {err}");
            }
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
                // Drop our PID tracking without killing the JVM — Minecraft is
                // intentionally detached (see lighty-java patch).
                if let Some(state) = app.try_state::<Arc<LaunchState>>() {
                    if let Some(pid) = state.take_pid() {
                        log::info!(
                            target: "rslauncher",
                            "[launcher] exiting — Minecraft keeps running (pid {pid})"
                        );
                    }
                }
            }
        });
}
