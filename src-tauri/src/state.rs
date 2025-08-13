use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use crate::types::{Model, DatasetEntry, GenerationConfig, GenerationProgress};
use crate::knowledge_base::KnowledgeBaseManager;
use crate::chromadb_server::ChromaDbServerManager;

pub struct AppState {
    pub models: Arc<RwLock<Vec<Model>>>,
    pub dataset: Arc<RwLock<Vec<DatasetEntry>>>,
    pub generation_config: Arc<RwLock<Option<GenerationConfig>>>,
    pub progress: Arc<RwLock<GenerationProgress>>,
    pub active_generations: Arc<RwLock<HashMap<String, CancellationToken>>>,
    pub knowledge_base_manager: Arc<RwLock<Option<KnowledgeBaseManager>>>,
    pub chromadb_server: Arc<ChromaDbServerManager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(Vec::new())),
            dataset: Arc::new(RwLock::new(Vec::new())),
            generation_config: Arc::new(RwLock::new(None)),
            progress: Arc::new(RwLock::new(GenerationProgress {
                current_batch: 0,
                total_batches: 0,
                entries_generated: 0,
                estimated_completion: "Not started".to_string(),
                status: "idle".to_string(),
                generation_id: None,
                concurrent_batches: 0,
                entries_per_second: 0.0,
                errors_count: 0,
                retries_count: 0,
            })),
            active_generations: Arc::new(RwLock::new(HashMap::new())),
            knowledge_base_manager: Arc::new(RwLock::new(None)),
            chromadb_server: Arc::new(ChromaDbServerManager::new()),
        }
    }
}