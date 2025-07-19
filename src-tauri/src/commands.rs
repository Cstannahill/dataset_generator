use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::types::{Model, GenerationConfig, GenerationProgress, GenerationTask, DatasetEntry};
use crate::state::AppState;
use crate::models::ModelManager;
use crate::dataset::DatasetGenerator;
use crate::dataset_concurrent::{ConcurrentDatasetGenerator, ConcurrentGenerationConfig, ProgressUpdate};

#[tauri::command]
pub async fn discover_models(state: State<'_, AppState>) -> Result<Vec<Model>, String> {
    let mut all_models = Vec::new();
    
    // Discover Ollama models
    match ModelManager::discover_ollama_models().await {
        Ok(mut ollama_models) => all_models.append(&mut ollama_models),
        Err(e) => println!("Warning: Could not discover Ollama models: {}", e),
    }
    
    // Add OpenAI models
    match ModelManager::get_openai_models().await {
        Ok(mut openai_models) => all_models.append(&mut openai_models),
        Err(e) => println!("Warning: Could not get OpenAI models: {}", e),
    }
    
    // Update state
    let mut models = state.models.write().await;
    *models = all_models.clone();
    
    Ok(all_models)
}

#[tauri::command]
pub async fn start_generation(
    config: GenerationConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Generate unique ID for this generation session
    let generation_id = Uuid::new_v4().to_string();
    
    // Update configuration
    let mut gen_config = state.generation_config.write().await;
    *gen_config = Some(config.clone());
    
    // Initialize progress with enhanced metrics
    let total_batches = (config.target_entries + config.batch_size - 1) / config.batch_size;
    let mut progress = state.progress.write().await;
    *progress = GenerationProgress {
        current_batch: 0,
        total_batches,
        entries_generated: 0,
        estimated_completion: "Starting...".to_string(),
        status: "running".to_string(),
        generation_id: Some(generation_id.clone()),
        concurrent_batches: 0,
        entries_per_second: 0.0,
        errors_count: 0,
        retries_count: 0,
    };
    
    // Create cancellation token for this generation
    let cancellation_token = CancellationToken::new();
    let mut active_generations = state.active_generations.write().await;
    active_generations.insert(generation_id.clone(), cancellation_token.clone());
    drop(active_generations);
    
    // Start optimized concurrent generation in background
    let state_clone = Arc::new(AppState {
        models: state.models.clone(),
        dataset: state.dataset.clone(),
        generation_config: state.generation_config.clone(),
        progress: state.progress.clone(),
        active_generations: state.active_generations.clone(),
    });
    
    let state_for_error = state_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = run_concurrent_generation_process(state_clone, generation_id, cancellation_token).await {
            tracing::error!("Generation error: {}", e);
            
            // Update status to error
            let mut progress = state_for_error.progress.write().await;
            progress.status = format!("error: {}", e);
        }
    });
    
    Ok("Concurrent generation started".to_string())
}

#[tauri::command]
pub async fn cancel_generation(
    generation_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut active_generations = state.active_generations.write().await;
    
    if let Some(cancellation_token) = active_generations.remove(&generation_id) {
        cancellation_token.cancel();
        
        // Update progress status
        let mut progress = state.progress.write().await;
        if progress.generation_id.as_ref() == Some(&generation_id) {
            progress.status = "cancelled".to_string();
        }
        
        Ok("Generation cancelled successfully".to_string())
    } else {
        Err("Generation not found or already completed".to_string())
    }
}

#[tauri::command]
pub async fn get_progress(state: State<'_, AppState>) -> Result<GenerationProgress, String> {
    let progress = state.progress.read().await;
    Ok(progress.clone())
}

#[tauri::command]
pub async fn export_dataset(state: State<'_, AppState>) -> Result<String, String> {
    let dataset = state.dataset.read().await;
    tracing::info!("Export dataset called - dataset has {} entries", dataset.len());
    
    if dataset.is_empty() {
        tracing::warn!("Dataset is empty!");
        return Err("No dataset entries to export. Please generate a dataset first.".to_string());
    }
    
    let json_output = serde_json::to_string_pretty(&*dataset)
        .map_err(|e| {
            tracing::error!("Failed to serialize dataset: {}", e);
            format!("Failed to serialize dataset: {}", e)
        })?;
    
    tracing::info!("Successfully serialized dataset - {} bytes", json_output.len());
    Ok(json_output)
}

#[tauri::command]
pub async fn debug_dataset_state(state: State<'_, AppState>) -> Result<String, String> {
    let dataset = state.dataset.read().await;
    let progress = state.progress.read().await;
    
    let debug_info = format!(
        "Dataset entries: {}\nProgress status: {}\nEntries generated: {}\nFirst entry sample: {:?}",
        dataset.len(),
        progress.status,
        progress.entries_generated,
        dataset.first()
    );
    
    tracing::info!("Debug dataset state: {}", debug_info);
    Ok(debug_info)
}

async fn run_concurrent_generation_process(
    state: Arc<AppState>,
    generation_id: String,
    cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
    let config = {
        let config_guard = state.generation_config.read().await;
        config_guard.as_ref().unwrap().clone()
    };
    
    let models = state.models.read().await;
    let selected_model = models.iter()
        .find(|m| m.id == config.selected_model)
        .ok_or_else(|| anyhow::anyhow!("Selected model not found"))?
        .clone();
    drop(models);
    
    // Use different generation approaches based on provider
    let generation_result = match selected_model.provider {
        crate::types::ModelProvider::Ollama => {
            // Use simple sequential generation for Ollama (more reliable)
            tracing::info!("Using sequential generation for Ollama model");
            run_sequential_ollama_generation(state.clone(), config.clone(), selected_model.clone(), cancellation_token.clone()).await
        }
        crate::types::ModelProvider::OpenAI => {
            // Use concurrent generation for OpenAI (better performance)
            tracing::info!("Using concurrent generation for OpenAI model");
            
            // Prepare generation tasks for OpenAI
            let total_batches = (config.target_entries + config.batch_size - 1) / config.batch_size;
            let mut tasks = Vec::new();
            
            for batch_id in 0..total_batches {
                let remaining_entries = config.target_entries.saturating_sub(batch_id * config.batch_size);
                let entries_to_generate = remaining_entries.min(config.batch_size);
                
                let context = if batch_id == 0 {
                    "This is the first batch of the dataset.".to_string()
                } else {
                    format!("Previous batches completed: {}. Current progress: {}/{} total entries.", 
                           batch_id, batch_id * config.batch_size, config.target_entries)
                };
                
                tasks.push(GenerationTask {
                    id: uuid::Uuid::new_v4().to_string(),
                    batch_id,
                    entries_to_generate,
                    model_id: selected_model.id.clone(),
                    provider: selected_model.provider.clone(),
                    goal: config.fine_tuning_goal.clone(),
                    context,
                });
            }
            
            let generation_config = ConcurrentGenerationConfig {
                max_concurrent_batches: 6,
                max_concurrent_requests_per_batch: 4,
                ollama_requests_per_second: 15,
                openai_requests_per_second: 80,
                max_retries: 3,
                retry_delay: std::time::Duration::from_millis(500),
                request_timeout: std::time::Duration::from_secs(45),
            };
            
            let generator = ConcurrentDatasetGenerator::new(generation_config);
            run_concurrent_openai_generation(generator, tasks, state.clone(), config.clone(), cancellation_token.clone()).await
        }
    };
    
    match generation_result {
        Ok(all_entries) => {
            tracing::info!("Generation process returned {} entries", all_entries.len());
            
            // Update dataset in state
            {
                let mut dataset = state.dataset.write().await;
                *dataset = all_entries.clone();
                tracing::info!("Stored {} entries in dataset state", dataset.len());
            }
            
            // Final progress update
            {
                let mut progress = state.progress.write().await;
                progress.status = "completed".to_string();
                progress.estimated_completion = "Finished".to_string();
                progress.entries_generated = config.target_entries;
                let total_batches = (config.target_entries + config.batch_size - 1) / config.batch_size;
                progress.current_batch = total_batches;
            }
            
            // Clean up active generation
            {
                let mut active_generations = state.active_generations.write().await;
                active_generations.remove(&generation_id);
            }
            
            tracing::info!("Generation completed successfully with {} entries", config.target_entries);
            Ok(())
        }
        Err(e) => {
            // Update progress with error status
            {
                let mut progress = state.progress.write().await;
                progress.status = if cancellation_token.is_cancelled() {
                    "cancelled".to_string()
                } else {
                    format!("error: {}", e)
                };
            }
            
            // Clean up active generation
            {
                let mut active_generations = state.active_generations.write().await;
                active_generations.remove(&generation_id);
            }
            
            Err(e)
        }
    }
}

async fn run_sequential_ollama_generation(
    state: Arc<AppState>, 
    config: GenerationConfig, 
    selected_model: Model,
    cancellation_token: CancellationToken
) -> anyhow::Result<Vec<DatasetEntry>> {
    
    let mut all_entries = Vec::new();
    let total_batches = (config.target_entries + config.batch_size - 1) / config.batch_size;
    
    tracing::info!("Starting sequential Ollama generation: {} batches of {} entries", total_batches, config.batch_size);
    
    for batch_num in 0..total_batches {
        if cancellation_token.is_cancelled() {
            return Err(anyhow::anyhow!("Generation cancelled"));
        }
        
        let remaining_entries = config.target_entries - all_entries.len();
        let current_batch_size = remaining_entries.min(config.batch_size);
        
        tracing::info!("Processing batch {}/{} with {} entries", batch_num + 1, total_batches, current_batch_size);
        
        // Update progress
        {
            let mut progress = state.progress.write().await;
            progress.current_batch = batch_num + 1;
            progress.status = format!("Generating batch {}/{}", batch_num + 1, total_batches);
        }
        
        // Generate batch using original DatasetGenerator
        let batch_entries = DatasetGenerator::generate_batch(
            &selected_model.id,
            &selected_model.provider,
            &config.fine_tuning_goal,
            current_batch_size,
            &all_entries,
        ).await?;
        
        tracing::info!("Batch {} generated {} entries", batch_num + 1, batch_entries.len());
        all_entries.extend(batch_entries);
        
        // Update progress
        {
            let mut progress = state.progress.write().await;
            progress.entries_generated = all_entries.len();
        }
        
        // Small delay to prevent overwhelming Ollama
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    tracing::info!("Sequential Ollama generation completed with {} total entries", all_entries.len());
    Ok(all_entries)
}

async fn run_concurrent_openai_generation(
    generator: ConcurrentDatasetGenerator,
    tasks: Vec<GenerationTask>,
    state: Arc<AppState>,
    config: GenerationConfig,
    cancellation_token: CancellationToken
) -> anyhow::Result<Vec<DatasetEntry>> {
    // Set up progress channel
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressUpdate>();
    
    // Start progress monitoring task
    let state_for_progress = state.clone();
    let progress_handle = tokio::spawn(async move {
        let start_time = Instant::now();
        
        while let Some(update) = progress_rx.recv().await {
            let mut progress = state_for_progress.progress.write().await;
            
            // Update progress with enhanced metrics
            progress.entries_generated = update.entries_generated;
            progress.errors_count = update.errors_count;
            progress.retries_count = update.retries_count;
            progress.concurrent_batches = update.concurrent_batches;
            progress.entries_per_second = update.entries_per_second;
            
            if let Some(completed_batch) = update.batch_completed {
                progress.current_batch = completed_batch + 1;
            }
            
            // Calculate estimated completion
            let _elapsed = start_time.elapsed().as_secs_f64();
            if update.entries_per_second > 0.0 {
                let remaining_entries = config.target_entries.saturating_sub(update.entries_generated);
                let estimated_seconds = remaining_entries as f64 / update.entries_per_second;
                progress.estimated_completion = if estimated_seconds < 60.0 {
                    format!("{:.0} seconds", estimated_seconds)
                } else {
                    format!("{:.1} minutes", estimated_seconds / 60.0)
                };
            }
            
            // Update status
            if update.entries_generated >= config.target_entries {
                progress.status = "completed".to_string();
                progress.estimated_completion = "Finished".to_string();
                break;
            } else {
                progress.status = format!("Processing {} concurrent batches", update.concurrent_batches);
            }
        }
    });
    
    // Execute concurrent generation
    let generation_result = generator.generate_concurrent(
        tasks,
        cancellation_token.clone(),
        progress_tx,
    ).await;
    
    // Wait for progress monitoring to complete
    progress_handle.abort();
    
    generation_result
}