/// Initialize the knowledge base system
#[tauri::command]
pub async fn initialize_knowledge_base(state: State<'_, AppState>) -> Result<String, String> {
    let kb_manager_lock = state.knowledge_base_manager.read().await;
    if let Some(ref kb_manager) = *kb_manager_lock {
        match kb_manager.initialize().await {
            Ok(()) => Ok("Knowledge base initialized successfully".to_string()),
            Err(e) => Err(format!("Failed to initialize knowledge base: {}", e)),
        }
    } else {
        Err("Knowledge base manager not configured".to_string())
    }
}
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
use crate::knowledge_base::{KnowledgeBaseManager, KnowledgeBaseConfig, KnowledgeBaseStats, ImprovementSuggestion};
use crate::vector_db::{CollectionInfo, SearchResult, QueryRequest};

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
        knowledge_base_manager: state.knowledge_base_manager.clone(),
        chromadb_server: state.chromadb_server.clone(),
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
    let config = state.generation_config.read().await;
    let format = config.as_ref().map(|c| &c.format);
    
    tracing::info!("Export dataset called - dataset has {} entries", dataset.len());
    if dataset.is_empty() {
        tracing::warn!("Dataset is empty!");
        return Err("No dataset entries to export. Please generate a dataset first.".to_string());
    }

    // Process through knowledge base if available
    let kb_state = state.knowledge_base_manager.read().await;
    if let Some(kb_manager) = kb_state.as_ref() {
        if let (Some(config_ref), Some(format_ref)) = (config.as_ref(), format) {
            tracing::info!("Processing entries through knowledge base pipeline...");
            
            let entries_clone: Vec<DatasetEntry> = dataset.clone();
            match kb_manager.process_entries(
                entries_clone,
                &config_ref.fine_tuning_goal,
                format_ref,
            ).await {
                Ok(stats) => {
                    tracing::info!("Knowledge base processing completed: {:?}", stats);
                }
                Err(e) => {
                    tracing::warn!("Knowledge base processing failed: {}", e);
                    // Continue with export even if knowledge base processing fails
                }
            }
        }
    }

    // Validate entries for format
    let filtered: Vec<&DatasetEntry> = dataset.iter().filter(|entry| {
        match format {
            Some(crate::types::DatasetFormat::RetrievalEmbedding) => {
                let obj = &entry.data;
                obj.get("query").is_some() && obj.get("positive_passage").is_some() && obj.get("negative_passages").is_some()
            }
            // ...add other format checks as needed...
            _ => true
        }
    }).collect();
    
    if filtered.len() != dataset.len() {
        tracing::warn!("Some entries did not match the selected format and were excluded");
    }
    
    // Deduplicate entries
    let mut seen = std::collections::HashSet::new();
    let deduped: Vec<&DatasetEntry> = filtered.into_iter().filter(|entry| {
        let s = format!("{:?}", entry.data);
        seen.insert(s)
    }).collect();
    
    // Generate JSONL format - one JSON object per line
    let mut jsonl_lines = Vec::new();
    for entry in deduped.iter() {
        let json_line = serde_json::to_string(entry)
            .map_err(|e| {
                tracing::error!("Failed to serialize dataset entry: {}", e);
                format!("Failed to serialize dataset entry: {}", e)
            })?;
        jsonl_lines.push(json_line);
    }
    let jsonl_output = jsonl_lines.join("\n");
    tracing::info!("Successfully serialized dataset as JSONL - {} bytes", jsonl_output.len());
    Ok(jsonl_output)
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

#[tauri::command]
pub async fn improve_prompt(prompt: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not found in environment. Please set it to use prompt improvement.".to_string())?;
    
    let system_prompt = "You are an expert at creating fine-tuning objectives for AI models. Your task is to improve and refine user-provided fine-tuning goals to make them more specific, structured, and effective for generating high-quality training datasets.

Focus on:
1. Clarity and specificity of the task
2. Clear definition of input and output formats
3. Specific examples of desired behavior
4. Context about the target domain or use case
5. Quality criteria and constraints

Return ONLY the improved fine-tuning goal without any preamble, explanation, or additional text. The response should be the goal itself, ready to use directly in dataset generation.";

    let user_prompt = format!("Improve this fine-tuning goal to make it more structured, specific, and effective for dataset generation:\n\n{}", prompt);
    
    let request_body = serde_json::json!({
        "model": "gpt-4.1-nano",
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user", 
                "content": user_prompt
            }
        ],
        "max_tokens": 1000,
        "temperature": 0.7
    });
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call OpenAI API: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let improved_prompt = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Failed to extract improved prompt")
            .to_string();
        
        // Clean up the response to remove any preamble text
        let cleaned_prompt = clean_improved_prompt(&improved_prompt);
        
        Ok(cleaned_prompt)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("OpenAI API error: {}", error_text))
    }
}

fn clean_improved_prompt(prompt: &str) -> String {
    let cleaned = prompt.trim();
    
    // Remove common preamble phrases
    let prefixes_to_remove = [
        "Certainly, here is your improved",
        "Here is an improved version",
        "Here's the improved",
        "Improved fine-tuning goal:",
        "Here is the refined",
        "Certainly! Here is",
        "Sure, here is",
        "Here's a more",
        "I'll improve",
        "Let me improve"
    ];
    
    let mut result = cleaned.to_string();
    
    // Remove any prefix that matches our patterns (case insensitive)
    for prefix in &prefixes_to_remove {
        if result.to_lowercase().starts_with(&prefix.to_lowercase()) {
            // Find the end of the preamble sentence and remove it
            if let Some(pos) = result.find(':') {
                result = result[pos + 1..].trim().to_string();
            } else if let Some(pos) = result.find('.') {
                // Check if this looks like the end of a preamble sentence
                let potential_start = &result[pos + 1..].trim();
                if potential_start.len() > 20 {  // Likely the actual content
                    result = potential_start.to_string();
                }
            }
            break;
        }
    }
    
    // Remove any remaining leading quotes or extra whitespace
    result = result.trim_start_matches('"').trim_start_matches('\'').trim().to_string();
    result = result.trim_end_matches('"').trim_end_matches('\'').trim().to_string();
    
    // If the result is still too short or looks incomplete, return the original
    if result.len() < 20 {
        cleaned.to_string()
    } else {
        result
    }
}

#[tauri::command]
pub async fn generate_use_case_suggestions(
    domain_context: String,
    format: String,
    model_id: String,
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    tracing::info!("Generating use case suggestions for format: {} with domain: {}", format, domain_context);
    
    let models = state.models.read().await;
    let selected_model = models.iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| "Selected model not found".to_string())?;
    
    let domain_text = if domain_context.trim().is_empty() {
        "any domain".to_string()
    } else {
        domain_context.trim().to_string()
    };
    
    let format_description = match format.as_str() {
        "alpaca" => "instruction-following tasks with clear input-output pairs",
        "conversation" => "multi-turn dialogue and chat-based interactions",
        "chain_of_thought" => "step-by-step reasoning and problem-solving",
        "preference_ranking" => "response quality comparison and preference learning",
        "function_call" => "API integration and tool usage",
        "multi_round_dialogue" => "complex multi-agent conversations",
        "code_task" => "code generation, debugging, and programming tasks",
        "reflection" => "self-correction and iterative improvement",
        "retrieval_embedding" => "information retrieval and semantic search",
        _ => "general AI training tasks"
    };
    
    let suggestions = match &selected_model.provider {
        crate::types::ModelProvider::Ollama => {
            generate_ollama_suggestions(&model_id, &domain_text, &format, format_description).await?
        },
        crate::types::ModelProvider::OpenAI => {
            generate_openai_suggestions(&model_id, &domain_text, &format, format_description).await?
        }
    };
    
    Ok(suggestions)
}

async fn generate_ollama_suggestions(
    model_id: &str,
    domain_context: &str,
    format: &str,
    format_description: &str
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        "Generate exactly 5 specific fine-tuning goals for {} format in the {} domain.

Format: {}

Requirements:
- Each goal should be 1-2 sentences
- Focus on practical, actionable objectives
- Be specific to the domain and format
- Return only the 5 goals, numbered 1-5
- No additional text or explanations

Domain: {}",
        format, domain_context, format_description, domain_context
    );
    
    let request_body = serde_json::json!({
        "model": model_id,
        "prompt": prompt,
        "stream": false
    });
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
        
        let generated_text = result["response"].as_str().unwrap_or("");
        let suggestions = parse_suggestions(generated_text);
        
        if suggestions.is_empty() {
            Ok(get_fallback_suggestions(format, domain_context))
        } else {
            Ok(suggestions)
        }
    } else {
        Err("Failed to generate suggestions from Ollama".to_string())
    }
}

async fn generate_openai_suggestions(
    model_id: &str,
    domain_context: &str,
    format: &str,
    format_description: &str
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not found in environment. Please set it to use suggestions.".to_string())?;
    
    let prompt = format!(
        "Generate exactly 5 specific fine-tuning goals for {} format in the {} domain.

Format: {}

Requirements:
- Each goal should be 1-2 sentences
- Focus on practical, actionable objectives
- Be specific to the domain and format
- Return only the 5 goals, numbered 1-5
- No additional text or explanations

Domain: {}",
        format, domain_context, format_description, domain_context
    );
    
    let request_body = serde_json::json!({
        "model": model_id,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7
    });
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to OpenAI: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;
        
        let generated_text = result["choices"][0]["message"]["content"].as_str().unwrap_or("");
        let suggestions = parse_suggestions(generated_text);
        
        if suggestions.is_empty() {
            Ok(get_fallback_suggestions(format, domain_context))
        } else {
            Ok(suggestions)
        }
    } else {
        Err("Failed to generate suggestions from OpenAI".to_string())
    }
}

fn parse_suggestions(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            // Look for numbered lines (1., 2., etc.) or lines that start with numbers
            if line.starts_with(char::is_numeric) {
                // Remove the number and any punctuation at the start
                let content = line
                    .chars()
                    .skip_while(|&c| c.is_numeric() || c == '.' || c == ')' || c.is_whitespace())
                    .collect::<String>()
                    .trim()
                    .to_string();
                
                if !content.is_empty() && content.len() > 10 {
                    Some(content)
                } else {
                    None
                }
            } else if line.len() > 20 && !line.contains("generate") && !line.contains("example") {
                // Fallback: any substantial line that doesn't look like instructions
                Some(line.to_string())
            } else {
                None
            }
        })
        .take(5)
        .collect()
}

fn get_fallback_suggestions(format: &str, domain_context: &str) -> Vec<String> {
    match format {
        "alpaca" => vec![
            format!("Train the model to follow instructions in {}", domain_context),
            format!("Improve task completion accuracy for {} scenarios", domain_context),
            format!("Enhance response quality for {} domain questions", domain_context),
            format!("Develop expertise in {} problem-solving", domain_context),
            format!("Optimize instruction understanding for {} tasks", domain_context),
        ],
        "conversation" => vec![
            format!("Create engaging dialogues in {} contexts", domain_context),
            format!("Improve conversational flow for {} discussions", domain_context),
            format!("Enhance multi-turn context retention in {}", domain_context),
            format!("Develop natural conversation skills for {} support", domain_context),
            format!("Train for appropriate tone in {} interactions", domain_context),
        ],
        "chain_of_thought" => vec![
            format!("Improve step-by-step reasoning for {} problems", domain_context),
            format!("Enhance logical thinking in {} analysis", domain_context),
            format!("Develop clear explanation skills for {} concepts", domain_context),
            format!("Train systematic problem-solving in {}", domain_context),
            format!("Improve reasoning transparency for {} decisions", domain_context),
        ],
        _ => vec![
            format!("Enhance performance in {} domain tasks", domain_context),
            format!("Improve accuracy for {} related queries", domain_context),
            format!("Develop expertise in {} problem solving", domain_context),
            format!("Optimize responses for {} use cases", domain_context),
            format!("Train for better {} domain understanding", domain_context),
        ],
    }
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
                dataset_format: config.format.clone(),
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
            &config.format,
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

// ============================================================================
// Knowledge Base Commands
// ============================================================================

#[tauri::command]
pub async fn initialize_knowledge_base(state: State<'_, AppState>) -> Result<(), String> {
    let kb_manager = KnowledgeBaseManager::new(KnowledgeBaseConfig::default());
    
    match kb_manager.initialize().await {
        Ok(_) => {
            // Store the manager in state for future use
            let mut kb_state = state.knowledge_base_manager.write().await;
            *kb_state = Some(kb_manager);
            Ok(())
        }
        Err(e) => Err(format!("Failed to initialize knowledge base: {}", e)),
    }
}

#[tauri::command]
pub async fn get_knowledge_base_stats(state: State<'_, AppState>) -> Result<KnowledgeBaseStats, String> {
    let kb_state = state.knowledge_base_manager.read().await;
    
    if let Some(kb_manager) = kb_state.as_ref() {
        match kb_manager.get_stats().await {
            Ok(stats) => Ok(stats),
            Err(e) => Err(format!("Failed to get knowledge base stats: {}", e)),
        }
    } else {
        Err("Knowledge base not initialized".to_string())
    }
}

#[tauri::command]
pub async fn search_knowledge_base(
    query_text: String,
    use_case_filter: Option<String>,
    format_filter: Option<String>,
    min_quality_score: Option<f32>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let kb_state = state.knowledge_base_manager.read().await;
    
    if let Some(kb_manager) = kb_state.as_ref() {
        let format_filter_parsed = format_filter.and_then(|f| {
            match f.as_str() {
                "Alpaca" => Some(crate::types::DatasetFormat::Alpaca),
                "Conversation" => Some(crate::types::DatasetFormat::Conversation),
                "ChainOfThought" => Some(crate::types::DatasetFormat::ChainOfThought),
                "PreferenceRanking" => Some(crate::types::DatasetFormat::PreferenceRanking),
                "FunctionCall" => Some(crate::types::DatasetFormat::FunctionCall),
                "MultiRoundDialogue" => Some(crate::types::DatasetFormat::MultiRoundDialogue),
                "CodeTask" => Some(crate::types::DatasetFormat::CodeTask),
                "Reflection" => Some(crate::types::DatasetFormat::Reflection),
                "RetrievalEmbedding" => Some(crate::types::DatasetFormat::RetrievalEmbedding),
                _ => None,
            }
        });

        let query = QueryRequest {
            query_text,
            use_case_filter,
            format_filter: format_filter_parsed,
            min_quality_score,
            limit: limit.unwrap_or(10),
        };

        match kb_manager.search_knowledge_base(query).await {
            Ok(results) => Ok(results),
            Err(e) => Err(format!("Failed to search knowledge base: {}", e)),
        }
    } else {
        Err("Knowledge base not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_improvement_suggestions(
    entries: Vec<DatasetEntry>,
    use_case: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<Vec<ImprovementSuggestion>, String> {
    let kb_state = state.knowledge_base_manager.read().await;
    
    if let Some(kb_manager) = kb_state.as_ref() {
        let dataset_format = match format.as_str() {
            "Alpaca" => crate::types::DatasetFormat::Alpaca,
            "Conversation" => crate::types::DatasetFormat::Conversation,
            "ChainOfThought" => crate::types::DatasetFormat::ChainOfThought,
            "PreferenceRanking" => crate::types::DatasetFormat::PreferenceRanking,
            "FunctionCall" => crate::types::DatasetFormat::FunctionCall,
            "MultiRoundDialogue" => crate::types::DatasetFormat::MultiRoundDialogue,
            "CodeTask" => crate::types::DatasetFormat::CodeTask,
            "Reflection" => crate::types::DatasetFormat::Reflection,
            "RetrievalEmbedding" => crate::types::DatasetFormat::RetrievalEmbedding,
            _ => crate::types::DatasetFormat::Alpaca,
        };

        match kb_manager.get_improvement_suggestions(&entries, &use_case, &dataset_format).await {
            Ok(suggestions) => Ok(suggestions),
            Err(e) => Err(format!("Failed to get improvement suggestions: {}", e)),
        }
    } else {
        Err("Knowledge base not initialized".to_string())
    }
}

#[tauri::command]
pub async fn list_collections(state: State<'_, AppState>) -> Result<Vec<CollectionInfo>, String> {
    let kb_state = state.knowledge_base_manager.read().await;
    
    if let Some(kb_manager) = kb_state.as_ref() {
        match kb_manager.list_collections().await {
            Ok(collections) => Ok(collections),
            Err(e) => Err(format!("Failed to list collections: {}", e)),
        }
    } else {
        Err("Knowledge base not initialized".to_string())
    }
}

#[tauri::command]
pub async fn generate_prompt_improvements(
    state: State<'_, AppState>,
    entries: Vec<crate::types::DatasetEntry>,
    use_case: String,
    format: crate::types::DatasetFormat,
) -> Result<String, String> {
    let kb_manager_guard = state.knowledge_base_manager.read().await;
    if let Some(kb_manager) = kb_manager_guard.as_ref() {
        match kb_manager.process_entries_with_feedback(entries, &use_case, &format).await {
            Ok((_stats, feedback)) => {
                // Generate dynamic prompt improvements
                let improvements = kb_manager.validator.generate_dynamic_prompt_improvements(&feedback);
                Ok(improvements)
            }
            Err(e) => Err(format!("Failed to generate prompt improvements: {}", e)),
        }
    } else {
        Err("Knowledge base not initialized".to_string())
    }
}

/// Start the ChromaDB server
#[tauri::command]
pub async fn start_chromadb_server(state: State<'_, AppState>) -> Result<String, String> {
    let server = &state.chromadb_server;
    
    match server.start_server().await {
        Ok(()) => {
            let status = server.get_server_status().await;
            Ok(format!("ChromaDB server started successfully on {}", status.base_url))
        }
        Err(e) => Err(format!("Failed to start ChromaDB server: {}", e)),
    }
}

/// Stop the ChromaDB server
#[tauri::command]
pub async fn stop_chromadb_server(state: State<'_, AppState>) -> Result<String, String> {
    let server = &state.chromadb_server;
    
    match server.stop_server().await {
        Ok(()) => Ok("ChromaDB server stopped successfully".to_string()),
        Err(e) => Err(format!("Failed to stop ChromaDB server: {}", e)),
    }
}

/// Get ChromaDB server status
#[tauri::command]
pub async fn get_chromadb_server_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let server = &state.chromadb_server;
    let status = server.get_server_status().await;
    
    Ok(serde_json::json!({
        "is_running": status.is_running,
        "has_process": status.has_process,
        "base_url": status.base_url,
        "port": status.port
    }))
}

/// Check if ChromaDB is available (installed)
#[tauri::command]
pub async fn check_chromadb_available(state: State<'_, AppState>) -> Result<bool, String> {
    let server = &state.chromadb_server;
    
    match server.check_chromadb_available() {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}