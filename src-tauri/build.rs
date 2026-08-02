use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Bake secrets into the binary at compile time.
    // Source (first match wins):
    //   1. env var (CI / local export)
    //   2. gitignored local file under src-tauri/
    // Never commit these values.
    bake_secret(
        "CURSEFORGE_API_KEY",
        "curseforge_api_key",
        "RSLAUNCHER_CF_API_KEY",
    );
    bake_secret(
        "AZURE_CLIENT_ID",
        "azure_client_id",
        "RSLAUNCHER_AZURE_CLIENT_ID",
    );

    tauri_build::build();
}

fn bake_secret(env_name: &str, file_name: &str, rustc_env: &str) {
    let value = env::var(env_name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| read_local_secret_file(file_name))
        .unwrap_or_default();

    println!("cargo:rerun-if-env-changed={env_name}");
    println!("cargo:rerun-if-changed={file_name}");
    // Literal embed - do not print the value to logs.
    println!("cargo:rustc-env={rustc_env}={value}");
}

fn read_local_secret_file(file_name: &str) -> Option<String> {
    let path = Path::new(file_name);
    let raw = fs::read_to_string(path).ok()?;
    let value = raw.trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
