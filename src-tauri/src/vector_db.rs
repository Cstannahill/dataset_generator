use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::embedding_service::EmbeddingResult;
use crate::types::DatasetFormat;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub use_case: String,
    pub dataset_format: DatasetFormat,
    pub entry_count: usize,
    pub created_at: i64,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub distance: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query_text: String,
    pub use_case_filter: Option<String>,
    pub format_filter: Option<DatasetFormat>,
    pub min_quality_score: Option<f32>,
    pub limit: usize,
}

pub struct VectorDbService {
    client: reqwest::Client,
    base_url: String,
}

impl VectorDbService {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:8465".to_string()),
        }
    }

    /// Initialize the vector database and create necessary collections
    pub async fn initialize(&self) -> Result<()> {
        // Check if ChromaDB is running
        let health_check = self.client
            .get(&format!("{}/api/v1/heartbeat", self.base_url))
            .send()
            .await;

        match health_check {
            Ok(response) if response.status().is_success() => {
                tracing::info!("ChromaDB is running and accessible");
                Ok(())
            }
            _ => {
                tracing::warn!("ChromaDB is not accessible. Make sure ChromaDB is running on {}", self.base_url);
                Err(anyhow::anyhow!("ChromaDB connection failed"))
            }
        }
    }

    /// Store embeddings in the vector database, organized by use case and format
    pub async fn store_embeddings(&self, embeddings: Vec<EmbeddingResult>) -> Result<()> {
        // Group embeddings by use case and format for collection organization
        let mut collections: HashMap<String, Vec<EmbeddingResult>> = HashMap::new();

        for embedding in embeddings {
            let use_case = embedding.metadata.get("use_case")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let dataset_format = embedding.metadata.get("dataset_format")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let collection_key = format!("{}_{}", 
                use_case.replace(" ", "_").to_lowercase(),
                dataset_format.replace(" ", "_").to_lowercase()
            );

            collections.entry(collection_key).or_insert_with(Vec::new).push(embedding);
        }

        // Store each collection
        for (collection_name, collection_embeddings) in collections {
            match self.store_collection(&collection_name, collection_embeddings).await {
                Ok(_) => {
                    tracing::info!("Successfully stored collection: {}", collection_name);
                }
                Err(e) => {
                    tracing::error!("Failed to store collection {}: {}", collection_name, e);
                }
            }
        }

        Ok(())
    }

    /// Store a single collection in ChromaDB
    async fn store_collection(&self, collection_name: &str, embeddings: Vec<EmbeddingResult>) -> Result<()> {
        // Create collection if it doesn't exist
        self.create_collection(collection_name).await?;

        // Prepare data for ChromaDB
        let ids: Vec<String> = embeddings.iter().map(|e| e.id.clone()).collect();
        let documents: Vec<String> = embeddings.iter().map(|e| e.text.clone()).collect();
        let embeddings_data: Vec<Vec<f32>> = embeddings.iter().map(|e| e.embedding.clone()).collect();
        let metadatas: Vec<HashMap<String, serde_json::Value>> = embeddings.iter().map(|e| e.metadata.clone()).collect();

        let request_body = serde_json::json!({
            "ids": ids,
            "documents": documents,
            "embeddings": embeddings_data,
            "metadatas": metadatas
        });

        let response = self.client
            .post(&format!("{}/api/v1/collections/{}/add", self.base_url, collection_name))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Added {} entries to collection: {}", ids.len(), collection_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("ChromaDB add error: {}", error_text))
        }
    }

    /// Create a new collection in ChromaDB
    async fn create_collection(&self, collection_name: &str) -> Result<()> {
        let request_body = serde_json::json!({
            "name": collection_name,
            "metadata": {
                "description": format!("Dataset collection for {}", collection_name),
                "created_at": chrono::Utc::now().timestamp()
            }
        });

        let response = self.client
            .post(&format!("{}/api/v1/collections", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        match response.status().as_u16() {
            201 => {
                tracing::info!("Created new collection: {}", collection_name);
                Ok(())
            }
            409 => {
                tracing::debug!("Collection already exists: {}", collection_name);
                Ok(()) // Collection already exists, which is fine
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!("ChromaDB collection creation error: {}", error_text))
            }
        }
    }

    /// Search for similar entries in the knowledge base
    pub async fn search_similar(&self, query: QueryRequest) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.generate_query_embedding(&query.query_text).await?;

        // Determine which collections to search
        let collections = self.get_target_collections(&query).await?;

        let mut all_results = Vec::new();

        for collection_name in collections {
            match self.search_collection(&collection_name, &query_embedding, &query).await {
                Ok(mut results) => all_results.append(&mut results),
                Err(e) => tracing::warn!("Failed to search collection {}: {}", collection_name, e),
            }
        }

        // Sort by distance and limit results
        all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        all_results.truncate(query.limit);

        Ok(all_results)
    }

    /// Search within a specific collection
    async fn search_collection(
        &self,
        collection_name: &str,
        query_embedding: &[f32],
        query: &QueryRequest,
    ) -> Result<Vec<SearchResult>> {
        let mut request_body = serde_json::json!({
            "query_embeddings": [query_embedding],
            "n_results": query.limit
        });

        // Add metadata filters if specified
        let mut where_clause = HashMap::new();
        
        if let Some(min_score) = query.min_quality_score {
            where_clause.insert("overall_score".to_string(), serde_json::json!({"$gte": min_score}));
        }

        if !where_clause.is_empty() {
            request_body["where"] = serde_json::json!(where_clause);
        }

        let response = self.client
            .post(&format!("{}/api/v1/collections/{}/query", self.base_url, collection_name))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            self.parse_search_results(result)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("ChromaDB search error: {}", error_text))
        }
    }

    /// Parse ChromaDB search results
    fn parse_search_results(&self, result: serde_json::Value) -> Result<Vec<SearchResult>> {
        let mut search_results = Vec::new();

        if let Some(ids_array) = result["ids"].as_array() {
            if let Some(ids) = ids_array.get(0).and_then(|v| v.as_array()) {
                let empty_vec = vec![];
                let documents = result["documents"].as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);
                    
                let empty_vec2 = vec![];
                let distances = result["distances"].as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec2);
                    
                let empty_vec3 = vec![];
                let metadatas = result["metadatas"].as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec3);

                for i in 0..ids.len() {
                    if let Some(id) = ids[i].as_str() {
                        let text = documents.get(i)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                            
                        let distance = distances.get(i)
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32;
                            
                        let metadata = metadatas.get(i)
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        search_results.push(SearchResult {
                            id: id.to_string(),
                            text,
                            distance,
                            metadata,
                        });
                    }
                }
            }
        }

        Ok(search_results)
    }

    /// Generate embedding for a search query
    async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        let request_body = serde_json::json!({
            "model": "nomic-embed-text",
            "prompt": query
        });

        let response = self.client
            .post("http://localhost:11434/api/embeddings")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            if let Some(embedding_array) = result["embedding"].as_array() {
                let embedding: Result<Vec<f32>, _> = embedding_array
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(|| anyhow::anyhow!("Invalid embedding value")))
                    .collect();
                
                embedding
            } else {
                Err(anyhow::anyhow!("Invalid embedding response format"))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Ollama embedding API error: {}", error_text))
        }
    }

    /// Get target collections based on query filters
    async fn get_target_collections(&self, query: &QueryRequest) -> Result<Vec<String>> {
        // Get all collections
        let response = self.client
            .get(&format!("{}/api/v1/collections", self.base_url))
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let mut collections = Vec::new();

            if let Some(collections_array) = result.as_array() {
                for collection in collections_array {
                    if let Some(name) = collection["name"].as_str() {
                        // Filter collections based on query criteria
                        let mut should_include = true;

                        if let Some(use_case_filter) = &query.use_case_filter {
                            let use_case_key = use_case_filter.replace(" ", "_").to_lowercase();
                            if !name.contains(&use_case_key) {
                                should_include = false;
                            }
                        }

                        if let Some(format_filter) = &query.format_filter {
                            let format_key = format!("{:?}", format_filter).replace(" ", "_").to_lowercase();
                            if !name.contains(&format_key) {
                                should_include = false;
                            }
                        }

                        if should_include {
                            collections.push(name.to_string());
                        }
                    }
                }
            }

            Ok(collections)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("ChromaDB collections list error: {}", error_text))
        }
    }

    /// Get information about all collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self.client
            .get(&format!("{}/api/v1/collections", self.base_url))
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let mut collection_infos = Vec::new();

            if let Some(collections_array) = result.as_array() {
                for collection in collections_array {
                    if let Some(name) = collection["name"].as_str() {
                        // Parse collection name to extract use case and format
                        let parts: Vec<&str> = name.split('_').collect();
                        let use_case = parts.get(0).unwrap_or(&"unknown").to_string();
                        let format_str = parts.get(1).unwrap_or(&"unknown").to_string();
                        
                        let dataset_format = match format_str.as_str() {
                            "alpaca" => DatasetFormat::Alpaca,
                            "conversation" => DatasetFormat::Conversation,
                            "chainofthought" => DatasetFormat::ChainOfThought,
                            "preferenceranking" => DatasetFormat::PreferenceRanking,
                            "functioncall" => DatasetFormat::FunctionCall,
                            "multirounddialogue" => DatasetFormat::MultiRoundDialogue,
                            "codetask" => DatasetFormat::CodeTask,
                            "reflection" => DatasetFormat::Reflection,
                            "retrievalembedding" => DatasetFormat::RetrievalEmbedding,
                            _ => DatasetFormat::Alpaca,
                        };

                        // Get collection count
                        let count_response = self.client
                            .get(&format!("{}/api/v1/collections/{}/count", self.base_url, name))
                            .send()
                            .await;

                        let entry_count = match count_response {
                            Ok(resp) if resp.status().is_success() => {
                                resp.json::<serde_json::Value>().await
                                    .ok()
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as usize
                            }
                            _ => 0,
                        };

                        let created_at = collection["metadata"]["created_at"].as_i64().unwrap_or(0);

                        collection_infos.push(CollectionInfo {
                            name: name.to_string(),
                            use_case,
                            dataset_format,
                            entry_count,
                            created_at,
                            last_updated: chrono::Utc::now().timestamp(),
                        });
                    }
                }
            }

            Ok(collection_infos)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("ChromaDB collections list error: {}", error_text))
        }
    }
}

/// Configuration for vector database service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    pub base_url: String,
    pub enable_storage: bool,
}

impl Default for VectorDbConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8465".to_string(),
            enable_storage: true,
        }
    }
}
