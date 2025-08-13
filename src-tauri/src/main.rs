// Prevent console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
mod models;
mod dataset;
mod dataset_concurrent;
mod state;
mod commands;
mod quality_validator;
mod embedding_service;
mod vector_db;
mod knowledge_base;
mod prompt_template;
mod chromadb_server;

use state::AppState;
use tauri::Manager;
use commands::{discover_models, start_generation, cancel_generation, get_progress, export_dataset, debug_dataset_state, improve_prompt, generate_use_case_suggestions, start_chromadb_server, stop_chromadb_server, get_chromadb_server_status, check_chromadb_available};

async fn setup_chromadb(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let state = app_handle.state::<AppState>();
    let server = &state.chromadb_server;
    
    println!("Checking ChromaDB availability...");
    match server.check_chromadb_available() {
        Ok(()) => {
            println!("ChromaDB found, starting server on port 8465...");
            match server.start_server().await {
                Ok(()) => {
                    println!("ChromaDB server started successfully");
                }
                Err(e) => {
                    eprintln!("Warning: Failed to start ChromaDB server: {}", e);
                    eprintln!("You can start it manually later or install ChromaDB with: pip install chromadb");
                }
            }
        }
        Err(_) => {
            eprintln!("ChromaDB not found. Please install it with: pip install chromadb");
            eprintln!("The application will continue without ChromaDB support.");
        }
    }
    
    Ok(())
}

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
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_chromadb(handle).await {
                    eprintln!("ChromaDB setup error: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            discover_models,
            start_generation,
            cancel_generation,
            get_progress,
            export_dataset,
            debug_dataset_state,
            improve_prompt,
            generate_use_case_suggestions,
            start_chromadb_server,
            stop_chromadb_server,
            get_chromadb_server_status,
            check_chromadb_available
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}