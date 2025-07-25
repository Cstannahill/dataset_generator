use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub size: String,
    pub modified: String,
    pub provider: ModelProvider,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelProvider {
    Ollama,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub instruction: String,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub target_entries: usize,
    pub batch_size: usize,
    pub selected_model: String,
    pub fine_tuning_goal: String,
    pub domain_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationProgress {
    pub current_batch: usize,
    pub total_batches: usize,
    pub entries_generated: usize,
    pub estimated_completion: String,
    pub status: String,
    pub generation_id: Option<String>,
    pub concurrent_batches: usize,
    pub entries_per_second: f64,
    pub errors_count: usize,
    pub retries_count: usize,
}

#[derive(Debug, Clone)]
pub struct GenerationTask {
    pub id: String,
    pub batch_id: usize,
    pub entries_to_generate: usize,
    pub model_id: String,
    pub provider: ModelProvider,
    pub goal: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_id: usize,
    pub entries: Vec<DatasetEntry>,
    pub generation_time: std::time::Duration,
    pub retry_count: usize,
}