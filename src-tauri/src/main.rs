// Prevent console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
mod models;
mod dataset;
mod dataset_concurrent;
mod state;
mod commands;

use state::AppState;
use commands::{discover_models, start_generation, cancel_generation, get_progress, export_dataset, debug_dataset_state};

fn main() {
    // Load environment variables from .env file (if it exists)
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Could not load .env file: {}", e);
        println!("Note: You can create a .env file in the project root with your API keys");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            discover_models,
            start_generation,
            cancel_generation,
            get_progress,
            export_dataset,
            debug_dataset_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}