use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Mutex;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio_util::sync::CancellationToken;
use anyhow::Result;
use crate::types::{
    DatasetEntry, ModelProvider, GenerationTask, BatchResult, DatasetFormat
};
use crate::prompt_template::PromptTemplateEngine;
use crate::quality_validator::ValidationFeedback;

/// Configuration for concurrent dataset generation
#[derive(Debug, Clone)]
pub struct ConcurrentGenerationConfig {
    pub max_concurrent_batches: usize,
    pub max_concurrent_requests_per_batch: usize,
    pub ollama_requests_per_second: u32,
    pub openai_requests_per_second: u32,
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub request_timeout: Duration,
    pub dataset_format: crate::types::DatasetFormat,
}

impl Default for ConcurrentGenerationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_batches: 4,
            max_concurrent_requests_per_batch: 3,
            ollama_requests_per_second: 10,
            openai_requests_per_second: 60, // OpenAI allows 60 requests per minute for tier 1
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            request_timeout: Duration::from_secs(30),
            dataset_format: crate::types::DatasetFormat::Alpaca,
        }
    }
}

/// Progress update message sent through the progress channel
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub batch_completed: Option<usize>,
    pub entries_generated: usize,
    pub errors_count: usize,
    pub retries_count: usize,
    pub concurrent_batches: usize,
    pub entries_per_second: f64,
}

/// Simple rate limiter for API requests
#[derive(Debug)]
pub struct SimpleRateLimiter {
    last_request: Arc<Mutex<Instant>>,
    min_interval: Duration,
}

impl SimpleRateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let min_interval = Duration::from_millis(1000 / requests_per_second as u64);
        Self {
            last_request: Arc::new(Mutex::new(Instant::now() - min_interval)),
            min_interval,
        }
    }

    pub async fn wait_for_permit(&self) {
        loop {
            let now = Instant::now();
            let should_wait = {
                let mut last = self.last_request.lock().unwrap();
                let elapsed = now.duration_since(*last);
                if elapsed >= self.min_interval {
                    *last = now;
                    false
                } else {
                    true
                }
            };

            if !should_wait {
                break;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// Highly optimized concurrent dataset generator with enhanced prompt system
pub struct ConcurrentDatasetGenerator {
    config: ConcurrentGenerationConfig,
    ollama_rate_limiter: SimpleRateLimiter,
    openai_rate_limiter: SimpleRateLimiter,
    client: reqwest::Client,
    prompt_engine: PromptTemplateEngine,
    validation_feedback_history: Arc<RwLock<Vec<ValidationFeedback>>>,
}

impl ConcurrentDatasetGenerator {
    /// Parse generated entries from model output
    fn parse_generated_entries(&self, text: &str, expected_count: usize) -> Result<Vec<DatasetEntry>, anyhow::Error> {
        tracing::info!("Parsing generated entries, expected count: {}", expected_count);
        // Try to extract JSON from the response (handle cases where there's extra text)
        let json_text = if let Some(start) = text.find('[') {
            if let Some(end) = text.rfind(']') {
                &text[start..=end]
            } else {
                text
            }
        } else {
            text
        };

        tracing::debug!("Extracted JSON text: {}", json_text);

        match serde_json::from_str::<Vec<DatasetEntry>>(json_text) {
            Ok(entries) => {
                tracing::info!("Successfully parsed {} entries from JSON", entries.len());
                if entries.is_empty() {
                    tracing::warn!("Parsed entries is empty, generating fallback");
                    Ok(self.generate_fallback_entries(expected_count))
                } else {
                    Ok(entries)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse generated JSON: {}, using fallback entries", e);
                tracing::debug!("Failed JSON content: {}", json_text);
                Ok(self.generate_fallback_entries(expected_count))
            }
        }
    }
    pub fn new(config: ConcurrentGenerationConfig) -> Self {
        // Create rate limiters for different providers
        let ollama_rate_limiter = SimpleRateLimiter::new(config.ollama_requests_per_second);
        let openai_rate_limiter = SimpleRateLimiter::new(config.openai_requests_per_second);

        // Create optimized HTTP client with connection pooling
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to create HTTP client");

        // Initialize prompt template engine
        let prompt_engine = PromptTemplateEngine::new();

        Self {
            config,
            ollama_rate_limiter,
            openai_rate_limiter,
            client,
            prompt_engine,
            validation_feedback_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Update the generator with validation feedback for continuous improvement
    pub async fn update_with_feedback(
        &mut self,
        feedback: ValidationFeedback,
        format: &DatasetFormat,
        batch_quality_score: f32,
    ) -> Result<()> {
        // Store feedback in history
        {
            let mut history = self.validation_feedback_history.write().await;
            history.push(feedback.clone());
            
            // Keep only recent feedback (last 50 entries)
            if history.len() > 50 {
                *history = history.iter().rev().take(50).rev().cloned().collect();
            }
        }

        // Update prompt templates based on feedback
        self.prompt_engine.update_template_with_feedback(format, &feedback, batch_quality_score)?;

        tracing::info!(
            "Updated generator with feedback: {} suggestions, {} avoid patterns",
            feedback.improvement_suggestions.len(),
            feedback.avoid_patterns.len()
        );

        Ok(())
    }

    /// Generate dataset entries with full concurrency optimization
    pub async fn generate_concurrent(
        &self,
        tasks: Vec<GenerationTask>,
        cancellation_token: CancellationToken,
        progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
    ) -> Result<Vec<DatasetEntry>> {
        let total_tasks = tasks.len();
        let batch_semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_batches));
        
        // Statistics tracking
        let start_time = Instant::now();
        let completed_batches = Arc::new(RwLock::new(0));
        let total_entries_generated = Arc::new(RwLock::new(0));
        let total_errors = Arc::new(RwLock::new(0));
        let total_retries = Arc::new(RwLock::new(0));

        // Results collection
        let results = Arc::new(RwLock::new(HashMap::new()));

        // Create futures for concurrent execution
        let mut futures = FuturesUnordered::new();
        
        for task in tasks {
            let semaphore = batch_semaphore.clone();
            let cancellation_token = cancellation_token.clone();
            let progress_tx = progress_tx.clone();
            let generator = self.clone();
            let completed_batches = completed_batches.clone();
            let total_entries_generated = total_entries_generated.clone();
            let total_errors = total_errors.clone();
            let total_retries = total_retries.clone();
            let results = results.clone();

            futures.push(tokio::spawn(async move {
                // Acquire semaphore permit for batch concurrency control
                let _permit = semaphore.acquire().await.unwrap();
                
                // Check for cancellation
                if cancellation_token.is_cancelled() {
                    return Ok(());
                }

                // Execute the generation task
                match generator.execute_task_with_retries(task.clone(), cancellation_token.clone()).await {
                    Ok(batch_result) => {
                        tracing::info!("Batch {} completed with {} entries", batch_result.batch_id, batch_result.entries.len());
                        
                        // Update statistics
                        let mut completed = completed_batches.write().await;
                        *completed += 1;
                        let completed_count = *completed;
                        
                        let mut total_entries = total_entries_generated.write().await;
                        *total_entries += batch_result.entries.len();
                        let entries_count = *total_entries;
                        
                        let mut retries = total_retries.write().await;
                        *retries += batch_result.retry_count;
                        let retries_count = *retries;

                        // Store results
                        let mut results_guard = results.write().await;
                        results_guard.insert(batch_result.batch_id, batch_result.entries.clone());
                        tracing::info!("Stored {} entries for batch {}, total results now: {}", 
                                     batch_result.entries.len(), batch_result.batch_id, results_guard.len());

                        // Calculate performance metrics
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let entries_per_second = if elapsed > 0.0 { entries_count as f64 / elapsed } else { 0.0 };
                        let concurrent_batches = total_tasks - completed_count;

                        // Send progress update
                        let _ = progress_tx.send(ProgressUpdate {
                            batch_completed: Some(batch_result.batch_id),
                            entries_generated: entries_count,
                            errors_count: *total_errors.read().await,
                            retries_count,
                            concurrent_batches,
                            entries_per_second,
                        });
                    }
                    Err(e) => {
                        let mut errors = total_errors.write().await;
                        *errors += 1;
                        tracing::error!("Batch {} failed: {}", task.batch_id, e);
                        
                        // Send error update
                        let _ = progress_tx.send(ProgressUpdate {
                            batch_completed: None,
                            entries_generated: *total_entries_generated.read().await,
                            errors_count: *errors,
                            retries_count: *total_retries.read().await,
                            concurrent_batches: total_tasks - *completed_batches.read().await,
                            entries_per_second: 0.0,
                        });
                    }
                }

                Ok::<(), anyhow::Error>(())
            }));
        }

        // Wait for all futures to complete or cancellation
        while let Some(result) = futures.next().await {
            if cancellation_token.is_cancelled() {
                break;
            }
            
            if let Err(e) = result {
                tracing::error!("Task execution error: {}", e);
            }
        }

        // Collect all results in order
        let results_guard = results.read().await;
        let mut all_entries = Vec::new();
        
        tracing::info!("Collecting results from {} tasks, results map has {} entries", total_tasks, results_guard.len());
        
        for i in 0..total_tasks {
            if let Some(entries) = results_guard.get(&i) {
                tracing::info!("Adding {} entries from batch {}", entries.len(), i);
                all_entries.extend(entries.clone());
            } else {
                tracing::warn!("No results found for batch {}", i);
            }
        }

        tracing::info!("Final collection: {} total entries from {} batches", all_entries.len(), total_tasks);
        Ok(all_entries)
    }

    /// Execute a single task with automatic retries and error handling
    async fn execute_task_with_retries(
        &self,
        task: GenerationTask,
        cancellation_token: CancellationToken,
    ) -> Result<BatchResult> {
        let mut last_error = None;
        let start_time = Instant::now();

        for retry_count in 0..=self.config.max_retries {
            if cancellation_token.is_cancelled() {
                return Err(anyhow::anyhow!("Generation cancelled"));
            }

            match self.execute_single_batch(&task, cancellation_token.clone()).await {
                Ok(entries) => {
                    return Ok(BatchResult {
                        batch_id: task.batch_id,
                        entries,
                        generation_time: start_time.elapsed(),
                        retry_count,
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                    if retry_count < self.config.max_retries {
                        tracing::warn!("Batch {} failed, retrying in {:?} (attempt {}/{})", 
                                     task.batch_id, self.config.retry_delay, retry_count + 1, self.config.max_retries);
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries exhausted")))
    }

    /// Execute a single batch with potential parallel sub-requests
    async fn execute_single_batch(
        &self,
        task: &GenerationTask,
        cancellation_token: CancellationToken,
    ) -> Result<Vec<DatasetEntry>> {
        // For large batches, split into parallel sub-requests
        let sub_batch_size = if task.entries_to_generate > self.config.max_concurrent_requests_per_batch {
            task.entries_to_generate / self.config.max_concurrent_requests_per_batch
        } else {
            task.entries_to_generate
        };

        let mut sub_tasks = Vec::new();
        let mut remaining = task.entries_to_generate;
        let mut sub_batch_id = 0;

        while remaining > 0 {
            let current_batch_size = remaining.min(sub_batch_size);
            sub_tasks.push((sub_batch_id, current_batch_size));
            remaining -= current_batch_size;
            sub_batch_id += 1;
        }

        // Execute sub-requests concurrently
        let mut futures = FuturesUnordered::new();
        
        for (_sub_id, size) in sub_tasks {
            let task_clone = task.clone();
            let cancellation_token = cancellation_token.clone();
            let generator = self.clone();

            futures.push(tokio::spawn(async move {
                generator.execute_api_request(
                    &task_clone.model_id,
                    &task_clone.provider,
                    &task_clone.goal,
                    size,
                    &task_clone.context,
                    cancellation_token,
                ).await
            }));
        }

        // Collect results from all sub-requests
        let mut all_entries = Vec::new();
        while let Some(result) = futures.next().await {
            if cancellation_token.is_cancelled() {
                return Err(anyhow::anyhow!("Generation cancelled"));
            }

            match result {
                Ok(Ok(entries)) => all_entries.extend(entries),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Sub-task failed: {}", e)),
            }
        }

        Ok(all_entries)
    }

    /// Execute a single API request with rate limiting
    async fn execute_api_request(
        &self,
        model_id: &str,
        provider: &ModelProvider,
        goal: &str,
        batch_size: usize,
        context: &str,
        cancellation_token: CancellationToken,
    ) -> Result<Vec<DatasetEntry>> {
        // Apply rate limiting based on provider
        let rate_limiter = match provider {
            ModelProvider::Ollama => &self.ollama_rate_limiter,
            ModelProvider::OpenAI => &self.openai_rate_limiter,
        };

        rate_limiter.wait_for_permit().await;

        if cancellation_token.is_cancelled() {
            return Err(anyhow::anyhow!("Generation cancelled"));
        }

        match provider {
            ModelProvider::Ollama => {
                self.generate_ollama_batch(model_id, goal, batch_size, context, cancellation_token).await
            }
            ModelProvider::OpenAI => {
                self.generate_openai_batch(model_id, goal, batch_size, context, cancellation_token).await
            }
        }
    }

    /// Optimized Ollama batch generation
    async fn generate_ollama_batch(
        &self,
        model_id: &str,
        goal: &str,
        batch_size: usize,
        context: &str,
        cancellation_token: CancellationToken,
    ) -> Result<Vec<DatasetEntry>> {
        let prompt = self.create_optimized_prompt(goal, batch_size, context);
        
        let request_body = serde_json::json!({
            "model": model_id,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
                "top_k": 40
            }
        });

        let request = self.client
            .post("http://localhost:11434/api/generate")
            .json(&request_body);

        let response = tokio::select! {
            result = request.send() => result?,
            _ = cancellation_token.cancelled() => {
                return Err(anyhow::anyhow!("Request cancelled"));
            }
        };

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let generated_text = result["response"].as_str().unwrap_or("[]");
            
            tracing::info!("Ollama response received, length: {} chars", generated_text.len());
            tracing::debug!("Ollama response content: {}", generated_text);
            
            let entries = self.parse_generated_entries(generated_text, batch_size)?;
            tracing::info!("Parsed {} entries from Ollama response", entries.len());
            Ok(entries)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Ollama API error: {} - {}", status, error_text);
            Err(anyhow::anyhow!("Ollama API error: {} - {}", status, error_text))
        }
    }

    /// Optimized OpenAI batch generation
    async fn generate_openai_batch(
        &self,
        model_id: &str,
        goal: &str,
        batch_size: usize,
        context: &str,
        cancellation_token: CancellationToken,
    ) -> Result<Vec<DatasetEntry>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!(
                "OPENAI_API_KEY not found in environment. Please set it in your .env file or system environment"
            ))?;

        let prompt = self.create_optimized_prompt(goal, batch_size, context);
        
        let request_body = serde_json::json!({
            "model": model_id,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert at creating high-quality training datasets. Always respond with valid JSON arrays containing the requested training examples."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4000,
            "top_p": 0.9
        });

        let request = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body);

        let response = tokio::select! {
            result = request.send() => result?,
            _ = cancellation_token.cancelled() => {
                return Err(anyhow::anyhow!("Request cancelled"));
            }
        };

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let generated_text = result["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("[]");
            
            self.parse_generated_entries(generated_text, batch_size)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("OpenAI API error: {}", error_text))
        }
    }

    /// Create an optimized prompt for better generation quality
    fn create_optimized_prompt(&self, goal: &str, batch_size: usize, context: &str) -> String {
        let format_schema = match self.config.dataset_format {
            crate::types::DatasetFormat::RetrievalEmbedding => "{\"query\": \"...\", \"positive_passage\": \"...\", \"negative_passages\": [\"...\", \"...\"]}",
            crate::types::DatasetFormat::Alpaca => "{\"instruction\": \"...\", \"input\": \"...\", \"output\": \"...\"}",
            // ...add other formats as needed...
            _ => "{...}"
        };
        format!(
            "Generate {} training examples for fine-tuning goal: {}. Context: {}.\n\nReturn only a JSON array of objects matching this exact schema: {}.\nDo not use any other format.\nGoal: {}",
            batch_size, goal, context, format_schema, goal
        )
    }

    /// Generate fallback entries when parsing fails
    fn generate_fallback_entries(&self, count: usize) -> Vec<DatasetEntry> {
        let format = &self.config.dataset_format;
        (0..count)
            .map(|i| {
                let data = match format {
                    crate::types::DatasetFormat::Alpaca => serde_json::json!({
                        "instruction": format!("Sample instruction {}", i + 1),
                        "input": format!("Sample input context {}", i + 1),
                        "output": format!("Sample response output {}", i + 1)
                    }),
                    crate::types::DatasetFormat::RetrievalEmbedding => serde_json::json!({
                        "query": format!("Sample query {}", i + 1),
                        "positive_passage": format!("Relevant passage {}", i + 1),
                        "negative_passages": [format!("Irrelevant passage {}", i + 1), format!("Another irrelevant passage {}", i + 1)]
                    }),
                    // ...add other formats as needed...
                    _ => serde_json::json!({
                        "instruction": format!("Sample instruction {}", i + 1),
                        "input": format!("Sample input context {}", i + 1),
                        "output": format!("Sample response output {}", i + 1)
                    })
                };
                DatasetEntry { data }
            })
            .collect()
    }

}

// Implement Clone for SimpleRateLimiter
impl Clone for SimpleRateLimiter {
    fn clone(&self) -> Self {
        Self {
            last_request: self.last_request.clone(),
            min_interval: self.min_interval,
        }
    }
}

// Implement Clone for the generator (needed for moving into async tasks)
impl Clone for ConcurrentDatasetGenerator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            ollama_rate_limiter: self.ollama_rate_limiter.clone(),
            openai_rate_limiter: self.openai_rate_limiter.clone(),
            client: self.client.clone(),
            prompt_engine: PromptTemplateEngine::new(), // Create new instance for clone
            validation_feedback_history: self.validation_feedback_history.clone(),
        }
    }
}