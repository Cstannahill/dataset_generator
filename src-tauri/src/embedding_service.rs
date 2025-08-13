use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::quality_validator::ValidatedEntry;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub id: String,
    pub embedding: Vec<f32>,
    pub text: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct EmbeddingService {
    client: reqwest::Client,
    model_name: String,
}

impl EmbeddingService {
    pub fn new(model_name: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            model_name: model_name.unwrap_or_else(|| "nomic-embed-text".to_string()),
        }
    }

    /// Generate embeddings for a batch of validated entries
    pub async fn embed_entries(&self, entries: &[ValidatedEntry]) -> Result<Vec<EmbeddingResult>> {
        let mut embedding_results = Vec::new();

        for entry in entries {
            match self.embed_single_entry(&entry).await {
                Ok(embedding_result) => {
                    embedding_results.push(embedding_result);
                }
                Err(e) => {
                    tracing::warn!("Failed to embed entry: {}", e);
                    // Continue with other entries
                }
            }
        }

        tracing::info!("Generated embeddings for {} entries", embedding_results.len());
        Ok(embedding_results)
    }

    /// Generate embedding for a single entry
    async fn embed_single_entry(&self, validated_entry: &ValidatedEntry) -> Result<EmbeddingResult> {
        let text_content = self.extract_text_content(validated_entry);
        let embedding = self.generate_embedding(&text_content).await?;
        
        let mut metadata = HashMap::new();
        metadata.insert("use_case".to_string(), serde_json::Value::String(validated_entry.metadata.use_case.clone()));
        metadata.insert("dataset_format".to_string(), serde_json::Value::String(format!("{:?}", validated_entry.metadata.dataset_format)));
        metadata.insert("content_hash".to_string(), serde_json::Value::String(validated_entry.metadata.content_hash.clone()));
        metadata.insert("validation_timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(validated_entry.metadata.validation_timestamp)));
        metadata.insert("overall_score".to_string(), serde_json::to_value(validated_entry.quality_score.overall_score).unwrap_or(serde_json::Value::Null));
        metadata.insert("relevance_score".to_string(), serde_json::to_value(validated_entry.quality_score.relevance_score).unwrap_or(serde_json::Value::Null));
        metadata.insert("coherence_score".to_string(), serde_json::to_value(validated_entry.quality_score.coherence_score).unwrap_or(serde_json::Value::Null));
        metadata.insert("completeness_score".to_string(), serde_json::to_value(validated_entry.quality_score.completeness_score).unwrap_or(serde_json::Value::Null));
        metadata.insert("format_compliance_score".to_string(), serde_json::to_value(validated_entry.quality_score.format_compliance_score).unwrap_or(serde_json::Value::Null));
        metadata.insert("tags".to_string(), serde_json::Value::Array(
            validated_entry.quality_score.tags.iter()
                .map(|tag| serde_json::Value::String(tag.clone()))
                .collect()
        ));
        metadata.insert("issues".to_string(), serde_json::Value::Array(
            validated_entry.quality_score.issues.iter()
                .map(|issue| serde_json::Value::String(issue.clone()))
                .collect()
        ));

        let embedding_id = uuid::Uuid::new_v4().to_string();

        Ok(EmbeddingResult {
            id: embedding_id,
            embedding,
            text: text_content,
            metadata,
        })
    }

    /// Extract meaningful text content from a validated entry for embedding
    fn extract_text_content(&self, validated_entry: &ValidatedEntry) -> String {
        let data = &validated_entry.entry.data;
        
        // Extract text based on dataset format
        let content_parts: Vec<String> = match validated_entry.metadata.dataset_format {
            crate::types::DatasetFormat::Alpaca => {
                vec![
                    data.get("instruction").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("input").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ]
            }
            crate::types::DatasetFormat::Conversation => {
                let mut parts = Vec::new();
                if let Some(messages) = data.get("messages").and_then(|v| v.as_array()) {
                    for message in messages {
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            parts.push(content.to_string());
                        }
                    }
                }
                parts
            }
            crate::types::DatasetFormat::ChainOfThought => {
                vec![
                    data.get("question").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("answer").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ]
            }
            crate::types::DatasetFormat::PreferenceRanking => {
                vec![
                    data.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("chosen").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("rejected").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ]
            }
            crate::types::DatasetFormat::FunctionCall => {
                let mut parts = Vec::new();
                if let Some(messages) = data.get("messages").and_then(|v| v.as_array()) {
                    for message in messages {
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            parts.push(content.to_string());
                        }
                    }
                }
                if let Some(function_data) = data.get("function") {
                    parts.push(serde_json::to_string(function_data).unwrap_or_default());
                }
                parts
            }
            crate::types::DatasetFormat::MultiRoundDialogue => {
                let mut parts = Vec::new();
                parts.push(data.get("instruction").and_then(|v| v.as_str()).unwrap_or("").to_string());
                if let Some(conversation) = data.get("conversation").and_then(|v| v.as_array()) {
                    for turn in conversation {
                        if let Some(content) = turn.get("content").and_then(|v| v.as_str()) {
                            parts.push(content.to_string());
                        }
                    }
                }
                parts
            }
            crate::types::DatasetFormat::CodeTask => {
                vec![
                    data.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ]
            }
            crate::types::DatasetFormat::Reflection => {
                vec![
                    data.get("instruction").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("reflection").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("corrected").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ]
            }
            crate::types::DatasetFormat::RetrievalEmbedding => {
                let mut parts = vec![
                    data.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    data.get("positive_passage").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ];
                
                if let Some(negative_passages) = data.get("negative_passages").and_then(|v| v.as_array()) {
                    for passage in negative_passages {
                        if let Some(text) = passage.as_str() {
                            parts.push(text.to_string());
                        }
                    }
                }
                parts
            }
            crate::types::DatasetFormat::Reranking => {
                let mut parts = vec![
                    data.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ];
                
                if let Some(documents) = data.get("documents").and_then(|v| v.as_array()) {
                    for doc in documents {
                        if let Some(text) = doc.as_str() {
                            parts.push(text.to_string());
                        }
                    }
                }
                parts
            }
        };

        // Combine all text parts
        let combined_text = content_parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Add use case context
        format!("Use case: {} | Content: {}", validated_entry.metadata.use_case, combined_text)
    }

    /// Generate embedding using Ollama's nomic-embed-text model
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request_body = serde_json::json!({
            "model": self.model_name,
            "prompt": text
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
}

/// Configuration for embedding service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub enable_embeddings: bool,
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "nomic-embed-text".to_string(),
            enable_embeddings: true,
            batch_size: 20,
        }
    }
}
