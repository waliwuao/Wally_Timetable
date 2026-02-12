#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod data;

use std::path::PathBuf;
use std::env;
use tauri::State;

struct AppState {
    theme_path: PathBuf,
    data_path: PathBuf,
}

#[tauri::command]
fn get_theme(state: State<AppState>) -> config::Theme {
    config::load_theme(&state.theme_path)
}

#[tauri::command]
fn get_schedule(state: State<AppState>) -> Result<data::TimetableData, String> {
    data::TimetableData::load(&state.data_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_cell(row: usize, col: usize, value: String, state: State<AppState>) -> Result<(), String> {
    data::TimetableData::save(&state.data_path, row, col, value).map_err(|e| e.to_string())
}

fn main() {
    let base_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    let (theme_path, data_path) = if base_path.ends_with("src-tauri") {
        (base_path.join("../assets/styles/theme.conf"), base_path.join("../data/schedule.csv"))
    } else {
        (base_path.join("assets/styles/theme.conf"), base_path.join("data/schedule.csv"))
    };

    tauri::Builder::default()
        .manage(AppState { theme_path, data_path })
        .invoke_handler(tauri::generate_handler![get_theme, get_schedule, save_cell])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}