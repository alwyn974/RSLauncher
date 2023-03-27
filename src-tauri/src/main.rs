// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::default::Default;
use std::sync::Mutex;
use rand::RngCore;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

/*
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
}*/

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Eq, PartialEq)]
struct Subtask {
    description: String,
    completed: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Eq, PartialEq)]
struct Todo {
    title: String,
    description: String,
    subtasks: HashMap<u32, Subtask>,
    completed: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Eq, PartialEq)]
struct TodoList {
    todos: HashMap<u32, Todo>,
    id: u32
}

impl TodoList {
    fn new() -> Self {
        TodoList {
            todos: HashMap::new(),
            id: 0
        }
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, todo) in self.todos.iter() {
            write!(f, "Todo ({}): '{}' - {}\n", id, todo.title, todo.description)?;
            for (id, subtask) in todo.subtasks.iter() {
                write!(f, "-> Subtask ({}): '{}' - Completed: {}\n", id, subtask.description, subtask.completed)?;
            }
        }
        Ok(())
    }

    fn add_todo(&mut self, title: String, description: String, completed: bool) -> u32 {
        let id = self.id;
        let todo = Todo {
            title,
            description,
            subtasks: HashMap::new(),
            completed,
        };
        self.todos.insert(id, todo);
        self.id += 1;
        self.write_to_file().unwrap();
        return id
    }

    fn add_subtask(&mut self, todo_id: u32, description: String) -> u32 {
        let id = self.todos.get(&todo_id).unwrap().subtasks.len() as u32;
        let subtask = Subtask {
            description,
            completed: false,
        };
        self.todos.get_mut(&todo_id).unwrap().subtasks.insert(id, subtask);
        self.write_to_file().unwrap();
        return id
    }

    /*fn complete_subtask(&mut self, todo_id: u32, subtask_id: u32) -> Result<(), &'static str> {
        return match self.todos.get_mut(&todo_id).unwrap().subtasks.get_mut(&subtask_id) {
            Some(subtask) => {
                subtask.completed = true;
                self.write_to_file().unwrap();
                Ok(())
            },
            None => Err("Subtask not found"),
        }
    }

    fn complete_todo(&mut self, todo_id: u32) -> Result<(), &'static str> {
        return match self.todos.get_mut(&todo_id) {
            Some(todo) => {
                todo.completed = true;
                self.write_to_file().unwrap();
                Ok(())
            },
            None => Err("Todo not found"),
        }
    }*/

    fn edit_todo(&mut self, todo_id: u32, title: String, description: String, completed: bool) -> Result<bool, &'static str> {
        return match self.todos.get_mut(&todo_id) {
            Some(todo) => {
                todo.title = title;
                todo.description = description;
                todo.completed = completed;
                self.write_to_file().unwrap();
                Ok(true)
            },
            None => Ok(false),
        }
    }

    fn edit_subtask(&mut self, todo_id: u32, subtask_id: u32, description: String) -> Result<(), &'static str> {
        return match self.todos.get_mut(&todo_id).unwrap().subtasks.get_mut(&subtask_id) {
            Some(subtask) => {
                subtask.description = description;
                self.write_to_file().unwrap();
                Ok(())
            },
            None => Err("Subtask not found"),
        }
    }

    fn delete_todo(&mut self, todo_id: u32) -> Result<bool, &'static str> {
        return match self.todos.remove(&todo_id) {
            Some(_) => {
                self.write_to_file().unwrap();
                Ok(true)
            },
            None => Ok(false),
        }
    }

    fn delete_subtask(&mut self, todo_id: u32, subtask_id: u32) -> Result<bool, &'static str> {
        return match self.todos.get_mut(&todo_id).unwrap().subtasks.remove(&subtask_id) {
            Some(_) => {
                self.write_to_file().unwrap();
                Ok(true)
            },
            None => Ok(false),
        }
    }

    fn get_todos(&self) -> Vec<Todo> {
        return self.todos.values().cloned().collect();
    }

    fn get_todo_by_id(&self, todo_id: u32) -> Result<Todo, &'static str> {
        return match self.todos.get(&todo_id) {
            Some(todo) => Ok(todo.clone()),
            None => Err("Todo not found"),
        }
    }

    fn get_subtasks(&self, todo_id: u32) -> Result<Vec<Subtask>, &'static str> {
        return match self.todos.get(&todo_id) {
            Some(todo) => Ok(todo.subtasks.values().cloned().collect()),
            None => Err("Todo not found"),
        }
    }

    fn get_subtask_by_id(&self, todo_id: u32, subtask_id: u32) -> Result<Subtask, &'static str> {
        return match self.todos.get(&todo_id) {
            Some(todo) => match todo.subtasks.get(&subtask_id) {
                Some(subtask) => Ok(subtask.clone()),
                None => Err("Subtask not found"),
            },
            None => Err("Todo not found"),
        }
    }

    fn write_to_file(&self) -> Result<(), &'static str> {
        let file = File::create("tasks.json").unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.todos).unwrap();
        return Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AppState {
    tasks: Mutex<TodoList>
}

impl AppState {
    fn new() -> Self {
        AppState {
            tasks: Mutex::new(TodoList::new())
        }
    }
}

#[tauri::command]
fn get_tasks(state: tauri::State<'_, AppState>) -> Vec<Todo> {
    return state.tasks.lock().unwrap().get_todos();
}

#[tauri::command]
fn add_task(title: &str, description: &str, completed: bool, state: tauri::State<'_, AppState>) -> u32 {
    return state.tasks.lock().unwrap().add_todo(title.to_string(), description.to_string(), completed)
}

#[tauri::command]
fn delete_task(id: u32, state: tauri::State<'_, AppState>) -> Result<bool, &'static str> {
    return state.tasks.lock().unwrap().delete_todo(id);
}

#[tauri::command]
fn edit_task(id: u32, title: &str, description: &str, completed: bool, state: tauri::State<'_, AppState>) -> Result<bool, &'static str> {
    return state.tasks.lock().unwrap().edit_todo(id, title.to_string(), description.to_string(), completed);
}

fn main() {
    let mut state = AppState::new();

    if std::fs::read_to_string("tasks.json").is_ok() {
        let tasks: HashMap<u32, Todo> = serde_json::from_str(&std::fs::read_to_string("tasks.json").unwrap()).unwrap();
        for task in tasks {
            state.tasks.lock().unwrap().todos.insert(task.0, task.1);
        }
        state.tasks.lock().unwrap().id = state.tasks.get_mut().unwrap().todos.keys().max().unwrap() + 1;
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            get_tasks,
            add_task,
            delete_task,
            edit_task
        ])
        .manage(state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
