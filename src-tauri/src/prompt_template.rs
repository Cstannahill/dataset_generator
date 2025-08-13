use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::types::{DatasetEntry, DatasetFormat};
use crate::quality_validator::ValidationFeedback;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub base_template: String,
    pub format_specific_templates: HashMap<DatasetFormat, String>,
    pub few_shot_examples: HashMap<DatasetFormat, Vec<DatasetEntry>>,
    pub chain_of_thought_examples: HashMap<DatasetFormat, Vec<CoTExample>>,
    pub dynamic_instructions: Vec<String>,
    pub negative_examples: HashMap<DatasetFormat, Vec<DatasetEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTExample {
    pub problem: String,
    pub reasoning_steps: Vec<String>,
    pub final_answer: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    pub previous_batches_summary: String,
    pub dataset_statistics: DatasetStatistics,
    pub common_errors: Vec<String>,
    pub validation_feedback: Option<ValidationFeedback>,
    pub domain_drift_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStatistics {
    pub total_entries: usize,
    pub average_quality_score: f32,
    pub format_distribution: HashMap<DatasetFormat, usize>,
    pub topic_distribution: HashMap<String, usize>,
    pub complexity_distribution: HashMap<String, usize>,
    pub batch_quality_trend: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub context_instructions: String,
    pub format_examples: Vec<DatasetEntry>,
    pub quality_guidelines: String,
    pub diversity_instructions: String,
    pub negative_sampling_hint: Option<String>,
}

pub struct PromptTemplateEngine {
    templates: HashMap<String, PromptTemplate>,
    default_template: PromptTemplate,
}

impl PromptTemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            default_template: Self::create_default_template(),
        };

        // Initialize with format-specific templates
        engine.initialize_format_templates();
        engine
    }

    /// Generate a context-aware prompt based on historical data and feedback
    pub fn generate_prompt(
        &self,
        format: &DatasetFormat,
        use_case: &str,
        batch_size: usize,
        context: &PromptContext,
        domain_context: &str,
    ) -> Result<GenerationPrompt> {
        let template = self.get_template_for_format(format);
        
        // Build system prompt with context awareness
        let system_prompt = self.build_system_prompt(&template, format, context)?;
        
        // Build user prompt with examples and guidelines
        let user_prompt = self.build_user_prompt(
            &template,
            format,
            use_case,
            batch_size,
            domain_context,
            context,
        )?;

        // Generate context-specific instructions
        let context_instructions = self.build_context_instructions(context)?;

        // Get format-specific examples
        let format_examples = self.get_format_examples(format, context);

        // Build quality guidelines
        let quality_guidelines = self.build_quality_guidelines(format, context)?;

        // Build diversity instructions
        let diversity_instructions = self.build_diversity_instructions(context)?;

        // Generate negative sampling hints for appropriate formats
        let negative_sampling_hint = self.generate_negative_sampling_hint(format, context);

        Ok(GenerationPrompt {
            system_prompt,
            user_prompt,
            context_instructions,
            format_examples,
            quality_guidelines,
            diversity_instructions,
            negative_sampling_hint,
        })
    }

    /// Update template with new feedback and learning
    pub fn update_template_with_feedback(
        &mut self,
        format: &DatasetFormat,
        feedback: &ValidationFeedback,
        _batch_quality_score: f32,
    ) -> Result<()> {
        let template_id = format!("{:?}_template", format);
        
        if let Some(template) = self.templates.get_mut(&template_id) {
            // Add avoid patterns to dynamic instructions
            for avoid_pattern in &feedback.avoid_patterns {
                let instruction = format!("AVOID: {}", avoid_pattern);
                if !template.dynamic_instructions.contains(&instruction) {
                    template.dynamic_instructions.push(instruction);
                }
            }

            // Add improvement suggestions
            for suggestion in &feedback.improvement_suggestions {
                let instruction = format!("FOCUS: {}", suggestion);
                if !template.dynamic_instructions.contains(&instruction) {
                    template.dynamic_instructions.push(instruction);
                }
            }

            // Maintain only the most recent and relevant instructions (max 20)
            if template.dynamic_instructions.len() > 20 {
                template.dynamic_instructions = template.dynamic_instructions
                    .iter()
                    .rev()
                    .take(20)
                    .rev()
                    .cloned()
                    .collect();
            }
        }

        tracing::info!(
            "Updated template for {:?} with {} new instructions",
            format,
            feedback.avoid_patterns.len() + feedback.improvement_suggestions.len()
        );

        Ok(())
    }

    /// Detect domain drift and adapt prompts accordingly
    pub fn detect_domain_drift(
        &self,
        recent_entries: &[DatasetEntry],
        historical_topics: &HashMap<String, usize>,
    ) -> Vec<String> {
        let mut drift_indicators = Vec::new();

        // Analyze topic distribution in recent entries
        let recent_topics = self.extract_topics_from_entries(recent_entries);
        
        // Compare with historical distribution
        for (topic, recent_count) in &recent_topics {
            let historical_count = historical_topics.get(topic).unwrap_or(&0);
            let historical_total: usize = historical_topics.values().sum();
            let recent_total = recent_topics.values().sum::<usize>();

            if historical_total > 0 && recent_total > 0 {
                let historical_ratio = *historical_count as f32 / historical_total as f32;
                let recent_ratio = *recent_count as f32 / recent_total as f32;

                if (recent_ratio - historical_ratio).abs() > 0.2 {
                    drift_indicators.push(format!(
                        "Topic '{}' distribution changed from {:.2}% to {:.2}%",
                        topic,
                        historical_ratio * 100.0,
                        recent_ratio * 100.0
                    ));
                }
            }
        }

        drift_indicators
    }

    /// Generate adaptive prompts for different complexity levels
    pub fn generate_adaptive_prompt(
        &self,
        format: &DatasetFormat,
        complexity_level: &str,
        current_quality_score: f32,
    ) -> Result<String> {
        let base_template = self.get_template_for_format(format);
        
        let complexity_instruction = match complexity_level {
            "beginner" => "Focus on simple, clear examples that are easy to understand and follow",
            "intermediate" => "Create moderately complex examples that require some reasoning but remain accessible",
            "advanced" => "Generate sophisticated examples that demonstrate deep understanding and complex reasoning",
            _ => "Create examples appropriate for the general audience",
        };

        let quality_adjustment = if current_quality_score < 0.7 {
            "\nIMPORTANT: Recent quality has been below target. Focus on:\n- Clearer instructions\n- More complete responses\n- Better alignment with the use case\n- Proper format compliance"
        } else if current_quality_score > 0.9 {
            "\nExcellent quality trend! Maintain current standards while exploring:\n- Slightly more complex scenarios\n- Edge cases\n- Creative variations within the format"
        } else {
            ""
        };

        Ok(format!(
            "{}\n\nCOMPLEXITY LEVEL: {}\n{}{}",
            base_template.base_template,
            complexity_instruction,
            quality_adjustment,
            if !base_template.dynamic_instructions.is_empty() {
                format!("\n\nDYNAMIC GUIDELINES:\n{}", base_template.dynamic_instructions.join("\n"))
            } else {
                String::new()
            }
        ))
    }

    // Private helper methods

    fn create_default_template() -> PromptTemplate {
        PromptTemplate {
            id: "default".to_string(),
            name: "Default Template".to_string(),
            base_template: include_str!("templates/default_prompt.txt").to_string(),
            format_specific_templates: HashMap::new(),
            few_shot_examples: HashMap::new(),
            chain_of_thought_examples: HashMap::new(),
            dynamic_instructions: Vec::new(),
            negative_examples: HashMap::new(),
        }
    }

    fn initialize_format_templates(&mut self) {
        // Initialize templates for each format
        for format in [
            DatasetFormat::Alpaca,
            DatasetFormat::Conversation,
            DatasetFormat::ChainOfThought,
            DatasetFormat::PreferenceRanking,
            DatasetFormat::FunctionCall,
            DatasetFormat::MultiRoundDialogue,
            DatasetFormat::CodeTask,
            DatasetFormat::Reflection,
            DatasetFormat::RetrievalEmbedding,
            DatasetFormat::Reranking,
        ] {
            let template = self.create_format_specific_template(&format);
            self.templates.insert(format!("{:?}_template", format), template);
        }
    }

    fn create_format_specific_template(&self, format: &DatasetFormat) -> PromptTemplate {
        let (template_content, examples) = match format {
            DatasetFormat::Alpaca => (
                include_str!("templates/alpaca_prompt.txt").to_string(),
                self.create_alpaca_examples()
            ),
            DatasetFormat::Conversation => (
                include_str!("templates/conversation_prompt.txt").to_string(),
                self.create_conversation_examples()
            ),
            DatasetFormat::ChainOfThought => (
                include_str!("templates/chain_of_thought_prompt.txt").to_string(),
                self.create_cot_examples()
            ),
            DatasetFormat::PreferenceRanking => (
                include_str!("templates/preference_ranking_prompt.txt").to_string(),
                self.create_preference_examples()
            ),
            DatasetFormat::Reranking => (
                include_str!("templates/reranking_prompt.txt").to_string(),
                self.create_reranking_examples()
            ),
            _ => (
                self.default_template.base_template.clone(),
                Vec::new()
            ),
        };

        PromptTemplate {
            id: format!("{:?}_template", format),
            name: format!("{:?} Template", format),
            base_template: template_content,
            format_specific_templates: HashMap::new(),
            few_shot_examples: HashMap::from([(format.clone(), examples)]),
            chain_of_thought_examples: HashMap::new(),
            dynamic_instructions: Vec::new(),
            negative_examples: HashMap::new(),
        }
    }

    fn get_template_for_format(&self, format: &DatasetFormat) -> &PromptTemplate {
        let template_id = format!("{:?}_template", format);
        self.templates.get(&template_id).unwrap_or(&self.default_template)
    }

    fn build_system_prompt(
        &self,
        _template: &PromptTemplate,
        format: &DatasetFormat,
        context: &PromptContext,
    ) -> Result<String> {
        let mut system_prompt = format!(
            "You are an expert AI trainer specializing in creating high-quality training datasets for the {:?} format.\n\n",
            format
        );

        // Add context awareness
        if !context.previous_batches_summary.is_empty() {
            system_prompt.push_str(&format!(
                "PREVIOUS BATCH CONTEXT:\n{}\n\n",
                context.previous_batches_summary
            ));
        }

        // Add quality trend information
        if !context.dataset_statistics.batch_quality_trend.is_empty() {
            let recent_trend: f32 = context.dataset_statistics.batch_quality_trend
                .iter()
                .rev()
                .take(3)
                .sum::<f32>() / 3.0;
            
            system_prompt.push_str(&format!(
                "QUALITY TREND: Recent average quality score is {:.2}. ",
                recent_trend
            ));

            if recent_trend < 0.7 {
                system_prompt.push_str("Focus on improving quality through clearer instructions and better examples.\n\n");
            } else if recent_trend > 0.85 {
                system_prompt.push_str("Maintain excellent quality while exploring more diverse scenarios.\n\n");
            } else {
                system_prompt.push_str("Quality is good, continue current approach with minor improvements.\n\n");
            }
        }

        // Add domain drift warnings
        if !context.domain_drift_indicators.is_empty() {
            system_prompt.push_str("DOMAIN ADAPTATION NEEDED:\n");
            for indicator in &context.domain_drift_indicators {
                system_prompt.push_str(&format!("- {}\n", indicator));
            }
            system_prompt.push_str("\n");
        }

        Ok(system_prompt)
    }

    fn build_user_prompt(
        &self,
        template: &PromptTemplate,
        format: &DatasetFormat,
        use_case: &str,
        batch_size: usize,
        domain_context: &str,
        context: &PromptContext,
    ) -> Result<String> {
        let mut prompt = template.base_template.clone();

        // Replace placeholders
        prompt = prompt.replace("{use_case}", use_case);
        prompt = prompt.replace("{batch_size}", &batch_size.to_string());
        prompt = prompt.replace("{domain_context}", domain_context);
        prompt = prompt.replace("{format}", &format!("{:?}", format));

        // Add few-shot examples if available
        if let Some(examples) = template.few_shot_examples.get(format) {
            if !examples.is_empty() {
                prompt.push_str("\n\nHIGH-QUALITY EXAMPLES TO FOLLOW:\n");
                for (i, example) in examples.iter().take(2).enumerate() {
                    prompt.push_str(&format!("Example {}:\n{}\n\n", i + 1, 
                        serde_json::to_string_pretty(&example.data).unwrap_or_default()));
                }
            }
        }

        // Add chain-of-thought examples for complex formats
        if matches!(format, DatasetFormat::ChainOfThought | DatasetFormat::CodeTask) {
            if let Some(cot_examples) = template.chain_of_thought_examples.get(format) {
                if !cot_examples.is_empty() {
                    prompt.push_str("\n\nCHAIN-OF-THOUGHT EXAMPLE:\n");
                    if let Some(example) = cot_examples.first() {
                        prompt.push_str(&format!(
                            "Problem: {}\nReasoning:\n{}\nFinal Answer: {}\n\n",
                            example.problem,
                            example.reasoning_steps.join("\n"),
                            example.final_answer
                        ));
                    }
                }
            }
        }

        // Add dynamic instructions based on feedback
        if !template.dynamic_instructions.is_empty() {
            prompt.push_str("\n\nIMPORTANT GUIDELINES BASED ON RECENT ANALYSIS:\n");
            for instruction in template.dynamic_instructions.iter().take(10) {
                prompt.push_str(&format!("- {}\n", instruction));
            }
            prompt.push_str("\n");
        }

        // Add common error avoidance
        if !context.common_errors.is_empty() {
            prompt.push_str("\nAVOID THESE COMMON ERRORS:\n");
            for error in context.common_errors.iter().take(5) {
                prompt.push_str(&format!("- {}\n", error));
            }
            prompt.push_str("\n");
        }

        Ok(prompt)
    }

    fn build_context_instructions(&self, context: &PromptContext) -> Result<String> {
        let mut instructions = String::new();

        if context.dataset_statistics.total_entries > 0 {
            instructions.push_str(&format!(
                "Dataset context: {} total entries, average quality {:.2}\n",
                context.dataset_statistics.total_entries,
                context.dataset_statistics.average_quality_score
            ));
        }

        if let Some(feedback) = &context.validation_feedback {
            if !feedback.improvement_suggestions.is_empty() {
                instructions.push_str("Recent improvement suggestions:\n");
                for suggestion in &feedback.improvement_suggestions {
                    instructions.push_str(&format!("- {}\n", suggestion));
                }
            }
        }

        Ok(instructions)
    }

    fn get_format_examples(&self, format: &DatasetFormat, _context: &PromptContext) -> Vec<DatasetEntry> {
        let template = self.get_template_for_format(format);
        template.few_shot_examples.get(format).cloned().unwrap_or_default()
    }

    fn build_quality_guidelines(&self, format: &DatasetFormat, context: &PromptContext) -> Result<String> {
        let mut guidelines = format!(
            "Quality guidelines for {:?} format:\n\
            - Ensure all required fields are present and meaningful\n\
            - Make instructions clear and actionable\n\
            - Provide relevant context in input fields\n\
            - Generate comprehensive and helpful outputs\n",
            format
        );

        // Add format-specific guidelines
        match format {
            DatasetFormat::Alpaca => {
                guidelines.push_str("- Instructions should be specific and task-oriented\n\
                                   - Inputs should provide necessary context\n\
                                   - Outputs should directly address the instruction\n");
            },
            DatasetFormat::Conversation => {
                guidelines.push_str("- Maintain natural conversation flow\n\
                                   - Each turn should add value to the dialogue\n\
                                   - Responses should be contextually appropriate\n");
            },
            DatasetFormat::ChainOfThought => {
                guidelines.push_str("- Show clear reasoning steps\n\
                                   - Each step should logically follow from the previous\n\
                                   - Final answer should be clearly stated\n");
            },
            DatasetFormat::Reranking => {
                guidelines.push_str("- Ensure clear relevance differences between positive and negative documents\n\
                                   - Queries should be realistic and specific\n\
                                   - Documents should vary in relevance levels\n");
            },
            _ => {}
        }

        // Add context-specific guidelines
        if let Some(feedback) = &context.validation_feedback {
            if !feedback.quality_patterns.is_empty() {
                guidelines.push_str("\nSuccessful patterns to continue:\n");
                for pattern in &feedback.quality_patterns {
                    guidelines.push_str(&format!("- {}\n", pattern));
                }
            }
        }

        Ok(guidelines)
    }

    fn build_diversity_instructions(&self, context: &PromptContext) -> Result<String> {
        let mut instructions = "DIVERSITY REQUIREMENTS:\n\
            - Vary complexity levels (beginner, intermediate, advanced)\n\
            - Use different domains and topics\n\
            - Include various sentence structures and lengths\n\
            - Explore different perspectives and approaches\n".to_string();

        // Add topic distribution guidance
        if !context.dataset_statistics.topic_distribution.is_empty() {
            let total_entries = context.dataset_statistics.total_entries;
            let underrepresented_topics: Vec<_> = context.dataset_statistics.topic_distribution
                .iter()
                .filter(|(_, count)| **count < total_entries / 10) // Less than 10% representation
                .map(|(topic, _)| topic)
                .collect();

            if !underrepresented_topics.is_empty() {
                instructions.push_str("\nFocus more on these underrepresented topics:\n");
                for topic in underrepresented_topics.iter().take(5) {
                    instructions.push_str(&format!("- {}\n", topic));
                }
            }
        }

        Ok(instructions)
    }

    fn generate_negative_sampling_hint(&self, format: &DatasetFormat, _context: &PromptContext) -> Option<String> {
        match format {
            DatasetFormat::PreferenceRanking => Some(
                "For rejected responses, create plausible but clearly inferior alternatives that:\n\
                - Are factually incorrect or incomplete\n\
                - Miss the main point of the question\n\
                - Use inappropriate tone or style\n\
                - Provide generic rather than specific answers".to_string()
            ),
            DatasetFormat::Reranking => Some(
                "For negative documents, include:\n\
                - Documents that are topically related but don't answer the query\n\
                - Documents with partial relevance but missing key information\n\
                - Documents that contradict the query intent\n\
                - Documents from similar domains but different contexts".to_string()
            ),
            _ => None,
        }
    }

    fn extract_topics_from_entries(&self, entries: &[DatasetEntry]) -> HashMap<String, usize> {
        let mut topics = HashMap::new();
        
        // Simple keyword-based topic extraction
        // In a real implementation, this could use NLP libraries or LLM-based topic modeling
        for entry in entries {
            let content = serde_json::to_string(&entry.data).unwrap_or_default().to_lowercase();
            
            // Basic topic keywords
            let topic_keywords = [
                ("technology", vec!["computer", "software", "programming", "AI", "machine learning"]),
                ("science", vec!["research", "experiment", "hypothesis", "theory"]),
                ("business", vec!["marketing", "sales", "revenue", "customer", "company"]),
                ("education", vec!["learning", "student", "teacher", "curriculum", "academic"]),
                ("health", vec!["medical", "health", "doctor", "patient", "treatment"]),
                ("finance", vec!["money", "investment", "financial", "bank", "economy"]),
            ];

            for (topic, keywords) in &topic_keywords {
                if keywords.iter().any(|keyword| content.contains(keyword)) {
                    *topics.entry(topic.to_string()).or_insert(0) += 1;
                }
            }
        }

        topics
    }

    // Example creation methods
    fn create_alpaca_examples(&self) -> Vec<DatasetEntry> {
        vec![
            DatasetEntry {
                data: serde_json::json!({
                    "instruction": "Explain the concept of machine learning to a beginner",
                    "input": "",
                    "output": "Machine learning is a type of artificial intelligence where computers learn to make predictions or decisions by analyzing patterns in data, rather than being explicitly programmed for every possible scenario. Think of it like teaching a child to recognize animals - instead of describing every feature of every animal, you show them many examples, and they learn to identify patterns that help them recognize new animals they haven't seen before."
                }),
            }
        ]
    }

    fn create_conversation_examples(&self) -> Vec<DatasetEntry> {
        vec![
            DatasetEntry {
                data: serde_json::json!([
                    {"role": "user", "content": "I'm having trouble sleeping lately. Any advice?"},
                    {"role": "assistant", "content": "I'm sorry to hear you're having sleep difficulties. Here are some evidence-based tips that might help: establish a consistent bedtime routine, avoid screens 1-2 hours before bed, keep your bedroom cool and dark, and try relaxation techniques like deep breathing. If problems persist, consider consulting a healthcare provider."}
                ]),
            }
        ]
    }

    fn create_cot_examples(&self) -> Vec<DatasetEntry> {
        vec![
            DatasetEntry {
                data: serde_json::json!({
                    "question": "A store offers a 20% discount on all items. If an item originally costs $50, what is the final price after discount?",
                    "answer": "Step 1: Calculate the discount amount: 20% of $50 = 0.20 × $50 = $10\nStep 2: Subtract the discount from the original price: $50 - $10 = $40\nFinal Answer: The final price after the 20% discount is $40."
                }),
            }
        ]
    }

    fn create_preference_examples(&self) -> Vec<DatasetEntry> {
        vec![
            DatasetEntry {
                data: serde_json::json!({
                    "prompt": "Explain why recycling is important",
                    "chosen": "Recycling is crucial for environmental sustainability because it reduces waste sent to landfills, conserves natural resources by reusing materials, decreases pollution from manufacturing new products, and helps combat climate change by reducing greenhouse gas emissions. For example, recycling one ton of paper saves 17 trees and 7,000 gallons of water.",
                    "rejected": "Recycling is good for the environment. It helps reduce waste and saves resources. People should recycle more."
                }),
            }
        ]
    }

    fn create_reranking_examples(&self) -> Vec<DatasetEntry> {
        vec![
            DatasetEntry {
                data: serde_json::json!({
                    "query": "How to bake chocolate chip cookies",
                    "positive_document": "To bake chocolate chip cookies, preheat oven to 375°F. Mix 2¼ cups flour, 1 tsp salt, and 1 tsp baking soda. In another bowl, cream 1 cup butter with ¾ cup each of brown and white sugar. Add 2 eggs and 2 tsp vanilla. Combine wet and dry ingredients, fold in 2 cups chocolate chips. Drop spoonfuls on baking sheet and bake 9-11 minutes until golden brown.",
                    "negative_document": "Chocolate chip cookies are a popular dessert enjoyed by many people around the world. They were invented in the 1930s and have become a staple in American households. The key to good cookies is using quality ingredients and proper technique."
                }),
            }
        ]
    }
}

impl Default for PromptTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
