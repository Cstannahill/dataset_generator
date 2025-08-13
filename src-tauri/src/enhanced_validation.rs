use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::types::{DatasetEntry, DatasetFormat};
use crate::quality_validator::{QualityScore, ValidatedEntry, ValidationFeedback};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStageValidationResult {
    pub rule_based_result: RuleBasedValidationResult,
    pub llm_based_result: QualityScore,
    pub final_score: QualityScore,
    pub auto_tags: Vec<String>,
    pub quality_insights: QualityInsights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleBasedValidationResult {
    pub json_validity: bool,
    pub required_fields_present: bool,
    pub field_completeness: HashMap<String, bool>,
    pub format_compliance: bool,
    pub content_length_check: bool,
    pub issues: Vec<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInsights {
    pub complexity_level: String,
    pub topic_categories: Vec<String>,
    pub error_types: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub content_analysis: ContentAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub word_count: usize,
    pub sentence_count: usize,
    pub readability_score: f32,
    pub technical_terms_count: usize,
    pub sentiment_score: f32,
    pub diversity_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAdaptationMetrics {
    pub domain_drift_score: f32,
    pub new_patterns: Vec<String>,
    pub obsolete_patterns: Vec<String>,
    pub adaptation_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeSamplingResult {
    pub negative_examples: Vec<DatasetEntry>,
    pub sampling_strategy: String,
    pub difficulty_level: String,
}

pub struct EnhancedQualityValidator {
    rule_validator: RuleBasedValidator,
    llm_validator: LLMValidator,
    auto_tagger: AutomaticTagger,
    domain_adapter: DomainAdapter,
    negative_sampler: NegativeSampler,
}

impl EnhancedQualityValidator {
    pub fn new(model_name: Option<String>) -> Self {
        Self {
            rule_validator: RuleBasedValidator::new(),
            llm_validator: LLMValidator::new(model_name),
            auto_tagger: AutomaticTagger::new(),
            domain_adapter: DomainAdapter::new(),
            negative_sampler: NegativeSampler::new(),
        }
    }

    /// Perform multi-stage validation with rule-based and LLM-based checks
    pub async fn multi_stage_validate(
        &self,
        entries: Vec<DatasetEntry>,
        use_case: &str,
        format: &DatasetFormat,
        historical_data: &[ValidatedEntry],
    ) -> Result<Vec<MultiStageValidationResult>> {
        let mut results = Vec::new();

        for entry in entries {
            // Stage 1: Rule-based validation
            let rule_result = self.rule_validator.validate(&entry, format)?;

            // Stage 2: LLM-based validation (only if rule-based passes basic checks)
            let llm_result = if rule_result.score > 0.5 {
                self.llm_validator.validate(&entry, use_case, format).await?
            } else {
                QualityScore {
                    overall_score: rule_result.score,
                    relevance_score: 0.0,
                    coherence_score: 0.0,
                    completeness_score: rule_result.score,
                    format_compliance_score: if rule_result.format_compliance { 1.0 } else { 0.0 },
                    issues: rule_result.issues.clone(),
                    tags: vec!["failed_rule_validation".to_string()],
                }
            };

            // Stage 3: Combine scores and generate final assessment
            let final_score = self.combine_validation_scores(&rule_result, &llm_result);

            // Stage 4: Automatic tagging
            let auto_tags = self.auto_tagger.generate_tags(&entry, &final_score, format).await?;

            // Stage 5: Generate quality insights
            let quality_insights = self.generate_quality_insights(&entry, &final_score, historical_data)?;

            results.push(MultiStageValidationResult {
                rule_based_result: rule_result,
                llm_based_result: llm_result,
                final_score,
                auto_tags,
                quality_insights,
            });
        }

        Ok(results)
    }

    /// Detect domain drift and adapt validation rules
    pub fn detect_domain_drift(
        &self,
        recent_entries: &[ValidatedEntry],
        historical_entries: &[ValidatedEntry],
    ) -> Result<DomainAdaptationMetrics> {
        self.domain_adapter.analyze_drift(recent_entries, historical_entries)
    }

    /// Generate negative samples for training
    pub async fn generate_negative_samples(
        &self,
        positive_entries: &[DatasetEntry],
        format: &DatasetFormat,
        difficulty: &str,
    ) -> Result<NegativeSamplingResult> {
        self.negative_sampler.generate_samples(positive_entries, format, difficulty).await
    }

    /// Update validation rules based on feedback
    pub fn update_validation_rules(&mut self, feedback: &ValidationFeedback) -> Result<()> {
        self.rule_validator.update_rules(feedback)?;
        self.domain_adapter.update_patterns(feedback)?;
        Ok(())
    }

    // Private helper methods

    fn combine_validation_scores(
        &self,
        rule_result: &RuleBasedValidationResult,
        llm_result: &QualityScore,
    ) -> QualityScore {
        // Weight rule-based and LLM-based scores
        let rule_weight = 0.3;
        let llm_weight = 0.7;

        let combined_overall = (rule_result.score * rule_weight) + (llm_result.overall_score * llm_weight);

        // Combine issues and tags
        let mut combined_issues = rule_result.issues.clone();
        combined_issues.extend(llm_result.issues.clone());

        let mut combined_tags = llm_result.tags.clone();
        if !rule_result.format_compliance {
            combined_tags.push("format_non_compliance".to_string());
        }
        if !rule_result.json_validity {
            combined_tags.push("json_invalid".to_string());
        }

        QualityScore {
            overall_score: combined_overall,
            relevance_score: llm_result.relevance_score,
            coherence_score: llm_result.coherence_score,
            completeness_score: llm_result.completeness_score,
            format_compliance_score: if rule_result.format_compliance { 
                llm_result.format_compliance_score 
            } else { 
                0.0 
            },
            issues: combined_issues,
            tags: combined_tags,
        }
    }

    fn generate_quality_insights(
        &self,
        entry: &DatasetEntry,
        quality_score: &QualityScore,
        historical_data: &[ValidatedEntry],
    ) -> Result<QualityInsights> {
        let content_analysis = self.analyze_content(entry)?;
        
        let complexity_level = self.determine_complexity_level(&content_analysis);
        let topic_categories = self.extract_topic_categories(entry)?;
        let error_types = self.categorize_errors(&quality_score.issues);
        let improvement_suggestions = self.generate_improvement_suggestions(quality_score, &content_analysis);

        Ok(QualityInsights {
            complexity_level,
            topic_categories,
            error_types,
            improvement_suggestions,
            content_analysis,
        })
    }

    fn analyze_content(&self, entry: &DatasetEntry) -> Result<ContentAnalysis> {
        let content = serde_json::to_string(&entry.data).unwrap_or_default();
        
        let word_count = content.split_whitespace().count();
        let sentence_count = content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
        
        // Simple readability approximation (Flesch-like)
        let avg_sentence_length = if sentence_count > 0 { word_count as f32 / sentence_count as f32 } else { 0.0 };
        let readability_score = 100.0 - (avg_sentence_length * 1.015);

        // Count technical terms (simplified)
        let technical_terms = ["algorithm", "implementation", "optimization", "framework", "architecture"];
        let technical_terms_count = technical_terms.iter()
            .map(|term| content.to_lowercase().matches(term).count())
            .sum();

        // Simple sentiment analysis (positive words vs negative words)
        let positive_words = ["good", "excellent", "effective", "successful", "optimal"];
        let negative_words = ["bad", "poor", "ineffective", "failed", "problematic"];
        
        let positive_count = positive_words.iter()
            .map(|word| content.to_lowercase().matches(word).count())
            .sum::<usize>() as f32;
        let negative_count = negative_words.iter()
            .map(|word| content.to_lowercase().matches(word).count())
            .sum::<usize>() as f32;
        
        let sentiment_score = if positive_count + negative_count > 0.0 {
            (positive_count - negative_count) / (positive_count + negative_count)
        } else {
            0.0
        };

        let diversity_indicators = self.calculate_diversity_indicators(&content);

        Ok(ContentAnalysis {
            word_count,
            sentence_count,
            readability_score,
            technical_terms_count,
            sentiment_score,
            diversity_indicators,
        })
    }

    fn determine_complexity_level(&self, analysis: &ContentAnalysis) -> String {
        if analysis.technical_terms_count > 5 || analysis.word_count > 200 {
            "advanced".to_string()
        } else if analysis.technical_terms_count > 2 || analysis.word_count > 100 {
            "intermediate".to_string()
        } else {
            "beginner".to_string()
        }
    }

    fn extract_topic_categories(&self, entry: &DatasetEntry) -> Result<Vec<String>> {
        let content = serde_json::to_string(&entry.data).unwrap_or_default().to_lowercase();
        
        let categories = [
            ("technology", vec!["software", "programming", "computer", "digital", "algorithm"]),
            ("science", vec!["research", "experiment", "theory", "analysis", "study"]),
            ("business", vec!["company", "market", "customer", "revenue", "strategy"]),
            ("education", vec!["learning", "student", "teaching", "curriculum", "academic"]),
            ("health", vec!["medical", "health", "treatment", "patient", "clinical"]),
            ("finance", vec!["money", "investment", "financial", "economic", "banking"]),
        ];

        let mut detected_categories = Vec::new();
        for (category, keywords) in &categories {
            if keywords.iter().any(|keyword| content.contains(keyword)) {
                detected_categories.push(category.to_string());
            }
        }

        if detected_categories.is_empty() {
            detected_categories.push("general".to_string());
        }

        Ok(detected_categories)
    }

    fn categorize_errors(&self, issues: &[String]) -> Vec<String> {
        let mut error_types = Vec::new();
        
        for issue in issues {
            let issue_lower = issue.to_lowercase();
            if issue_lower.contains("grammar") || issue_lower.contains("spelling") {
                error_types.push("language_quality".to_string());
            } else if issue_lower.contains("format") || issue_lower.contains("structure") {
                error_types.push("format_compliance".to_string());
            } else if issue_lower.contains("incomplete") || issue_lower.contains("missing") {
                error_types.push("completeness".to_string());
            } else if issue_lower.contains("irrelevant") || issue_lower.contains("off-topic") {
                error_types.push("relevance".to_string());
            } else {
                error_types.push("other".to_string());
            }
        }

        error_types.sort();
        error_types.dedup();
        error_types
    }

    fn generate_improvement_suggestions(
        &self,
        quality_score: &QualityScore,
        content_analysis: &ContentAnalysis,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if quality_score.relevance_score < 0.7 {
            suggestions.push("Improve alignment with the stated objective".to_string());
        }
        
        if quality_score.coherence_score < 0.7 {
            suggestions.push("Enhance logical flow and clarity".to_string());
        }
        
        if quality_score.completeness_score < 0.7 {
            suggestions.push("Provide more comprehensive information".to_string());
        }

        if content_analysis.word_count < 20 {
            suggestions.push("Expand content to provide more detail".to_string());
        }

        if content_analysis.readability_score < 30.0 {
            suggestions.push("Simplify language for better readability".to_string());
        }

        if content_analysis.sentence_count == 0 {
            suggestions.push("Structure content with proper sentences".to_string());
        }

        suggestions
    }

    fn calculate_diversity_indicators(&self, content: &str) -> Vec<String> {
        let mut indicators = Vec::new();

        let unique_words: std::collections::HashSet<&str> = content.split_whitespace().collect();
        let total_words = content.split_whitespace().count();
        
        if total_words > 0 {
            let diversity_ratio = unique_words.len() as f32 / total_words as f32;
            if diversity_ratio > 0.8 {
                indicators.push("high_vocabulary_diversity".to_string());
            } else if diversity_ratio < 0.5 {
                indicators.push("low_vocabulary_diversity".to_string());
            }
        }

        // Check for question marks, exclamation points, etc.
        if content.contains('?') {
            indicators.push("contains_questions".to_string());
        }
        if content.contains('!') {
            indicators.push("contains_exclamations".to_string());
        }
        if content.contains("example") || content.contains("for instance") {
            indicators.push("includes_examples".to_string());
        }

        indicators
    }
}

// Supporting structs and implementations

pub struct RuleBasedValidator {
    format_validators: HashMap<DatasetFormat, Box<dyn FormatValidator>>,
}

pub trait FormatValidator: Send + Sync {
    fn validate(&self, entry: &DatasetEntry) -> Result<RuleBasedValidationResult>;
}

pub struct LLMValidator {
    client: reqwest::Client,
    model_name: String,
}

pub struct AutomaticTagger {
    tag_patterns: HashMap<String, Vec<String>>,
}

pub struct DomainAdapter {
    adaptation_history: Vec<DomainAdaptationMetrics>,
}

pub struct NegativeSampler {
    client: reqwest::Client,
    model_name: String,
}

// Implementation details for supporting structs...

impl RuleBasedValidator {
    pub fn new() -> Self {
        Self {
            format_validators: HashMap::new(),
        }
    }

    pub fn validate(&self, entry: &DatasetEntry, format: &DatasetFormat) -> Result<RuleBasedValidationResult> {
        // Basic JSON validity check
        let json_validity = entry.data.is_object() || entry.data.is_array();
        
        // Basic field presence check
        let required_fields_present = match format {
            DatasetFormat::Alpaca => {
                entry.data.get("instruction").is_some() && 
                entry.data.get("output").is_some()
            },
            DatasetFormat::Conversation => {
                entry.data.is_array()
            },
            _ => true, // Simplified for other formats
        };

        // Field completeness check
        let mut field_completeness = HashMap::new();
        if let Some(instruction) = entry.data.get("instruction") {
            field_completeness.insert("instruction".to_string(), 
                instruction.as_str().map_or(false, |s| !s.trim().is_empty()));
        }
        if let Some(output) = entry.data.get("output") {
            field_completeness.insert("output".to_string(), 
                output.as_str().map_or(false, |s| !s.trim().is_empty()));
        }

        // Content length check
        let content_str = serde_json::to_string(&entry.data).unwrap_or_default();
        let content_length_check = content_str.len() > 20; // Minimum content length

        // Format compliance
        let format_compliance = json_validity && required_fields_present;

        // Generate issues
        let mut issues = Vec::new();
        if !json_validity {
            issues.push("Invalid JSON structure".to_string());
        }
        if !required_fields_present {
            issues.push("Missing required fields".to_string());
        }
        if !content_length_check {
            issues.push("Content too short".to_string());
        }

        // Calculate rule-based score
        let score = (
            if json_validity { 0.25 } else { 0.0 } +
            if required_fields_present { 0.25 } else { 0.0 } +
            if content_length_check { 0.25 } else { 0.0 } +
            if format_compliance { 0.25 } else { 0.0 }
        );

        Ok(RuleBasedValidationResult {
            json_validity,
            required_fields_present,
            field_completeness,
            format_compliance,
            content_length_check,
            issues,
            score,
        })
    }

    pub fn update_rules(&mut self, _feedback: &ValidationFeedback) -> Result<()> {
        // Update validation rules based on feedback
        // Implementation would adapt rules based on common issues
        Ok(())
    }
}

impl LLMValidator {
    pub fn new(model_name: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            model_name: model_name.unwrap_or_else(|| "llama3.2:3b".to_string()),
        }
    }

    pub async fn validate(
        &self,
        entry: &DatasetEntry,
        use_case: &str,
        format: &DatasetFormat,
    ) -> Result<QualityScore> {
        // This would implement LLM-based validation similar to the existing quality_validator
        // For brevity, returning a placeholder implementation
        Ok(QualityScore {
            overall_score: 0.8,
            relevance_score: 0.8,
            coherence_score: 0.8,
            completeness_score: 0.8,
            format_compliance_score: 0.8,
            issues: vec![],
            tags: vec!["llm_validated".to_string()],
        })
    }
}

impl AutomaticTagger {
    pub fn new() -> Self {
        let mut tag_patterns = HashMap::new();
        
        // Initialize tag patterns
        tag_patterns.insert("complexity".to_string(), vec![
            "beginner".to_string(), "intermediate".to_string(), "advanced".to_string()
        ]);
        tag_patterns.insert("domain".to_string(), vec![
            "technology".to_string(), "science".to_string(), "business".to_string(),
            "education".to_string(), "health".to_string(), "finance".to_string()
        ]);

        Self { tag_patterns }
    }

    pub async fn generate_tags(
        &self,
        entry: &DatasetEntry,
        quality_score: &QualityScore,
        format: &DatasetFormat,
    ) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        // Add format tag
        tags.push(format!("format:{:?}", format).to_lowercase());

        // Add quality level tag
        if quality_score.overall_score > 0.8 {
            tags.push("quality:high".to_string());
        } else if quality_score.overall_score > 0.6 {
            tags.push("quality:medium".to_string());
        } else {
            tags.push("quality:low".to_string());
        }

        // Add content-based tags
        let content = serde_json::to_string(&entry.data).unwrap_or_default().to_lowercase();
        
        if content.contains("code") || content.contains("programming") {
            tags.push("content:programming".to_string());
        }
        if content.contains("math") || content.contains("calculation") {
            tags.push("content:mathematics".to_string());
        }
        if content.contains("explain") || content.contains("description") {
            tags.push("task:explanation".to_string());
        }

        Ok(tags)
    }
}

impl DomainAdapter {
    pub fn new() -> Self {
        Self {
            adaptation_history: Vec::new(),
        }
    }

    pub fn analyze_drift(
        &self,
        recent_entries: &[ValidatedEntry],
        historical_entries: &[ValidatedEntry],
    ) -> Result<DomainAdaptationMetrics> {
        // Simplified drift detection
        let recent_avg_quality: f32 = recent_entries.iter()
            .map(|e| e.quality_score.overall_score)
            .sum::<f32>() / recent_entries.len() as f32;
        
        let historical_avg_quality: f32 = historical_entries.iter()
            .map(|e| e.quality_score.overall_score)
            .sum::<f32>() / historical_entries.len() as f32;

        let drift_score = (recent_avg_quality - historical_avg_quality).abs();

        Ok(DomainAdaptationMetrics {
            domain_drift_score: drift_score,
            new_patterns: vec!["pattern1".to_string()], // Placeholder
            obsolete_patterns: vec!["old_pattern".to_string()], // Placeholder
            adaptation_suggestions: vec!["suggestion1".to_string()], // Placeholder
        })
    }

    pub fn update_patterns(&mut self, _feedback: &ValidationFeedback) -> Result<()> {
        // Update domain adaptation patterns
        Ok(())
    }
}

impl NegativeSampler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            model_name: "llama3.2:3b".to_string(),
        }
    }

    pub async fn generate_samples(
        &self,
        positive_entries: &[DatasetEntry],
        format: &DatasetFormat,
        difficulty: &str,
    ) -> Result<NegativeSamplingResult> {
        // Generate negative samples based on positive examples
        let strategy = match format {
            DatasetFormat::PreferenceRanking => "adversarial_response_generation",
            DatasetFormat::Reranking => "hard_negative_mining",
            _ => "quality_degradation",
        };

        // Placeholder implementation
        Ok(NegativeSamplingResult {
            negative_examples: vec![], // Would generate actual negative samples
            sampling_strategy: strategy.to_string(),
            difficulty_level: difficulty.to_string(),
        })
    }
}
