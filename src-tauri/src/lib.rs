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
use tauri_plugin_log::{Target, TargetKind};

use crate::state::LaunchState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    AppState::init(config::LAUNCHER_NAME).expect("failed to init Lighty AppState");

    if let Some(key) = config::curseforge_api_key() {
        curseforge::set_api_key(key);
    }

    let launch_state = Arc::new(LaunchState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
