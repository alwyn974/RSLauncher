// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use std::collections::HashSet;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

#[derive(Serialize, Deserialize)]
struct AppState {
    tasks: HashSet<Task>,
}

#[tauri::command]
async fn add_task(mut state: State<'_, HashSet<Task>>, description: String) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let new_task = Task {
        id: rand::random(),
        description,
        completed: false,
    };
    state.insert(new_task.clone());
    Ok(())
}


#[tauri::command]
async fn get_tasks(state: State<'_, HashSet<Task>>) -> Vec<Task> {
    state.iter().cloned().collect()
}

#[tauri::command]
async fn complete_task(state: State<'_, HashSet<Task>>, id: u32) -> Result<(), String> {
    for task in state.tasks.iter_mut() {
        if task.id == id {
            task.completed = true;
            return Ok(());
        }
    }
    Err("Task not found".to_string())
}

#[tauri::command]
async fn remove_completed_tasks(state: State<'_, HashSet<Task>>) {
    state.tasks.retain(|task| !task.completed);
}

#[tauri::command]
async fn remove_task(state: State<'_, HashSet<Task>>, id: u32) -> Result<(), String> {
    state
        .tasks
        .retain(|task| task.id != id)
        .then(|| ())
        .ok_or_else(|| "Task not found".to_string())
}


fn main() {
    tauri::Builder::default()
        .setup(|app| async move {
            app.manage(AppState { tasks: HashSet::new() });
            Ok(())
        })
        // .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn submit_task(task: &str) -> String {
    format!("Task: '{}' added", task)
}