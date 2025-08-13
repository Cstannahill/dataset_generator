// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod commands;
pub mod dataset;
pub mod dataset_concurrent;
pub mod models;
pub mod state;
pub mod types;
pub mod quality_validator;
pub mod embedding_service;
pub mod vector_db;
pub mod knowledge_base;
pub mod prompt_template;
pub mod enhanced_validation;
pub mod quality_visualization;
pub mod enhanced_commands;
pub mod chromadb_server;

use crate::commands::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state::AppState::new())
                .invoke_handler(tauri::generate_handler![
            commands::discover_models,
            commands::start_generation,
            commands::cancel_generation,
            commands::get_progress,
            commands::export_dataset,
            commands::debug_dataset_state,
            commands::improve_prompt,
            commands::generate_use_case_suggestions,
            commands::initialize_knowledge_base,
            commands::get_knowledge_base_stats,
            commands::search_knowledge_base,
            commands::get_improvement_suggestions,
            commands::list_collections,
            commands::generate_prompt_improvements
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}