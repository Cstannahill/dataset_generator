use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::types::{DatasetEntry, DatasetFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall_score: f32,
    pub relevance_score: f32,
    pub coherence_score: f32,
    pub completeness_score: f32,
    pub format_compliance_score: f32,
    pub issues: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFeedback {
    pub common_issues: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub quality_patterns: Vec<String>,
    pub avoid_patterns: Vec<String>,
    pub batch_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedEntry {
    pub entry: DatasetEntry,
    pub quality_score: QualityScore,
    pub metadata: EntryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub use_case: String,
    pub dataset_format: DatasetFormat,
    pub content_hash: String,
    pub validation_timestamp: i64,
    pub embedding_id: Option<String>,
}

pub struct QualityValidator {
    client: reqwest::Client,
    model_name: String,
}

impl QualityValidator {
    pub fn new(model_name: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            model_name: model_name.unwrap_or_else(|| "llama3.2:3b".to_string()),
        }
    }

    /// Validate a batch of entries using local LLM
    pub async fn validate_entries(
        &self,
        entries: Vec<DatasetEntry>,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<Vec<ValidatedEntry>> {
        let total_entries = entries.len();
        let mut validated_entries = Vec::new();

        for entry in entries {
            match self.validate_single_entry(&entry, use_case, format).await {
                Ok(validated_entry) => {
                    // Only include high-quality entries (score > 0.7)
                    if validated_entry.quality_score.overall_score > 0.7 {
                        validated_entries.push(validated_entry);
                    } else {
                        tracing::info!(
                            "Filtering out low-quality entry with score: {:.2}",
                            validated_entry.quality_score.overall_score
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to validate entry: {}", e);
                    // Continue with other entries
                }
            }
        }

        tracing::info!(
            "Quality validation completed: {}/{} entries passed",
            validated_entries.len(),
            total_entries
        );

        Ok(validated_entries)
    }

    /// Validate entries and generate feedback for prompt improvement
    pub async fn validate_entries_with_feedback(
        &self,
        entries: Vec<DatasetEntry>,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<(Vec<ValidatedEntry>, ValidationFeedback)> {
        let total_entries = entries.len();
        let mut validated_entries = Vec::new();
        let mut all_quality_scores = Vec::new();

        for entry in entries {
            match self.validate_single_entry(&entry, use_case, format).await {
                Ok(validated_entry) => {
                    all_quality_scores.push(validated_entry.quality_score.clone());
                    
                    // Only include high-quality entries (score > 0.7)
                    if validated_entry.quality_score.overall_score > 0.7 {
                        validated_entries.push(validated_entry);
                    } else {
                        tracing::info!(
                            "Filtering out low-quality entry with score: {:.2}",
                            validated_entry.quality_score.overall_score
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to validate entry: {}", e);
                    // Continue with other entries
                }
            }
        }

        // Generate feedback based on validation results
        let feedback = self.generate_validation_feedback(&all_quality_scores, use_case, format).await?;

        tracing::info!(
            "Quality validation completed: {}/{} entries passed",
            validated_entries.len(),
            total_entries
        );

        Ok((validated_entries, feedback))
    }

    /// Generate feedback based on validation patterns to improve future prompts
    pub async fn generate_validation_feedback(
        &self,
        quality_scores: &[QualityScore],
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<ValidationFeedback> {
        if quality_scores.is_empty() {
            return Ok(ValidationFeedback {
                common_issues: vec![],
                improvement_suggestions: vec![],
                quality_patterns: vec![],
                avoid_patterns: vec![],
                batch_summary: "No entries to analyze".to_string(),
            });
        }

        let feedback_prompt = self.create_feedback_prompt(quality_scores, use_case, format);
        let llm_response = self.query_ollama(&feedback_prompt).await?;
        let feedback = self.parse_feedback_response(&llm_response)?;

        Ok(feedback)
    }

    /// Generate dynamic prompt improvements based on validation feedback
    pub fn generate_dynamic_prompt_improvements(&self, feedback: &ValidationFeedback) -> String {
        if feedback.improvement_suggestions.is_empty() && feedback.avoid_patterns.is_empty() {
            return String::new();
        }

        let mut improvements = Vec::new();

        if !feedback.avoid_patterns.is_empty() {
            improvements.push(format!(
                "AVOID THE FOLLOWING PATTERNS:\n{}",
                feedback.avoid_patterns.iter()
                    .map(|pattern| format!("- {}", pattern))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !feedback.improvement_suggestions.is_empty() {
            improvements.push(format!(
                "FOCUS MORE ON:\n{}",
                feedback.improvement_suggestions.iter()
                    .map(|suggestion| format!("- {}", suggestion))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !feedback.quality_patterns.is_empty() {
            improvements.push(format!(
                "SUCCESSFUL PATTERNS TO FOLLOW:\n{}",
                feedback.quality_patterns.iter()
                    .map(|pattern| format!("- {}", pattern))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if improvements.is_empty() {
            return String::new();
        }

        format!(
            "\n\n--- QUALITY IMPROVEMENT GUIDELINES ---\n{}\n\nBase your generation on these insights from recent quality analysis.\n",
            improvements.join("\n\n")
        )
    }

    /// Validate a single entry using the local LLM
    async fn validate_single_entry(
        &self,
        entry: &DatasetEntry,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<ValidatedEntry> {
        let content_hash = self.calculate_content_hash(entry);
        
        let validation_prompt = self.create_validation_prompt(entry, use_case, format);
        
        let llm_response = self.query_ollama(&validation_prompt).await?;
        let quality_score = self.parse_quality_response(&llm_response)?;

        let metadata = EntryMetadata {
            use_case: use_case.to_string(),
            dataset_format: format.clone(),
            content_hash,
            validation_timestamp: chrono::Utc::now().timestamp(),
            embedding_id: None,
        };

        Ok(ValidatedEntry {
            entry: entry.clone(),
            quality_score,
            metadata,
        })
    }

    /// Create a validation prompt for the local LLM
    fn create_validation_prompt(&self, entry: &DatasetEntry, use_case: &str, format: &DatasetFormat) -> String {
        let format_description = match format {
            DatasetFormat::Alpaca => "instruction-input-output format for supervised fine-tuning",
            DatasetFormat::Conversation => "conversation format with role-based messages",
            DatasetFormat::ChainOfThought => "step-by-step reasoning format",
            DatasetFormat::PreferenceRanking => "preference ranking with chosen/rejected pairs",
            DatasetFormat::FunctionCall => "function calling format with tools",
            DatasetFormat::MultiRoundDialogue => "multi-turn dialogue format",
            DatasetFormat::CodeTask => "code generation and execution format",
            DatasetFormat::Reflection => "self-reflection and correction format",
            DatasetFormat::RetrievalEmbedding => "query-passage pairs for retrieval training",
            DatasetFormat::Reranking => "pairwise reranking format with query, positive, and negative documents",
        };

        format!(
            r#"You are an expert AI trainer evaluating dataset quality. Please assess this training example for use case: "{}"

Dataset format: {}
Entry data: {}

Evaluate the entry on these criteria (score 0.0-1.0 for each):
1. RELEVANCE: How well does this align with the use case "{}"?
2. COHERENCE: Is the content logical, clear, and well-structured?
3. COMPLETENESS: Are all required fields present and substantive?
4. FORMAT_COMPLIANCE: Does it correctly follow the {} format?

Also identify:
- ISSUES: Any problems, inconsistencies, or areas for improvement
- TAGS: Relevant content categories, difficulty level, topic areas

Respond in this exact JSON format:
{{
  "overall_score": 0.85,
  "relevance_score": 0.9,
  "coherence_score": 0.8,
  "completeness_score": 0.9,
  "format_compliance_score": 0.8,
  "issues": ["minor grammatical error", "could be more specific"],
  "tags": ["beginner", "mathematics", "problem-solving"]
}}

Be strict but fair. Only give high scores (>0.8) to truly excellent examples."#,
            use_case,
            format_description,
            serde_json::to_string_pretty(&entry.data).unwrap_or_else(|_| "Invalid JSON".to_string()),
            use_case,
            format_description
        )
    }

    /// Query the local Ollama LLM
    async fn query_ollama(&self, prompt: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "model": self.model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "top_p": 0.9,
                "top_k": 40
            }
        });

        let response = self.client
            .post("http://localhost:11434/api/generate")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["response"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Ollama API error: {}", error_text))
        }
    }

    /// Parse the LLM's quality assessment response
    fn parse_quality_response(&self, response: &str) -> Result<QualityScore> {
        // Try to extract JSON from the response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_text = &response[json_start..json_end];

        match serde_json::from_str::<QualityScore>(json_text) {
            Ok(score) => Ok(score),
            Err(e) => {
                tracing::warn!("Failed to parse quality response: {}", e);
                tracing::debug!("Response text: {}", response);
                
                // Fallback to basic scoring if parsing fails
                Ok(QualityScore {
                    overall_score: 0.5,
                    relevance_score: 0.5,
                    coherence_score: 0.5,
                    completeness_score: 0.5,
                    format_compliance_score: 0.5,
                    issues: vec!["Failed to parse validation response".to_string()],
                    tags: vec!["unvalidated".to_string()],
                })
            }
        }
    }

    /// Calculate content hash for deduplication
    fn calculate_content_hash(&self, entry: &DatasetEntry) -> String {
        use sha2::{Sha256, Digest};
        use base64::{Engine as _, engine::general_purpose};
        
        let content = serde_json::to_string(&entry.data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        general_purpose::STANDARD.encode(hash)
    }

    /// Create a feedback prompt to analyze validation patterns
    fn create_feedback_prompt(&self, quality_scores: &[QualityScore], use_case: &str, format: &DatasetFormat) -> String {
        let total_entries = quality_scores.len();
        let high_quality_count = quality_scores.iter().filter(|s| s.overall_score > 0.8).count();
        let medium_quality_count = quality_scores.iter().filter(|s| s.overall_score > 0.6 && s.overall_score <= 0.8).count();
        let low_quality_count = quality_scores.iter().filter(|s| s.overall_score <= 0.6).count();

        let common_issues: Vec<String> = quality_scores.iter()
            .flat_map(|s| &s.issues)
            .fold(std::collections::HashMap::new(), |mut acc, issue| {
                *acc.entry(issue.clone()).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .filter(|(_, count)| *count >= 2) // Issues appearing in at least 2 entries
            .map(|(issue, count)| format!("{} (appeared {} times)", issue, count))
            .collect();

        let avg_relevance = quality_scores.iter().map(|s| s.relevance_score).sum::<f32>() / total_entries as f32;
        let avg_coherence = quality_scores.iter().map(|s| s.coherence_score).sum::<f32>() / total_entries as f32;
        let avg_completeness = quality_scores.iter().map(|s| s.completeness_score).sum::<f32>() / total_entries as f32;

        format!(
            r#"You are an expert AI trainer analyzing a batch of dataset validation results. Based on the patterns you see, provide actionable feedback to improve future dataset generation.

VALIDATION SUMMARY:
- Use case: "{}"
- Dataset format: {:?}
- Total entries analyzed: {}
- High quality (>0.8): {}
- Medium quality (0.6-0.8): {}
- Low quality (<0.6): {}
- Average relevance score: {:.2}
- Average coherence score: {:.2}
- Average completeness score: {:.2}

COMMON ISSUES FOUND:
{}

Based on this analysis, provide specific feedback in this exact JSON format:
{{
  "common_issues": ["issue1", "issue2"],
  "improvement_suggestions": ["focus more on X", "ensure Y is always included"],
  "quality_patterns": ["successful pattern 1", "successful pattern 2"],
  "avoid_patterns": ["don't do X", "avoid Y pattern"],
  "batch_summary": "Brief summary of the overall quality and main insights"
}}

Be specific and actionable. Focus on patterns that would help the generation model create better training data."#,
            use_case,
            format,
            total_entries,
            high_quality_count,
            medium_quality_count,
            low_quality_count,
            avg_relevance,
            avg_coherence,
            avg_completeness,
            if common_issues.is_empty() { "No recurring issues found".to_string() } else { common_issues.join("\n") }
        )
    }

    /// Parse the LLM's feedback response
    fn parse_feedback_response(&self, response: &str) -> Result<ValidationFeedback> {
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_text = &response[json_start..json_end];

        match serde_json::from_str::<ValidationFeedback>(json_text) {
            Ok(feedback) => Ok(feedback),
            Err(e) => {
                tracing::warn!("Failed to parse feedback response: {}", e);
                tracing::debug!("Response text: {}", response);
                
                // Fallback feedback if parsing fails
                Ok(ValidationFeedback {
                    common_issues: vec!["Failed to parse validation feedback".to_string()],
                    improvement_suggestions: vec!["Ensure clear instructions and examples".to_string()],
                    quality_patterns: vec![],
                    avoid_patterns: vec![],
                    batch_summary: "Unable to analyze feedback due to parsing error".to_string(),
                })
            }
        }
    }
}

/// Configuration for quality validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub model_name: String,
    pub min_quality_score: f32,
    pub enable_validation: bool,
    pub batch_size: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            model_name: "llama3.2:3b".to_string(),
            min_quality_score: 0.7,
            enable_validation: true,
            batch_size: 10,
        }
    }
}
