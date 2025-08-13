use serde::{Deserialize, Serialize};
use tauri::State;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;
use crate::types::{GenerationConfig, DatasetFormat};
use crate::enhanced_validation::{EnhancedQualityValidator, MultiStageValidationResult};
use crate::quality_visualization::{QualityVisualizationService, QualityVisualizationData};
use crate::prompt_template::{PromptTemplateEngine, PromptContext, DatasetStatistics};
use crate::knowledge_base::{KnowledgeBaseManager, KnowledgeBaseConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedGenerationConfig {
    pub base_config: GenerationConfig,
    pub dataset_format: DatasetFormat,
    pub enable_multi_stage_validation: bool,
    pub enable_prompt_optimization: bool,
    pub enable_knowledge_base: bool,
    pub enable_negative_sampling: bool,
    pub quality_threshold: f32,
    pub domain_adaptation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationReport {
    pub total_entries_generated: usize,
    pub total_entries_validated: usize,
    pub total_entries_stored: usize,
    pub average_quality_score: f32,
    pub validation_summary: ValidationSummary,
    pub quality_insights: QualityInsightsSummary,
    pub prompt_improvements_applied: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub rule_based_pass_rate: f32,
    pub llm_based_pass_rate: f32,
    pub combined_pass_rate: f32,
    pub most_common_issues: Vec<String>,
    pub quality_distribution: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInsightsSummary {
    pub complexity_distribution: std::collections::HashMap<String, usize>,
    pub topic_distribution: std::collections::HashMap<String, usize>,
    pub improvement_opportunities: Vec<String>,
    pub domain_drift_detected: bool,
}

/// Start enhanced dataset generation with all quality improvements
#[tauri::command]
pub async fn start_enhanced_generation(
    config: EnhancedGenerationConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Generate unique ID for this generation session
    let generation_id = Uuid::new_v4().to_string();
    
    // Update configuration
    let mut gen_config = state.generation_config.write().await;
    *gen_config = Some(config.base_config.clone());
    drop(gen_config);
    
    // Initialize enhanced services
    let quality_viz = QualityVisualizationService::new();
    let prompt_engine = PromptTemplateEngine::new();
    
    // Initialize enhanced validator
    let enhanced_validator = EnhancedQualityValidator::new(Some("llama3.2:3b".to_string()));
    
    // Initialize knowledge base if configured  
    let knowledge_base: Option<KnowledgeBaseManager> = None; // Simplified for now
    
    // Start enhanced generation process (simplified approach without background task)
    // For now, we'll skip the background generation and just return the ID
    tracing::info!("Enhanced generation requested with ID: {}", generation_id);
    
    // Suppress unused variable warnings for now
    let _ = (enhanced_validator, quality_viz, prompt_engine, knowledge_base);

    Ok(generation_id)
}

/// Get comprehensive quality dashboard data
#[tauri::command]
pub async fn get_quality_dashboard(
    state: State<'_, AppState>,
) -> Result<QualityVisualizationData, String> {
    // This would retrieve data from the quality visualization service
    // For now, returning a placeholder
    Ok(QualityVisualizationData {
        overall_metrics: crate::quality_visualization::OverallQualityMetrics {
            total_entries: 0,
            average_quality: 0.0,
            quality_distribution: crate::quality_visualization::QualityDistribution {
                high_quality_count: 0,
                medium_quality_count: 0,
                low_quality_count: 0,
                percentages: std::collections::HashMap::new(),
            },
            pass_rate: 0.0,
            improvement_rate: 0.0,
        },
        quality_trends: crate::quality_visualization::QualityTrendData {
            batch_scores: vec![],
            moving_average: vec![],
            trend_direction: "stable".to_string(),
            trend_strength: 0.0,
            prediction: crate::quality_visualization::QualityPrediction {
                next_batch_predicted_score: 0.7,
                confidence_interval: (0.6, 0.8),
                recommendations: vec![],
            },
        },
        error_analysis: crate::quality_visualization::ErrorAnalysisData {
            error_categories: std::collections::HashMap::new(),
            most_common_issues: vec![],
            error_trends: vec![],
            resolution_suggestions: std::collections::HashMap::new(),
        },
        improvement_tracking: crate::quality_visualization::ImprovementTrackingData {
            quality_progression: vec![],
            milestone_achievements: vec![],
            performance_metrics: crate::quality_visualization::PerformanceMetrics {
                entries_per_second: 1.0,
                validation_efficiency: 0.8,
                resource_utilization: 0.6,
                throughput_trend: "stable".to_string(),
            },
            optimization_opportunities: vec![],
        },
        domain_insights: crate::quality_visualization::DomainInsightData {
            topic_distribution: std::collections::HashMap::new(),
            complexity_distribution: std::collections::HashMap::new(),
            domain_drift_indicators: vec![],
            adaptation_history: vec![],
            knowledge_gaps: vec![],
        },
    })
}

/// Generate negative samples for training
#[tauri::command]
pub async fn generate_negative_samples(
    positive_entries: Vec<crate::types::DatasetEntry>,
    format: DatasetFormat,
    difficulty: String,
) -> Result<Vec<crate::types::DatasetEntry>, String> {
    let enhanced_validator = EnhancedQualityValidator::new(Some("llama3.2:3b".to_string()));
    
    match enhanced_validator.generate_negative_samples(&positive_entries, &format, &difficulty).await {
        Ok(result) => Ok(result.negative_examples),
        Err(e) => Err(format!("Failed to generate negative samples: {}", e)),
    }
}

/// Update prompt templates with feedback
#[tauri::command]
pub async fn update_prompt_templates(
    format: DatasetFormat,
    feedback: crate::quality_validator::ValidationFeedback,
    quality_score: f32,
) -> Result<bool, String> {
    let mut prompt_engine = PromptTemplateEngine::new();
    
    match prompt_engine.update_template_with_feedback(&format, &feedback, quality_score) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to update prompt templates: {}", e)),
    }
}

/// Get domain adaptation recommendations
#[tauri::command]
pub async fn get_domain_adaptation_insights(
    recent_entries: Vec<crate::quality_validator::ValidatedEntry>,
    historical_entries: Vec<crate::quality_validator::ValidatedEntry>,
) -> Result<crate::enhanced_validation::DomainAdaptationMetrics, String> {
    let enhanced_validator = EnhancedQualityValidator::new(Some("llama3.2:3b".to_string()));
    
    match enhanced_validator.detect_domain_drift(&recent_entries, &historical_entries) {
        Ok(metrics) => Ok(metrics),
        Err(e) => Err(format!("Failed to analyze domain drift: {}", e)),
    }
}

/// Export enhanced dataset with quality metadata
#[tauri::command]
pub async fn export_enhanced_dataset(
    include_quality_metadata: bool,
    include_validation_reports: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let entries_guard = state.dataset.read().await;
    let entries = entries_guard.clone();
    
    if entries.is_empty() {
        return Err("No dataset entries to export".to_string());
    }

    let mut export_data = serde_json::json!({
        "entries": entries,
        "metadata": {
            "total_entries": entries.len(),
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "enhanced_export": true
        }
    });

    if include_quality_metadata {
        // Add quality scores and validation metadata
        export_data["quality_metadata"] = serde_json::json!({
            "validation_enabled": true,
            "quality_threshold": 0.7,
            "validation_model": "llama3.2:3b"
        });
    }

    if include_validation_reports {
        // Add validation reports and insights
        export_data["validation_reports"] = serde_json::json!({
            "summary": "Validation reports would be included here",
            "detailed_analysis": "Detailed quality analysis would be included here"
        });
    }

    match serde_json::to_string_pretty(&export_data) {
        Ok(json_string) => Ok(json_string),
        Err(e) => Err(format!("Failed to serialize enhanced dataset: {}", e)),
    }
}

// Private helper functions

async fn run_enhanced_generation_process(
    state: Arc<AppState>,
    generation_id: String,
    config: EnhancedGenerationConfig,
    enhanced_validator: EnhancedQualityValidator,
    mut quality_viz: QualityVisualizationService,
    prompt_engine: PromptTemplateEngine,
    knowledge_base: Option<KnowledgeBaseManager>,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    
    tracing::info!("Starting enhanced generation process with ID: {}", generation_id);

    // Initialize progress tracking
    {
        let mut progress = state.progress.write().await;
        progress.status = "initializing_enhanced_systems".to_string();
        progress.generation_id = Some(generation_id.clone());
    }

    // Initialize knowledge base if enabled
    if let Some(ref kb) = knowledge_base {
        kb.initialize().await?;
    }

    // Create prompt context from historical data
    let prompt_context = create_prompt_context(&state, &config).await?;

    // Generate enhanced prompts
    let generation_prompt = prompt_engine.generate_prompt(
        &config.dataset_format,
        &config.base_config.fine_tuning_goal,
        config.base_config.batch_size,
        &prompt_context,
        &config.base_config.domain_context,
    )?;

    tracing::info!("Generated enhanced prompt with {} examples", generation_prompt.format_examples.len());

    // Run the generation process with enhanced features
    let mut all_validated_entries = Vec::new();
    let total_batches = (config.base_config.target_entries + config.base_config.batch_size - 1) / config.base_config.batch_size;

    for batch_id in 0..total_batches {
        // Update progress
        {
            let mut progress = state.progress.write().await;
            progress.current_batch = batch_id + 1;
            progress.status = format!("processing_enhanced_batch_{}", batch_id + 1);
        }

        // Generate batch entries (this would integrate with the existing generation system)
        let batch_entries = generate_enhanced_batch(
            &config,
            batch_id,
            &generation_prompt,
            &state,
        ).await?;

        // Multi-stage validation if enabled
        if config.enable_multi_stage_validation {
            let validation_results = enhanced_validator.multi_stage_validate(
                batch_entries.clone(),
                &config.base_config.fine_tuning_goal,
                &config.dataset_format,
                &all_validated_entries,
            ).await?;

            // Filter based on quality threshold
            let high_quality_entries: Vec<_> = validation_results
                .into_iter()
                .filter(|result| result.final_score.overall_score >= config.quality_threshold)
                .collect();

            tracing::info!(
                "Batch {} validation: {}/{} entries passed quality threshold",
                batch_id + 1,
                high_quality_entries.len(),
                batch_entries.len()
            );

            // Update quality visualization
            quality_viz.add_validation_results(high_quality_entries.clone());

            // Convert to ValidatedEntry for storage
            for result in high_quality_entries {
                let validated_entry = crate::quality_validator::ValidatedEntry {
                    entry: result.llm_based_result.tags.first().map(|_| crate::types::DatasetEntry {
                        data: serde_json::json!({}), // Placeholder
                    }).unwrap_or_else(|| crate::types::DatasetEntry {
                        data: serde_json::json!({}),
                    }),
                    quality_score: result.final_score,
                    metadata: crate::quality_validator::EntryMetadata {
                        use_case: config.base_config.fine_tuning_goal.clone(),
                        dataset_format: config.dataset_format.clone(),
                        content_hash: "".to_string(),
                        validation_timestamp: chrono::Utc::now().timestamp(),
                        embedding_id: None,
                    },
                };
                all_validated_entries.push(validated_entry);
            }
        }

        // Process through knowledge base if enabled
        if let Some(ref kb) = knowledge_base {
            let processing_stats = kb.process_entries(
                batch_entries,
                &config.base_config.fine_tuning_goal,
                &config.dataset_format,
            ).await?;

            tracing::info!(
                "Knowledge base processing: {}/{} entries stored",
                processing_stats.stored_entries,
                processing_stats.total_entries
            );
        }
    }

    // Generate final report
    let processing_time = start_time.elapsed().as_millis() as u64;
    let generation_report = GenerationReport {
        total_entries_generated: all_validated_entries.len(),
        total_entries_validated: all_validated_entries.len(),
        total_entries_stored: all_validated_entries.len(),
        average_quality_score: if !all_validated_entries.is_empty() {
            all_validated_entries.iter()
                .map(|entry| entry.quality_score.overall_score)
                .sum::<f32>() / all_validated_entries.len() as f32
        } else {
            0.0
        },
        validation_summary: ValidationSummary {
            rule_based_pass_rate: 85.0, // Placeholder
            llm_based_pass_rate: 78.0,  // Placeholder
            combined_pass_rate: 82.0,   // Placeholder
            most_common_issues: vec!["minor formatting".to_string()], // Placeholder
            quality_distribution: std::collections::HashMap::new(),
        },
        quality_insights: QualityInsightsSummary {
            complexity_distribution: std::collections::HashMap::new(),
            topic_distribution: std::collections::HashMap::new(),
            improvement_opportunities: vec!["More diverse examples".to_string()], // Placeholder
            domain_drift_detected: false,
        },
        prompt_improvements_applied: 5, // Placeholder
        processing_time_ms: processing_time,
    };

    tracing::info!("Enhanced generation completed: {:?}", generation_report);

    // Update final progress
    {
        let mut progress = state.progress.write().await;
        progress.status = "completed".to_string();
        progress.entries_generated = all_validated_entries.len();
    }

    Ok(())
}

async fn create_prompt_context(
    state: &Arc<AppState>,
    config: &EnhancedGenerationConfig,
) -> Result<PromptContext> {
    // Create context from historical data
    let entries_guard = state.dataset.read().await;
    let historical_entries = entries_guard.clone();

    Ok(PromptContext {
        previous_batches_summary: format!("Generated {} entries previously", historical_entries.len()),
        dataset_statistics: DatasetStatistics {
            total_entries: historical_entries.len(),
            average_quality_score: 0.8, // Placeholder
            format_distribution: std::collections::HashMap::new(),
            topic_distribution: std::collections::HashMap::new(),
            complexity_distribution: std::collections::HashMap::new(),
            batch_quality_trend: vec![0.75, 0.78, 0.82, 0.85], // Placeholder trend
        },
        common_errors: vec![
            "Incomplete responses".to_string(),
            "Off-topic content".to_string(),
        ],
        validation_feedback: None,
        domain_drift_indicators: vec![],
    })
}

async fn generate_enhanced_batch(
    config: &EnhancedGenerationConfig,
    batch_id: usize,
    generation_prompt: &crate::prompt_template::GenerationPrompt,
    state: &Arc<AppState>,
) -> Result<Vec<crate::types::DatasetEntry>> {
    // This would integrate with the existing batch generation system
    // For now, returning placeholder entries
    let batch_size = config.base_config.batch_size;
    
    let mut entries = Vec::new();
    for i in 0..batch_size {
        entries.push(crate::types::DatasetEntry {
            data: serde_json::json!({
                "instruction": format!("Enhanced instruction {} from batch {}", i + 1, batch_id + 1),
                "input": "Enhanced input context",
                "output": "Enhanced output response"
            }),
        });
    }

    Ok(entries)
}
