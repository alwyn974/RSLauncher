use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use lighty_launcher::prelude::AppState;
use serde::{Deserialize, Serialize};
use sysinfo::Disks;
use tauri::{AppHandle, Emitter, State};

use crate::config;
use crate::dto::{StorageInfo, StorageLocationCheck, StorageMigrationProgress};
use crate::error::AppError;
use crate::state::LaunchState;

const BOOTSTRAP_FILE: &str = "storage.json";
const IGNORELIST_FILE: &str = "mods-ignorelist.txt";
const FREE_SPACE_RESERVE: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
struct StoragePaths {
    root: Option<PathBuf>,
    data_dir: PathBuf,
    config_dir: PathBuf,
    cache_dir: PathBuf,
    java_dir: PathBuf,
    bootstrap_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageBootstrap {
    storage_root: PathBuf,
}

#[derive(Debug, Clone)]
struct MoveEntry {
    source: PathBuf,
    destination: PathBuf,
}

static PATHS: OnceLock<StoragePaths> = OnceLock::new();

pub fn init() -> Result<(), AppError> {
    let name = config::LAUNCHER_NAME;
    let default_data = dirs::data_dir()
        .ok_or_else(|| AppError::msg("The system data directory is unavailable"))?
        .join(name);
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::msg("The system config directory is unavailable"))?
        .join(name);
    let default_cache = dirs::cache_dir()
        .ok_or_else(|| AppError::msg("The system cache directory is unavailable"))?
        .join(name);
    let bootstrap_path = config_dir.join(BOOTSTRAP_FILE);

    let root = read_bootstrap(&bootstrap_path);
    let (data_dir, cache_dir, java_dir) = match &root {
        Some(root) => (root.join("instances"), root.join("cache"), root.join("jre")),
        None => (default_data, default_cache, config_dir.join("jre")),
    };

    AppState::init_with_paths(name, &data_dir, &config_dir, &cache_dir)
        .map_err(|err| AppError::msg(err.to_string()))?;
    PATHS
        .set(StoragePaths {
            root,
            data_dir,
            config_dir,
            cache_dir,
            java_dir,
            bootstrap_path,
        })
        .map_err(|_| AppError::msg("Launcher storage is already initialized"))
}

fn paths() -> &'static StoragePaths {
    PATHS.get().expect("storage::init must run before use")
}

pub fn java_dir() -> &'static Path {
    &paths().java_dir
}

pub fn root_dir() -> &'static Path {
    paths().root.as_deref().unwrap_or(&paths().data_dir)
}

fn read_bootstrap(path: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(path).ok()?;
    match serde_json::from_str::<StorageBootstrap>(&raw) {
        Ok(value) if value.storage_root.is_absolute() => Some(value.storage_root),
        Ok(_) => {
            log::warn!(target: "rslauncher", "[storage] ignoring non-absolute storage root");
            None
        }
        Err(err) => {
            log::warn!(target: "rslauncher", "[storage] invalid storage.json: {err}");
            None
        }
    }
}

fn selected_root(selected: &Path) -> Result<PathBuf, AppError> {
    if !selected.is_absolute() {
        return Err(AppError::msg("Choose an absolute folder path"));
    }
    let already_named = selected
        .file_name()
        .map(|name| {
            name.to_string_lossy()
                .eq_ignore_ascii_case(config::LAUNCHER_NAME)
        })
        .unwrap_or(false);
    Ok(if already_named {
        selected.to_path_buf()
    } else {
        selected.join(config::LAUNCHER_NAME)
    })
}

fn move_entries(root: &Path) -> Vec<MoveEntry> {
    let mut entries = Vec::new();
    for id in crate::modpack_profile::ids() {
        if let Some(profile) = crate::modpack_profile::get_by_id(id) {
            entries.push(MoveEntry {
                source: paths().data_dir.join(&profile.instance_name),
                destination: root.join("instances").join(&profile.instance_name),
            });
        }
    }
    entries.push(MoveEntry {
        source: paths().data_dir.join(IGNORELIST_FILE),
        destination: root.join(IGNORELIST_FILE),
    });
    entries.push(MoveEntry {
        source: paths().java_dir.clone(),
        destination: root.join("jre"),
    });
    entries.push(MoveEntry {
        source: paths().cache_dir.clone(),
        destination: root.join("cache"),
    });
    entries
}

fn tree_size(path: &Path) -> Result<u64, AppError> {
    if !path.exists() {
        return Ok(0);
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut size = 0_u64;
    for entry in fs::read_dir(path)? {
        size = size.saturating_add(tree_size(&entry?.path())?);
    }
    Ok(size)
}

fn bytes_to_move(root: &Path) -> Result<u64, AppError> {
    move_entries(root).iter().try_fold(0_u64, |total, entry| {
        Ok(total.saturating_add(tree_size(&entry.source)?))
    })
}

fn free_space(path: &Path) -> u64 {
    Disks::new_with_refreshed_list()
        .list()
        .iter()
        .filter(|disk| path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .map(|disk| disk.available_space())
        .unwrap_or(0)
}

fn ensure_writable(root: &Path) -> Result<(), AppError> {
    fs::create_dir_all(root)?;
    let probe = root.join(format!(".rslauncher-write-test-{}", std::process::id()));
    fs::write(&probe, b"ok")
        .map_err(|err| AppError::msg(format!("The selected folder is not writable: {err}")))?;
    fs::remove_file(probe)?;
    Ok(())
}

fn is_non_empty(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    fs::read_dir(path)
        .ok()
        .and_then(|mut entries| entries.next())
        .is_some()
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    let Ok(left) = fs::canonicalize(left) else {
        return false;
    };
    let Ok(right) = fs::canonicalize(right) else {
        return false;
    };
    left.starts_with(&right) || right.starts_with(&left)
}

fn validate_destination(root: &Path) -> Result<StorageLocationCheck, AppError> {
    ensure_writable(root)?;

    if paths()
        .root
        .as_deref()
        .is_some_and(|current| paths_overlap(current, root))
    {
        return Err(AppError::msg("This is already the active storage folder"));
    }

    for managed in [
        root.join("instances"),
        root.join("cache"),
        root.join("jre"),
        root.join(IGNORELIST_FILE),
    ] {
        if is_non_empty(&managed) {
            return Err(AppError::msg(format!(
                "The destination already contains launcher data: {}",
                managed.display()
            )));
        }
    }

    for entry in move_entries(root) {
        if paths_overlap(&entry.source, root) {
            return Err(AppError::msg(
                "Choose a folder outside the current launcher data folders",
            ));
        }
    }

    let bytes = bytes_to_move(root)?;
    let free = free_space(root);
    let required = if bytes == 0 {
        0
    } else {
        bytes.saturating_add(FREE_SPACE_RESERVE)
    };
    if free < required {
        return Err(AppError::msg(format!(
            "Not enough free space ({} bytes required, {} available)",
            required, free
        )));
    }
    Ok(StorageLocationCheck {
        root: root.to_string_lossy().into_owned(),
        bytes_to_move: bytes,
        free_bytes: free,
        required_bytes: required,
    })
}

fn copy_tree(
    source: &Path,
    destination: &Path,
    copied: &mut u64,
    total: u64,
    app: &AppHandle,
) -> Result<(), AppError> {
    if !source.exists() {
        return Ok(());
    }
    if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
        *copied = copied.saturating_add(fs::metadata(source)?.len());
        emit_progress(app, "copying", source, *copied, total);
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        copy_tree(
            &entry.path(),
            &destination.join(entry.file_name()),
            copied,
            total,
            app,
        )?;
    }
    Ok(())
}

fn verify_tree(source: &Path, destination: &Path) -> Result<(), AppError> {
    if !source.exists() {
        return Ok(());
    }
    if source.is_file() {
        let source_len = fs::metadata(source)?.len();
        let destination_len = fs::metadata(destination)
            .map_err(|_| AppError::msg(format!("Missing copied file: {}", destination.display())))?
            .len();
        if source_len != destination_len {
            return Err(AppError::msg(format!(
                "Copied file verification failed: {}",
                destination.display()
            )));
        }
        return Ok(());
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        verify_tree(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn emit_progress(app: &AppHandle, stage: &str, path: &Path, copied: u64, total: u64) {
    let percent = copied
        .saturating_mul(100)
        .checked_div(total)
        .unwrap_or(100)
        .min(100) as u32;
    let _ = app.emit(
        "storage://progress",
        StorageMigrationProgress {
            stage: stage.into(),
            detail: path.to_string_lossy().into_owned(),
            bytes_copied: copied,
            bytes_total: total,
            percent,
        },
    );
}

fn write_bootstrap(root: &Path) -> Result<(), AppError> {
    fs::create_dir_all(&paths().config_dir)?;
    let payload = serde_json::to_vec_pretty(&StorageBootstrap {
        storage_root: root.to_path_buf(),
    })?;
    fs::write(&paths().bootstrap_path, payload)?;
    Ok(())
}

fn remove_source(entry: &MoveEntry) {
    let result = if entry.source.is_dir() {
        fs::remove_dir_all(&entry.source)
    } else if entry.source.exists() {
        fs::remove_file(&entry.source)
    } else {
        Ok(())
    };
    if let Err(err) = result {
        log::warn!(
            target: "rslauncher",
            "[storage] copied data but could not remove {}: {err}",
            entry.source.display()
        );
    }
}

#[tauri::command]
pub async fn get_storage_info() -> Result<StorageInfo, AppError> {
    let root = root_dir().to_path_buf();
    Ok(StorageInfo {
        root: root.to_string_lossy().into_owned(),
        data_dir: paths().data_dir.to_string_lossy().into_owned(),
        cache_dir: paths().cache_dir.to_string_lossy().into_owned(),
        java_dir: paths().java_dir.to_string_lossy().into_owned(),
        custom: paths().root.is_some(),
        installed_bytes: bytes_to_move(&root)?,
        free_bytes: free_space(&root),
    })
}

#[tauri::command]
pub async fn inspect_storage_location(path: String) -> Result<StorageLocationCheck, AppError> {
    let root = selected_root(Path::new(&path))?;
    validate_destination(&root)
}

#[tauri::command]
pub async fn migrate_storage(
    path: String,
    app: AppHandle,
    launch_state: State<'_, Arc<LaunchState>>,
) -> Result<StorageInfo, AppError> {
    if launch_state.is_running() || !launch_state.try_set_busy() {
        return Err(AppError::msg(
            "Close Minecraft and wait for the current launch before moving files",
        ));
    }
    let preparation = (|| {
        let root = selected_root(Path::new(&path))?;
        let check = validate_destination(&root)?;
        let entries = move_entries(&root);
        Ok::<_, AppError>((root, check, entries))
    })();
    let (root, check, entries) = match preparation {
        Ok(prepared) => prepared,
        Err(err) => {
            launch_state.set_busy(false);
            return Err(err);
        }
    };
    let app_for_task = app.clone();
    let root_for_task = root.clone();

    let migration_result = tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let mut copied = 0_u64;
        for entry in &entries {
            copy_tree(
                &entry.source,
                &entry.destination,
                &mut copied,
                check.bytes_to_move,
                &app_for_task,
            )?;
        }
        emit_progress(
            &app_for_task,
            "verifying",
            &root_for_task,
            copied,
            check.bytes_to_move,
        );
        for entry in &entries {
            verify_tree(&entry.source, &entry.destination)?;
        }
        write_bootstrap(&root_for_task)?;
        for entry in &entries {
            remove_source(entry);
        }
        emit_progress(
            &app_for_task,
            "done",
            &root_for_task,
            check.bytes_to_move,
            check.bytes_to_move,
        );
        Ok(())
    })
    .await
    .map_err(|err| AppError::msg(format!("Storage migration task failed: {err}")))
    .and_then(|result| result);

    if let Err(err) = migration_result {
        launch_state.set_busy(false);
        return Err(err);
    }
    // Keep launches blocked until the requested restart: this process still
    // holds Lighty's old immutable paths, whose files have just been moved.

    Ok(StorageInfo {
        root: root.to_string_lossy().into_owned(),
        data_dir: root.join("instances").to_string_lossy().into_owned(),
        cache_dir: root.join("cache").to_string_lossy().into_owned(),
        java_dir: root.join("jre").to_string_lossy().into_owned(),
        custom: true,
        installed_bytes: check.bytes_to_move,
        free_bytes: free_space(&root),
    })
}

#[cfg(test)]
mod tests {
    use super::selected_root;
    use std::path::Path;

    #[test]
    fn appends_launcher_folder_to_selected_parent() {
        let selected = if cfg!(windows) {
            Path::new(r"D:\\Games")
        } else {
            Path::new("/games")
        };
        assert_eq!(
            selected_root(selected).unwrap(),
            selected.join("RSLauncher")
        );
    }

    #[test]
    fn keeps_explicit_launcher_folder() {
        let selected = if cfg!(windows) {
            Path::new(r"D:\\Games\\rslauncher")
        } else {
            Path::new("/games/rslauncher")
        };
        assert_eq!(selected_root(selected).unwrap(), selected);
    }
}
