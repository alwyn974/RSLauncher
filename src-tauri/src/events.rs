use std::sync::Arc;
use std::time::Instant;

use lighty_launcher::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::dto::Progress;
use crate::state::LaunchState;

fn emit_progress(app: &AppHandle, progress: Progress) {
    let _ = app.emit("launch://progress", progress);
}

/// Route through the `log` facade (tauri-plugin-log → stdout / file / webview → UI).
fn emit_log(level: &str, source: &str, message: impl Into<String>) {
    let message = message.into();
    match level {
        "ERROR" => log::error!(target: "rslauncher", "[{source}] {message}"),
        "WARN" => log::warn!(target: "rslauncher", "[{source}] {message}"),
        _ => log::info!(target: "rslauncher", "[{source}] {message}"),
    }
}

pub fn spawn_bridge(app: AppHandle, bus: EventBus, launch_state: Arc<LaunchState>) {
    let mut rx = bus.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut downloaded: u64 = 0;
        let mut total: u64 = 0;
        let mut last_tick = Instant::now();
        let mut bytes_window: u64 = 0;
        let mut java_downloaded: u64 = 0;
        let mut java_total: u64 = 0;

        while let Ok(event) = rx.next().await {
            match event {
                Event::Launch(LaunchEvent::IsInstalled { version }) => {
                    let msg = format!("{version} already installed");
                    emit_log("INFO", "launcher", &msg);
                    emit_progress(
                        &app,
                        Progress::detail("preparing", "Instance ready", msg, 32),
                    );
                }
                Event::Launch(LaunchEvent::InstallStarted { total_bytes, .. }) => {
                    total = total_bytes.max(1);
                    downloaded = 0;
                    bytes_window = 0;
                    last_tick = Instant::now();
                    let detail = format!("{} MB total", total_bytes / 1_000_000);
                    emit_log(
                        "INFO",
                        "launcher",
                        format!("Installing — {detail}"),
                    );
                    emit_progress(
                        &app,
                        Progress {
                            stage: "downloading".into(),
                            step: "Downloading game files".into(),
                            file: detail,
                            files_done: 0,
                            files_total: 0,
                            percent: 35,
                            bytes_per_sec: 0.0,
                            eta_sec: 0,
                        },
                    );
                }
                Event::Launch(LaunchEvent::InstallProgress { bytes }) => {
                    downloaded = downloaded.saturating_add(bytes);
                    bytes_window = bytes_window.saturating_add(bytes);
                    let elapsed = last_tick.elapsed().as_secs_f64().max(0.25);
                    let bytes_per_sec = bytes_window as f64 / elapsed;
                    if elapsed >= 0.5 {
                        bytes_window = 0;
                        last_tick = Instant::now();
                    }
                    // Map install 0–100% into overall pipeline ~35–80%.
                    let install_pct =
                        ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0);
                    let percent = 35 + ((install_pct * 0.45) as u32).min(45);
                    let remaining = total.saturating_sub(downloaded);
                    let eta_sec = if bytes_per_sec > 1.0 {
                        (remaining as f64 / bytes_per_sec).ceil() as u32
                    } else {
                        0
                    };
                    let done_mb = downloaded / 1_000_000;
                    let total_mb = total / 1_000_000;
                    emit_progress(
                        &app,
                        Progress {
                            stage: "downloading".into(),
                            step: "Downloading game files".into(),
                            file: format!("{done_mb} / {total_mb} MB"),
                            files_done: 0,
                            files_total: 0,
                            percent,
                            bytes_per_sec,
                            eta_sec,
                        },
                    );
                }
                Event::Launch(LaunchEvent::InstallCompleted { .. }) => {
                    emit_progress(
                        &app,
                        Progress::detail(
                            "verifying",
                            "Verifying files",
                            "Install completed — checking integrity",
                            82,
                        ),
                    );
                    emit_log("INFO", "launcher", "Install completed");
                }
                Event::Launch(LaunchEvent::Launching { version }) => {
                    emit_progress(
                        &app,
                        Progress::detail(
                            "launching",
                            "Starting game",
                            format!("Launching {version}"),
                            94,
                        ),
                    );
                    emit_log("INFO", "launcher", format!("Launching {version}"));
                }
                Event::Launch(LaunchEvent::Launched { pid, version }) => {
                    launch_state.set_pid(Some(pid));
                    if launch_state.is_cancelled() {
                        let instance = crate::modpack::build_instance();
                        let _ = instance.close_instance(pid).await;
                        launch_state.set_pid(None);
                        emit_progress(&app, Progress::idle());
                        emit_log("WARN", "launcher", "Launch cancelled");
                        continue;
                    }
                    emit_progress(
                        &app,
                        Progress::detail(
                            "running",
                            "Game running",
                            format!("{version} · pid {pid}"),
                            100,
                        ),
                    );
                    emit_log(
                        "INFO",
                        "launcher",
                        format!("Game launched ({version}, pid {pid})"),
                    );
                }
                Event::Launch(LaunchEvent::NotLaunched { error, .. }) => {
                    launch_state.set_pid(None);
                    emit_progress(&app, Progress::error(error.clone()));
                    emit_log("ERROR", "launcher", error);
                }
                Event::Launch(LaunchEvent::ProcessOutput { line, stream, .. }) => {
                    let level = if stream == "stderr" { "WARN" } else { "INFO" };
                    emit_log(level, "game", line);
                }
                Event::Launch(LaunchEvent::ProcessExited { exit_code, .. }) => {
                    launch_state.set_pid(None);
                    emit_progress(&app, Progress::idle());
                    emit_log(
                        "INFO",
                        "launcher",
                        format!("Game exited (code {exit_code})"),
                    );
                }
                Event::Java(e) => {
                    handle_java(&app, e, &mut java_downloaded, &mut java_total);
                }
                Event::Loader(e) => handle_loader(&app, e),
                Event::Core(e) => handle_core(&app, e),
                Event::InstanceLaunched(e) => {
                    emit_log(
                        "INFO",
                        "launcher",
                        format!("Instance launched (pid {})", e.pid),
                    );
                }
                Event::InstanceExited(e) => {
                    launch_state.set_pid(None);
                    emit_progress(&app, Progress::idle());
                    emit_log(
                        "INFO",
                        "launcher",
                        format!(
                            "Instance '{}' exited (code {:?})",
                            e.instance_name, e.exit_code
                        ),
                    );
                }
                Event::InstanceDeleted(e) => {
                    emit_log(
                        "INFO",
                        "launcher",
                        format!("Instance deleted: {}", e.instance_name),
                    );
                }
                Event::ConsoleOutput(e) => {
                    let level = match e.stream {
                        ConsoleStream::Stderr => "WARN",
                        ConsoleStream::Stdout => "INFO",
                    };
                    emit_log(level, "game", e.line);
                }
                Event::Auth(AuthEvent::AuthenticationInProgress { step, .. }) => {
                    let _ = app.emit(
                        "auth://status",
                        crate::dto::AuthStatusPayload { step: step.clone() },
                    );
                    emit_log("INFO", "auth", &step);
                    emit_progress(
                        &app,
                        Progress::detail("preparing", "Authenticating", step, 10),
                    );
                }
                Event::Auth(AuthEvent::AuthenticationFailed { error, .. }) => {
                    emit_log("ERROR", "auth", error);
                }
                Event::Auth(AuthEvent::AuthenticationSuccess { username, .. }) => {
                    emit_log("INFO", "auth", format!("Signed in as {username}"));
                    emit_progress(
                        &app,
                        Progress::detail(
                            "preparing",
                            "Session ready",
                            format!("Signed in as {username}"),
                            14,
                        ),
                    );
                }
                _ => {}
            }
        }
    });
}

fn handle_java(
    app: &AppHandle,
    event: JavaEvent,
    java_downloaded: &mut u64,
    java_total: &mut u64,
) {
    match event {
        JavaEvent::JavaNotFound {
            distribution,
            version,
        } => {
            let detail = format!("{distribution} {version} missing — will download");
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Java runtime", detail, 16),
            );
        }
        JavaEvent::JavaAlreadyInstalled {
            distribution,
            version,
            ..
        } => {
            let detail = format!("{distribution} {version} ready");
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Java runtime", detail, 22),
            );
        }
        JavaEvent::JavaDownloadStarted {
            distribution,
            version,
            total_bytes,
        } => {
            *java_total = total_bytes.max(1);
            *java_downloaded = 0;
            let detail = format!(
                "Downloading {distribution} {version} ({} MB)",
                total_bytes / 1_000_000
            );
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Downloading Java", detail, 18),
            );
        }
        JavaEvent::JavaDownloadProgress { bytes } => {
            *java_downloaded = java_downloaded.saturating_add(bytes);
            let pct = ((*java_downloaded as f64 / *java_total as f64) * 100.0).clamp(0.0, 100.0);
            let overall = 16 + ((pct * 0.08) as u32).min(8);
            let detail = format!(
                "{} / {} MB",
                *java_downloaded / 1_000_000,
                *java_total / 1_000_000
            );
            emit_progress(
                app,
                Progress::detail("java", "Downloading Java", detail, overall),
            );
        }
        JavaEvent::JavaDownloadCompleted {
            distribution,
            version,
        } => {
            let detail = format!("{distribution} {version} download done");
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Java downloaded", detail, 24),
            );
        }
        JavaEvent::JavaExtractionStarted {
            distribution,
            version,
        } => {
            let detail = format!("Extracting {distribution} {version}…");
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Extracting Java", detail, 25),
            );
        }
        JavaEvent::JavaExtractionProgress {
            files_extracted,
            total_files,
        } => {
            let pct = if total_files > 0 {
                (files_extracted as f64 / total_files as f64) * 100.0
            } else {
                0.0
            };
            let overall = 25 + ((pct * 0.04) as u32).min(4);
            emit_progress(
                app,
                Progress {
                    stage: "java".into(),
                    step: "Extracting Java".into(),
                    file: format!("{files_extracted}/{total_files} files"),
                    files_done: files_extracted as u32,
                    files_total: total_files as u32,
                    percent: overall,
                    bytes_per_sec: 0.0,
                    eta_sec: 0,
                },
            );
        }
        JavaEvent::JavaExtractionCompleted {
            distribution,
            version,
            ..
        } => {
            let detail = format!("{distribution} {version} ready");
            emit_log("INFO", "java", &detail);
            emit_progress(
                app,
                Progress::detail("java", "Java ready", detail, 28),
            );
        }
    }
}

fn handle_loader(app: &AppHandle, event: LoaderEvent) {
    match event {
        LoaderEvent::FetchingData {
            loader,
            minecraft_version,
            loader_version,
        } => {
            let detail =
                format!("{loader} · MC {minecraft_version} · {loader_version}");
            emit_log(
                "INFO",
                "loader",
                format!("Fetching loader data — {detail}"),
            );
            emit_progress(
                app,
                Progress::detail("loader", "Fetching loader data", detail, 30),
            );
        }
        LoaderEvent::DataFetched { loader, .. } => {
            let detail = format!("{loader} metadata fetched");
            emit_log("INFO", "loader", &detail);
            emit_progress(
                app,
                Progress::detail("loader", "Loader data ready", detail, 33),
            );
        }
        LoaderEvent::ManifestNotFound {
            loader,
            error,
            ..
        } => {
            emit_log(
                "ERROR",
                "loader",
                format!("{loader} manifest missing: {error}"),
            );
            emit_progress(
                app,
                Progress::detail(
                    "loader",
                    "Loader manifest missing",
                    error,
                    30,
                ),
            );
        }
        LoaderEvent::ManifestCached { loader } => {
            let detail = format!("Using cached {loader} manifest");
            emit_log("INFO", "loader", &detail);
            emit_progress(
                app,
                Progress::detail("loader", "Loader cached", detail, 31),
            );
        }
        LoaderEvent::MergingLoaderData {
            base_loader,
            overlay_loader,
        } => {
            let detail = format!("Merging {overlay_loader} → {base_loader}");
            emit_log("INFO", "loader", &detail);
            emit_progress(
                app,
                Progress::detail("loader", "Merging loader data", detail, 34),
            );
        }
        LoaderEvent::DataMerged {
            base_loader,
            overlay_loader,
        } => {
            let detail = format!("{overlay_loader} + {base_loader} merged");
            emit_log("INFO", "loader", &detail);
            emit_progress(
                app,
                Progress::detail("loader", "Loader ready", detail, 36),
            );
        }
    }
}

fn handle_core(app: &AppHandle, event: CoreEvent) {
    match event {
        CoreEvent::ExtractionStarted {
            archive_type,
            file_count,
            ..
        } => {
            let detail = if file_count > 0 {
                format!("{archive_type} · {file_count} files")
            } else {
                format!("Extracting {archive_type}")
            };
            emit_log("INFO", "core", format!("Extracting — {detail}"));
            emit_progress(
                app,
                Progress::detail("verifying", "Extracting archives", detail, 84),
            );
        }
        CoreEvent::ExtractionProgress {
            files_extracted,
            total_files,
        } => {
            let pct = if total_files > 0 {
                (files_extracted as f64 / total_files as f64) * 100.0
            } else {
                0.0
            };
            let overall = 84 + ((pct * 0.08) as u32).min(8);
            emit_progress(
                app,
                Progress {
                    stage: "verifying".into(),
                    step: "Extracting archives".into(),
                    file: format!("{files_extracted}/{total_files} files"),
                    files_done: files_extracted as u32,
                    files_total: total_files as u32,
                    percent: overall,
                    bytes_per_sec: 0.0,
                    eta_sec: 0,
                },
            );
        }
        CoreEvent::ExtractionCompleted {
            archive_type,
            files_extracted,
        } => {
            let detail = format!("{archive_type} done ({files_extracted} files)");
            emit_log("INFO", "core", &detail);
            emit_progress(
                app,
                Progress::detail("verifying", "Extraction done", detail, 90),
            );
        }
    }
}
