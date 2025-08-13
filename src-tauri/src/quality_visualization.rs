use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::quality_validator::{ValidatedEntry, QualityScore};
use crate::enhanced_validation::{MultiStageValidationResult, DomainAdaptationMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityVisualizationData {
    pub overall_metrics: OverallQualityMetrics,
    pub quality_trends: QualityTrendData,
    pub error_analysis: ErrorAnalysisData,
    pub improvement_tracking: ImprovementTrackingData,
    pub domain_insights: DomainInsightData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallQualityMetrics {
    pub total_entries: usize,
    pub average_quality: f32,
    pub quality_distribution: QualityDistribution,
    pub pass_rate: f32,
    pub improvement_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDistribution {
    pub high_quality_count: usize,    // > 0.8
    pub medium_quality_count: usize,  // 0.6-0.8
    pub low_quality_count: usize,     // < 0.6
    pub percentages: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendData {
    pub batch_scores: Vec<BatchScore>,
    pub moving_average: Vec<f32>,
    pub trend_direction: String, // "improving", "declining", "stable"
    pub trend_strength: f32,     // -1.0 to 1.0
    pub prediction: QualityPrediction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScore {
    pub batch_id: usize,
    pub timestamp: i64,
    pub average_score: f32,
    pub entry_count: usize,
    pub validation_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPrediction {
    pub next_batch_predicted_score: f32,
    pub confidence_interval: (f32, f32),
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysisData {
    pub error_categories: HashMap<String, ErrorCategoryStats>,
    pub most_common_issues: Vec<IssueFrequency>,
    pub error_trends: Vec<ErrorTrendPoint>,
    pub resolution_suggestions: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCategoryStats {
    pub count: usize,
    pub percentage: f32,
    pub average_impact: f32,
    pub examples: Vec<String>,
    pub trend: String, // "increasing", "decreasing", "stable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFrequency {
    pub issue: String,
    pub count: usize,
    pub percentage: f32,
    pub severity: String, // "high", "medium", "low"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrendPoint {
    pub timestamp: i64,
    pub error_rate: f32,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementTrackingData {
    pub quality_progression: Vec<QualityProgressPoint>,
    pub milestone_achievements: Vec<Milestone>,
    pub performance_metrics: PerformanceMetrics,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProgressPoint {
    pub timestamp: i64,
    pub cumulative_entries: usize,
    pub average_quality: f32,
    pub quality_variance: f32,
    pub processing_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub milestone_type: String,
    pub achieved_at: i64,
    pub description: String,
    pub value: f32,
    pub significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub entries_per_second: f32,
    pub validation_efficiency: f32,
    pub resource_utilization: f32,
    pub throughput_trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub area: String,
    pub potential_improvement: f32,
    pub effort_required: String, // "low", "medium", "high"
    pub impact: String,          // "low", "medium", "high"
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInsightData {
    pub topic_distribution: HashMap<String, f32>,
    pub complexity_distribution: HashMap<String, f32>,
    pub domain_drift_indicators: Vec<DriftIndicator>,
    pub adaptation_history: Vec<AdaptationEvent>,
    pub knowledge_gaps: Vec<KnowledgeGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftIndicator {
    pub indicator_type: String,
    pub severity: f32,
    pub description: String,
    pub first_detected: i64,
    pub trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationEvent {
    pub timestamp: i64,
    pub event_type: String,
    pub description: String,
    pub impact_score: f32,
    pub success_metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub topic: String,
    pub coverage_percentage: f32,
    pub quality_score: f32,
    pub priority: String, // "high", "medium", "low"
    pub fill_strategy: Vec<String>,
}

pub struct QualityVisualizationService {
    historical_data: Vec<ValidatedEntry>,
    validation_results: Vec<MultiStageValidationResult>,
    batch_history: Vec<BatchScore>,
}

impl QualityVisualizationService {
    pub fn new() -> Self {
        Self {
            historical_data: Vec::new(),
            validation_results: Vec::new(),
            batch_history: Vec::new(),
        }
    }

    /// Generate comprehensive quality visualization data
    pub fn generate_visualization_data(&self) -> Result<QualityVisualizationData> {
        let overall_metrics = self.calculate_overall_metrics()?;
        let quality_trends = self.analyze_quality_trends()?;
        let error_analysis = self.analyze_errors()?;
        let improvement_tracking = self.track_improvements()?;
        let domain_insights = self.analyze_domain_insights()?;

        Ok(QualityVisualizationData {
            overall_metrics,
            quality_trends,
            error_analysis,
            improvement_tracking,
            domain_insights,
        })
    }

    /// Add new validation results for tracking
    pub fn add_validation_results(&mut self, results: Vec<MultiStageValidationResult>) {
        for result in &results {
            // Convert to ValidatedEntry for historical tracking
            let validated_entry = ValidatedEntry {
                entry: crate::types::DatasetEntry {
                    data: serde_json::json!({}), // Placeholder
                },
                quality_score: result.final_score.clone(),
                metadata: crate::quality_validator::EntryMetadata {
                    use_case: "unknown".to_string(),
                    dataset_format: crate::types::DatasetFormat::Alpaca, // Placeholder
                    content_hash: "".to_string(),
                    validation_timestamp: chrono::Utc::now().timestamp(),
                    embedding_id: None,
                },
            };
            self.historical_data.push(validated_entry);
        }
        self.validation_results.extend(results);
    }

    /// Add batch completion data
    pub fn add_batch_completion(&mut self, batch_score: BatchScore) {
        self.batch_history.push(batch_score);
        
        // Keep only recent history (last 100 batches)
        if self.batch_history.len() > 100 {
            self.batch_history = self.batch_history.iter().rev().take(100).rev().cloned().collect();
        }
    }

    /// Generate real-time quality dashboard data
    pub fn get_dashboard_data(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut dashboard = HashMap::new();

        // Current quality metrics
        let recent_scores: Vec<f32> = self.historical_data.iter()
            .rev()
            .take(50)
            .map(|entry| entry.quality_score.overall_score)
            .collect();

        let current_average = if !recent_scores.is_empty() {
            recent_scores.iter().sum::<f32>() / recent_scores.len() as f32
        } else {
            0.0
        };

        dashboard.insert("current_quality".to_string(), serde_json::json!(current_average));
        dashboard.insert("total_entries".to_string(), serde_json::json!(self.historical_data.len()));
        
        // Quality trend (last 10 batches)
        let recent_trend: Vec<f32> = self.batch_history.iter()
            .rev()
            .take(10)
            .map(|batch| batch.average_score)
            .collect();

        dashboard.insert("quality_trend".to_string(), serde_json::json!(recent_trend));

        // Pass rate
        let high_quality_count = self.historical_data.iter()
            .filter(|entry| entry.quality_score.overall_score > 0.8)
            .count();
        let pass_rate = if !self.historical_data.is_empty() {
            high_quality_count as f32 / self.historical_data.len() as f32
        } else {
            0.0
        };

        dashboard.insert("pass_rate".to_string(), serde_json::json!(pass_rate));

        // Error summary
        let recent_errors: Vec<String> = self.historical_data.iter()
            .rev()
            .take(20)
            .flat_map(|entry| &entry.quality_score.issues)
            .cloned()
            .collect();

        let error_counts = self.count_error_frequencies(&recent_errors);
        dashboard.insert("recent_errors".to_string(), serde_json::json!(error_counts));

        Ok(dashboard)
    }

    // Private implementation methods

    fn calculate_overall_metrics(&self) -> Result<OverallQualityMetrics> {
        let total_entries = self.historical_data.len();
        
        if total_entries == 0 {
            return Ok(OverallQualityMetrics {
                total_entries: 0,
                average_quality: 0.0,
                quality_distribution: QualityDistribution {
                    high_quality_count: 0,
                    medium_quality_count: 0,
                    low_quality_count: 0,
                    percentages: HashMap::new(),
                },
                pass_rate: 0.0,
                improvement_rate: 0.0,
            });
        }

        let total_quality: f32 = self.historical_data.iter()
            .map(|entry| entry.quality_score.overall_score)
            .sum();
        let average_quality = total_quality / total_entries as f32;

        let high_quality_count = self.historical_data.iter()
            .filter(|entry| entry.quality_score.overall_score > 0.8)
            .count();
        let medium_quality_count = self.historical_data.iter()
            .filter(|entry| entry.quality_score.overall_score > 0.6 && entry.quality_score.overall_score <= 0.8)
            .count();
        let low_quality_count = total_entries - high_quality_count - medium_quality_count;

        let mut percentages = HashMap::new();
        percentages.insert("high".to_string(), (high_quality_count as f32 / total_entries as f32) * 100.0);
        percentages.insert("medium".to_string(), (medium_quality_count as f32 / total_entries as f32) * 100.0);
        percentages.insert("low".to_string(), (low_quality_count as f32 / total_entries as f32) * 100.0);

        let quality_distribution = QualityDistribution {
            high_quality_count,
            medium_quality_count,
            low_quality_count,
            percentages,
        };

        let pass_rate = (high_quality_count as f32 / total_entries as f32) * 100.0;
        let improvement_rate = self.calculate_improvement_rate();

        Ok(OverallQualityMetrics {
            total_entries,
            average_quality,
            quality_distribution,
            pass_rate,
            improvement_rate,
        })
    }

    fn analyze_quality_trends(&self) -> Result<QualityTrendData> {
        let batch_scores = self.batch_history.clone();
        
        // Calculate moving average
        let window_size = 5;
        let moving_average = self.calculate_moving_average(&batch_scores, window_size);
        
        // Determine trend direction and strength
        let (trend_direction, trend_strength) = self.calculate_trend(&moving_average);
        
        // Generate prediction
        let prediction = self.predict_next_quality(&batch_scores);

        Ok(QualityTrendData {
            batch_scores,
            moving_average,
            trend_direction,
            trend_strength,
            prediction,
        })
    }

    fn analyze_errors(&self) -> Result<ErrorAnalysisData> {
        let all_issues: Vec<String> = self.historical_data.iter()
            .flat_map(|entry| &entry.quality_score.issues)
            .cloned()
            .collect();

        let error_categories = self.categorize_errors(&all_issues);
        let most_common_issues = self.get_most_common_issues(&all_issues);
        let error_trends = self.calculate_error_trends();
        let resolution_suggestions = self.generate_resolution_suggestions(&error_categories);

        Ok(ErrorAnalysisData {
            error_categories,
            most_common_issues,
            error_trends,
            resolution_suggestions,
        })
    }

    fn track_improvements(&self) -> Result<ImprovementTrackingData> {
        let quality_progression = self.calculate_quality_progression();
        let milestone_achievements = self.identify_milestones();
        let performance_metrics = self.calculate_performance_metrics();
        let optimization_opportunities = self.identify_optimization_opportunities();

        Ok(ImprovementTrackingData {
            quality_progression,
            milestone_achievements,
            performance_metrics,
            optimization_opportunities,
        })
    }

    fn analyze_domain_insights(&self) -> Result<DomainInsightData> {
        let topic_distribution = self.calculate_topic_distribution();
        let complexity_distribution = self.calculate_complexity_distribution();
        let domain_drift_indicators = self.detect_domain_drift_indicators();
        let adaptation_history = self.get_adaptation_history();
        let knowledge_gaps = self.identify_knowledge_gaps();

        Ok(DomainInsightData {
            topic_distribution,
            complexity_distribution,
            domain_drift_indicators,
            adaptation_history,
            knowledge_gaps,
        })
    }

    // Helper methods for calculations

    fn calculate_improvement_rate(&self) -> f32 {
        if self.batch_history.len() < 2 {
            return 0.0;
        }

        let recent_avg = self.batch_history.iter().rev().take(5)
            .map(|batch| batch.average_score)
            .sum::<f32>() / 5.0;

        let older_avg = self.batch_history.iter().rev().skip(5).take(5)
            .map(|batch| batch.average_score)
            .sum::<f32>() / 5.0;

        if older_avg > 0.0 {
            ((recent_avg - older_avg) / older_avg) * 100.0
        } else {
            0.0
        }
    }

    fn calculate_moving_average(&self, batch_scores: &[BatchScore], window_size: usize) -> Vec<f32> {
        let mut moving_avg = Vec::new();
        
        for i in 0..batch_scores.len() {
            let start = if i >= window_size { i - window_size + 1 } else { 0 };
            let window = &batch_scores[start..=i];
            let avg = window.iter().map(|b| b.average_score).sum::<f32>() / window.len() as f32;
            moving_avg.push(avg);
        }

        moving_avg
    }

    fn calculate_trend(&self, moving_average: &[f32]) -> (String, f32) {
        if moving_average.len() < 2 {
            return ("stable".to_string(), 0.0);
        }

        let recent = moving_average.iter().rev().take(5).cloned().collect::<Vec<_>>();
        let older = moving_average.iter().rev().skip(5).take(5).cloned().collect::<Vec<_>>();

        if recent.is_empty() || older.is_empty() {
            return ("stable".to_string(), 0.0);
        }

        let recent_avg = recent.iter().sum::<f32>() / recent.len() as f32;
        let older_avg = older.iter().sum::<f32>() / older.len() as f32;

        let trend_strength = (recent_avg - older_avg) / older_avg.max(0.1);

        let direction = if trend_strength > 0.05 {
            "improving"
        } else if trend_strength < -0.05 {
            "declining"
        } else {
            "stable"
        };

        (direction.to_string(), trend_strength)
    }

    fn predict_next_quality(&self, batch_scores: &[BatchScore]) -> QualityPrediction {
        if batch_scores.len() < 3 {
            return QualityPrediction {
                next_batch_predicted_score: 0.7,
                confidence_interval: (0.6, 0.8),
                recommendations: vec!["Insufficient data for prediction".to_string()],
            };
        }

        // Simple linear regression prediction
        let recent_scores: Vec<f32> = batch_scores.iter().rev().take(10)
            .map(|batch| batch.average_score)
            .collect();

        let avg_score = recent_scores.iter().sum::<f32>() / recent_scores.len() as f32;
        let variance = recent_scores.iter()
            .map(|score| (score - avg_score).powi(2))
            .sum::<f32>() / recent_scores.len() as f32;
        let std_dev = variance.sqrt();

        let confidence_interval = (
            (avg_score - 1.96 * std_dev).max(0.0),
            (avg_score + 1.96 * std_dev).min(1.0)
        );

        let recommendations = if avg_score < 0.7 {
            vec![
                "Focus on improving prompt quality".to_string(),
                "Increase validation strictness".to_string(),
                "Review common error patterns".to_string(),
            ]
        } else if avg_score > 0.85 {
            vec![
                "Maintain current quality standards".to_string(),
                "Explore more complex scenarios".to_string(),
                "Consider increasing batch size".to_string(),
            ]
        } else {
            vec![
                "Continue current approach".to_string(),
                "Minor optimization opportunities available".to_string(),
            ]
        };

        QualityPrediction {
            next_batch_predicted_score: avg_score,
            confidence_interval,
            recommendations,
        }
    }

    fn categorize_errors(&self, issues: &[String]) -> HashMap<String, ErrorCategoryStats> {
        let mut categories = HashMap::new();
        
        let category_keywords = [
            ("format_compliance", vec!["format", "structure", "json", "field"]),
            ("content_quality", vec!["grammar", "spelling", "clarity", "coherence"]),
            ("relevance", vec!["irrelevant", "off-topic", "unrelated"]),
            ("completeness", vec!["incomplete", "missing", "insufficient", "short"]),
            ("accuracy", vec!["incorrect", "wrong", "inaccurate", "false"]),
        ];

        for (category, keywords) in &category_keywords {
            let matching_issues: Vec<_> = issues.iter()
                .filter(|issue| keywords.iter().any(|keyword| issue.to_lowercase().contains(keyword)))
                .cloned()
                .collect();

            if !matching_issues.is_empty() {
                categories.insert(category.to_string(), ErrorCategoryStats {
                    count: matching_issues.len(),
                    percentage: (matching_issues.len() as f32 / issues.len() as f32) * 100.0,
                    average_impact: 0.7, // Placeholder
                    examples: matching_issues.into_iter().take(3).collect(),
                    trend: "stable".to_string(), // Placeholder
                });
            }
        }

        categories
    }

    fn get_most_common_issues(&self, issues: &[String]) -> Vec<IssueFrequency> {
        let issue_counts = self.count_error_frequencies(issues);
        let total_issues = issues.len();
        
        let mut issue_frequencies: Vec<_> = issue_counts.into_iter()
            .map(|(issue, count)| IssueFrequency {
                issue,
                count,
                percentage: (count as f32 / total_issues as f32) * 100.0,
                severity: if count > total_issues / 4 { "high" } 
                         else if count > total_issues / 10 { "medium" } 
                         else { "low" }.to_string(),
            })
            .collect();

        issue_frequencies.sort_by(|a, b| b.count.cmp(&a.count));
        issue_frequencies.into_iter().take(10).collect()
    }

    fn count_error_frequencies(&self, issues: &[String]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for issue in issues {
            *counts.entry(issue.clone()).or_insert(0) += 1;
        }
        counts
    }

    // Placeholder implementations for remaining methods
    fn calculate_error_trends(&self) -> Vec<ErrorTrendPoint> { Vec::new() }
    fn generate_resolution_suggestions(&self, _categories: &HashMap<String, ErrorCategoryStats>) -> HashMap<String, Vec<String>> { HashMap::new() }
    fn calculate_quality_progression(&self) -> Vec<QualityProgressPoint> { Vec::new() }
    fn identify_milestones(&self) -> Vec<Milestone> { Vec::new() }
    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            entries_per_second: 1.0,
            validation_efficiency: 0.8,
            resource_utilization: 0.6,
            throughput_trend: "stable".to_string(),
        }
    }
    fn identify_optimization_opportunities(&self) -> Vec<OptimizationOpportunity> { Vec::new() }
    fn calculate_topic_distribution(&self) -> HashMap<String, f32> { HashMap::new() }
    fn calculate_complexity_distribution(&self) -> HashMap<String, f32> { HashMap::new() }
    fn detect_domain_drift_indicators(&self) -> Vec<DriftIndicator> { Vec::new() }
    fn get_adaptation_history(&self) -> Vec<AdaptationEvent> { Vec::new() }
    fn identify_knowledge_gaps(&self) -> Vec<KnowledgeGap> { Vec::new() }
}

impl Default for QualityVisualizationService {
    fn default() -> Self {
        Self::new()
    }
}
