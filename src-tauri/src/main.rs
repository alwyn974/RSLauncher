// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::default::Default;
use std::sync::Mutex;
use rand::RngCore;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task ({}): '{}' - Completed: {}", self.id, self.description, self.completed)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AppState {
    tasks: Mutex<Vec<Task>>
}

#[tauri::command]
fn add_task(desc: String, state: tauri::State<'_, AppState>) -> String {
    let mut rng = rand::thread_rng();
    let task = Task {
        id: rng.next_u32(),
        description: desc,
        completed: false,
    };

    let mut tasks = state.tasks.lock().unwrap();
    tasks.push(task);
    return tasks.last().unwrap().to_string()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, add_task])
        .manage(AppState { tasks: Default::default() })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
