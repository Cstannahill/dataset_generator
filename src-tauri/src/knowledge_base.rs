use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::types::{DatasetEntry, DatasetFormat};
use crate::quality_validator::{QualityValidator, ValidatedEntry, ValidationConfig, ValidationFeedback};
use crate::embedding_service::{EmbeddingService, EmbeddingConfig};
use crate::vector_db::{VectorDbService, CollectionInfo, SearchResult, QueryRequest, VectorDbConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    pub validation: ValidationConfig,
    pub embedding: EmbeddingConfig,
    pub vector_db: VectorDbConfig,
    pub enable_knowledge_base: bool,
}

impl Default for KnowledgeBaseConfig {
    fn default() -> Self {
        Self {
            validation: ValidationConfig::default(),
            embedding: EmbeddingConfig::default(),
            vector_db: VectorDbConfig::default(),
            enable_knowledge_base: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_entries: usize,
    pub validated_entries: usize,
    pub embedded_entries: usize,
    pub stored_entries: usize,
    pub validation_time_ms: u64,
    pub embedding_time_ms: u64,
    pub storage_time_ms: u64,
}

pub struct KnowledgeBaseManager {
    pub validator: QualityValidator,
    embedding_service: EmbeddingService,
    vector_db: VectorDbService,
    config: KnowledgeBaseConfig,
}

impl KnowledgeBaseManager {
    pub fn new(config: KnowledgeBaseConfig) -> Self {
        let validator = QualityValidator::new(Some(config.validation.model_name.clone()));
        let embedding_service = EmbeddingService::new(Some(config.embedding.model_name.clone()));
        let vector_db = VectorDbService::new(Some(config.vector_db.base_url.clone()));

        Self {
            validator,
            embedding_service,
            vector_db,
            config,
        }
    }

    /// Initialize the knowledge base system
    pub async fn initialize(&self) -> Result<()> {
        if !self.config.enable_knowledge_base {
            tracing::info!("Knowledge base is disabled");
            return Ok(());
        }

        tracing::info!("Initializing knowledge base system...");

        // Initialize vector database
        self.vector_db.initialize().await?;

        tracing::info!("Knowledge base system initialized successfully");
        Ok(())
    }

    /// Process a batch of dataset entries through the complete knowledge base pipeline
    pub async fn process_entries(
        &self,
        entries: Vec<DatasetEntry>,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<ProcessingStats> {
        if !self.config.enable_knowledge_base {
            tracing::info!("Knowledge base processing is disabled, skipping");
            return Ok(ProcessingStats {
                total_entries: entries.len(),
                validated_entries: 0,
                embedded_entries: 0,
                stored_entries: 0,
                validation_time_ms: 0,
                embedding_time_ms: 0,
                storage_time_ms: 0,
            });
        }

        let total_entries = entries.len();
        tracing::info!("Processing {} entries through knowledge base pipeline", total_entries);

        // Step 1: Quality validation
        let validation_start = std::time::Instant::now();
        let validated_entries = if self.config.validation.enable_validation {
            self.validator.validate_entries(entries, use_case, format).await?
        } else {
            // Skip validation, convert entries to validated format
            entries.into_iter().map(|entry| {
                ValidatedEntry {
                    entry,
                    quality_score: crate::quality_validator::QualityScore {
                        overall_score: 1.0,
                        relevance_score: 1.0,
                        coherence_score: 1.0,
                        completeness_score: 1.0,
                        format_compliance_score: 1.0,
                        issues: vec![],
                        tags: vec!["unvalidated".to_string()],
                    },
                    metadata: crate::quality_validator::EntryMetadata {
                        use_case: use_case.to_string(),
                        dataset_format: format.clone(),
                        content_hash: format!("unvalidated_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()),
                        validation_timestamp: chrono::Utc::now().timestamp(),
                        embedding_id: None,
                    },
                }
            }).collect()
        };
        let validation_time = validation_start.elapsed();

        tracing::info!("Quality validation completed: {}/{} entries passed", 
                      validated_entries.len(), total_entries);

        // Step 2: Generate embeddings
        let embedding_start = std::time::Instant::now();
        let embedding_results = if self.config.embedding.enable_embeddings && !validated_entries.is_empty() {
            self.embedding_service.embed_entries(&validated_entries).await?
        } else {
            vec![]
        };
        let embedding_time = embedding_start.elapsed();

        tracing::info!("Embedding generation completed: {} embeddings created", 
                      embedding_results.len());

        // Step 3: Store in vector database
        let storage_start = std::time::Instant::now();
        let stored_count = if self.config.vector_db.enable_storage && !embedding_results.is_empty() {
            self.vector_db.store_embeddings(embedding_results.clone()).await?;
            embedding_results.len()
        } else {
            0
        };
        let storage_time = storage_start.elapsed();

        tracing::info!("Vector storage completed: {} entries stored", stored_count);

        Ok(ProcessingStats {
            total_entries,
            validated_entries: validated_entries.len(),
            embedded_entries: embedding_results.len(),
            stored_entries: stored_count,
            validation_time_ms: validation_time.as_millis() as u64,
            embedding_time_ms: embedding_time.as_millis() as u64,
            storage_time_ms: storage_time.as_millis() as u64,
        })
    }

    /// Process entries with feedback generation for prompt improvement
    pub async fn process_entries_with_feedback(
        &self,
        entries: Vec<DatasetEntry>,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<(ProcessingStats, ValidationFeedback)> {
        if !self.config.enable_knowledge_base {
            tracing::info!("Knowledge base processing is disabled, skipping");
            let empty_feedback = ValidationFeedback {
                common_issues: vec![],
                improvement_suggestions: vec![],
                quality_patterns: vec![],
                avoid_patterns: vec![],
                batch_summary: "Knowledge base disabled".to_string(),
            };
            return Ok((ProcessingStats {
                total_entries: entries.len(),
                validated_entries: 0,
                embedded_entries: 0,
                stored_entries: 0,
                validation_time_ms: 0,
                embedding_time_ms: 0,
                storage_time_ms: 0,
            }, empty_feedback));
        }

        let total_entries = entries.len();
        tracing::info!("Processing {} entries through knowledge base pipeline with feedback", total_entries);

        // Step 1: Quality validation with feedback generation
        let validation_start = std::time::Instant::now();
        let (validated_entries, feedback) = if self.config.validation.enable_validation {
            self.validator.validate_entries_with_feedback(entries, use_case, format).await?
        } else {
            // Skip validation, convert entries to validated format
            let validated_entries: Vec<ValidatedEntry> = entries.into_iter().map(|entry| {
                ValidatedEntry {
                    entry,
                    quality_score: crate::quality_validator::QualityScore {
                        overall_score: 1.0,
                        relevance_score: 1.0,
                        coherence_score: 1.0,
                        completeness_score: 1.0,
                        format_compliance_score: 1.0,
                        issues: vec![],
                        tags: vec!["unvalidated".to_string()],
                    },
                    metadata: crate::quality_validator::EntryMetadata {
                        use_case: use_case.to_string(),
                        dataset_format: format.clone(),
                        content_hash: format!("unvalidated_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()),
                        validation_timestamp: chrono::Utc::now().timestamp(),
                        embedding_id: None,
                    },
                }
            }).collect();

            let empty_feedback = ValidationFeedback {
                common_issues: vec![],
                improvement_suggestions: vec![],
                quality_patterns: vec![],
                avoid_patterns: vec![],
                batch_summary: "Validation disabled - no feedback available".to_string(),
            };

            (validated_entries, empty_feedback)
        };
        let validation_time = validation_start.elapsed();

        let validated_count = validated_entries.len();
        tracing::info!("Validation completed: {}/{} entries passed", validated_count, total_entries);

        // Step 2: Generate embeddings
        let embedding_start = std::time::Instant::now();
        let embedding_results = if self.config.embedding.enable_embeddings && !validated_entries.is_empty() {
            self.embedding_service.embed_entries(&validated_entries).await?
        } else {
            vec![]
        };
        let embedding_time = embedding_start.elapsed();

        let embedded_count = embedding_results.len();
        tracing::info!("Embedding completed: {} entries embedded", embedded_count);

        // Step 3: Store in vector database
        let storage_start = std::time::Instant::now();
        let stored_count = if self.config.vector_db.enable_storage && !embedding_results.is_empty() {
            self.vector_db.store_embeddings(embedding_results).await?;
            embedded_count
        } else {
            0
        };
        let storage_time = storage_start.elapsed();

        tracing::info!("Storage completed: {} entries stored", stored_count);

        let stats = ProcessingStats {
            total_entries,
            validated_entries: validated_count,
            embedded_entries: embedded_count,
            stored_entries: stored_count,
            validation_time_ms: validation_time.as_millis() as u64,
            embedding_time_ms: embedding_time.as_millis() as u64,
            storage_time_ms: storage_time.as_millis() as u64,
        };

        Ok((stats, feedback))
    }

    /// Search the knowledge base for similar entries
    pub async fn search_knowledge_base(&self, query: QueryRequest) -> Result<Vec<SearchResult>> {
        if !self.config.enable_knowledge_base || !self.config.vector_db.enable_storage {
            return Ok(vec![]);
        }

        tracing::info!("Searching knowledge base for: {}", query.query_text);
        self.vector_db.search_similar(query).await
    }

    /// Get information about all collections in the knowledge base
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        if !self.config.enable_knowledge_base || !self.config.vector_db.enable_storage {
            return Ok(vec![]);
        }

        self.vector_db.list_collections().await
    }

    /// Get knowledge base statistics
    pub async fn get_stats(&self) -> Result<KnowledgeBaseStats> {
        if !self.config.enable_knowledge_base {
            return Ok(KnowledgeBaseStats::default());
        }

        let collections = self.list_collections().await?;
        
        let total_entries: usize = collections.iter().map(|c| c.entry_count).sum();
        let unique_use_cases: std::collections::HashSet<String> = collections.iter()
            .map(|c| c.use_case.clone())
            .collect();
        let unique_formats: std::collections::HashSet<DatasetFormat> = collections.iter()
            .map(|c| c.dataset_format.clone())
            .collect();

        Ok(KnowledgeBaseStats {
            total_collections: collections.len(),
            total_entries,
            unique_use_cases: unique_use_cases.len(),
            unique_formats: unique_formats.len(),
            oldest_entry_timestamp: collections.iter().map(|c| c.created_at).min(),
            newest_entry_timestamp: collections.iter().map(|c| c.last_updated).max(),
            collections,
        })
    }

    /// Find similar examples for a given use case and format
    pub async fn find_similar_examples(
        &self,
        use_case: &str,
        format: &DatasetFormat,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query = QueryRequest {
            query_text: query_text.to_string(),
            use_case_filter: Some(use_case.to_string()),
            format_filter: Some(format.clone()),
            min_quality_score: Some(0.8), // Only high-quality examples
            limit,
        };

        self.search_knowledge_base(query).await
    }

    /// Get recommended improvements based on knowledge base analysis
    pub async fn get_improvement_suggestions(
        &self,
        current_entries: &[DatasetEntry],
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<Vec<ImprovementSuggestion>> {
        let mut suggestions = Vec::new();

        // Find similar high-quality examples
        let sample_text = if let Some(entry) = current_entries.first() {
            serde_json::to_string(&entry.data).unwrap_or_default()
        } else {
            use_case.to_string()
        };

        let similar_examples = self.find_similar_examples(
            use_case,
            format,
            &sample_text,
            5,
        ).await?;

        if similar_examples.is_empty() {
            suggestions.push(ImprovementSuggestion {
                suggestion_type: "new_use_case".to_string(),
                description: format!("This appears to be a new use case: '{}'. Consider generating more examples to build up the knowledge base.", use_case),
                confidence: 0.8,
                examples: vec![],
            });
        } else {
            // Analyze quality patterns
            let avg_quality: f32 = similar_examples.iter()
                .filter_map(|r| r.metadata.get("overall_score")?.as_f64())
                .map(|s| s as f32)
                .collect::<Vec<_>>()
                .iter()
                .sum::<f32>() / similar_examples.len() as f32;

            if avg_quality > 0.9 {
                suggestions.push(ImprovementSuggestion {
                    suggestion_type: "quality_benchmark".to_string(),
                    description: format!("High-quality examples exist for this use case (avg score: {:.2}). Use these as references for improvement.", avg_quality),
                    confidence: 0.9,
                    examples: similar_examples.clone().into_iter().take(3).collect(),
                });
            }

            // Check for common tags/patterns
            let all_tags: Vec<String> = similar_examples.iter()
                .flat_map(|r| {
                    r.metadata.get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect::<Vec<_>>())
                        .unwrap_or_default()
                })
                .collect();

            let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for tag in all_tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }

            if let Some((most_common_tag, count)) = tag_counts.iter().max_by_key(|(_, &count)| count) {
                if *count >= 3 {
                    suggestions.push(ImprovementSuggestion {
                        suggestion_type: "pattern_recommendation".to_string(),
                        description: format!("Consider focusing on '{}' content - this appears {} times in similar high-quality examples.", most_common_tag, count),
                        confidence: 0.7,
                        examples: vec![],
                    });
                }
            }
        }

        Ok(suggestions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseStats {
    pub total_collections: usize,
    pub total_entries: usize,
    pub unique_use_cases: usize,
    pub unique_formats: usize,
    pub oldest_entry_timestamp: Option<i64>,
    pub newest_entry_timestamp: Option<i64>,
    pub collections: Vec<CollectionInfo>,
}

impl Default for KnowledgeBaseStats {
    fn default() -> Self {
        Self {
            total_collections: 0,
            total_entries: 0,
            unique_use_cases: 0,
            unique_formats: 0,
            oldest_entry_timestamp: None,
            newest_entry_timestamp: None,
            collections: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub confidence: f32,
    pub examples: Vec<SearchResult>,
}
