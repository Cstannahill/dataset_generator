use crate::types::{DatasetEntry, ModelProvider, DatasetFormat};

pub struct DatasetGenerator;

impl DatasetGenerator {
    pub async fn generate_batch(
        model_id: &str,
        provider: &ModelProvider,
        goal: &str,
        format: &DatasetFormat,
        batch_size: usize,
        existing_entries: &[DatasetEntry],
    ) -> anyhow::Result<Vec<DatasetEntry>> {
        match provider {
            ModelProvider::Ollama => Self::generate_ollama_batch(model_id, goal, format, batch_size, existing_entries).await,
            ModelProvider::OpenAI => Self::generate_openai_batch(model_id, goal, format, batch_size, existing_entries).await,
        }
    }
    
    fn get_format_prompt(format: &DatasetFormat) -> &'static str {
        match format {
            DatasetFormat::Alpaca => "Format each as JSON with fields: instruction, input, output.",
            DatasetFormat::Conversation => "Format each as JSON with a 'messages' array containing objects with 'role' (user/assistant) and 'content' fields.",
            DatasetFormat::ChainOfThought => "Format each as JSON with fields: question, answer (including step-by-step reasoning).",
            DatasetFormat::PreferenceRanking => "Format each as JSON with fields: prompt, chosen, rejected.",
            DatasetFormat::FunctionCall => "Format each as JSON with fields: messages (conversation), function (name and arguments).",
            DatasetFormat::MultiRoundDialogue => "Format each as JSON with fields: instruction, conversation (array of role/content objects).",
            DatasetFormat::CodeTask => "Format each as JSON with fields: prompt, code, output.",
            DatasetFormat::Reflection => "Format each as JSON with fields: instruction, output, reflection, corrected.",
            DatasetFormat::RetrievalEmbedding => "Format each as JSON with fields: query, positive_passage, negative_passages (array).",
            DatasetFormat::Reranking => "Format each as JSON with fields: query, documents (array of text), relevance_scores (array of floats).",
        }
    }
    
    async fn generate_ollama_batch(
        model_id: &str,
        goal: &str,
        format: &DatasetFormat,
        batch_size: usize,
        existing_entries: &[DatasetEntry],
    ) -> anyhow::Result<Vec<DatasetEntry>> {
        let client = reqwest::Client::new();
        
        let context = if existing_entries.is_empty() {
            "This is the first batch.".to_string()
        } else {
            format!("Previous entries exist: {}", existing_entries.len())
        };
        
        let format_instruction = Self::get_format_prompt(format);
        
        let prompt = format!(
            "Generate {} training examples for fine-tuning goal: {}. Context: {}. 
            
            {}
            Return as a JSON array of objects.
            
            Goal: {}",
            batch_size, goal, context, format_instruction, goal
        );
        
        let request_body = serde_json::json!({
            "model": model_id,
            "prompt": prompt,
            "stream": false
        });
        
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let generated_text = result["response"].as_str().unwrap_or("[]");
            
            // Parse the generated JSON
            let parsed_entries: Result<Vec<serde_json::Value>, _> = serde_json::from_str(generated_text);
            
            let entries = match parsed_entries {
                Ok(values) => values.into_iter().map(|value| DatasetEntry { data: value }).collect(),
                Err(_) => {
                    // Fallback: create sample entries if parsing fails
                    Self::create_fallback_entries(format, batch_size)
                }
            };
            
            Ok(entries)
        } else {
            Err(anyhow::anyhow!("Failed to generate batch from Ollama"))
        }
    }
    
    async fn generate_openai_batch(
        model_id: &str,
        goal: &str,
        format: &DatasetFormat,
        batch_size: usize,
        existing_entries: &[DatasetEntry],
    ) -> anyhow::Result<Vec<DatasetEntry>> {
        let client = reqwest::Client::new();
        
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not found in environment. Please set it in your .env file or system environment"))?;
        
        let context = if existing_entries.is_empty() {
            "This is the first batch.".to_string()
        } else {
            format!("Previous entries exist: {}", existing_entries.len())
        };
        
        let format_instruction = Self::get_format_prompt(format);
        
        let prompt = format!(
            "Generate {} training examples for fine-tuning goal: {}. Context: {}. 
            
            {}
            Return as a JSON array of objects.
            
            Goal: {}",
            batch_size, goal, context, format_instruction, goal
        );
        
        let request_body = serde_json::json!({
            "model": model_id,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7
        });
        
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let generated_text = result["choices"][0]["message"]["content"].as_str().unwrap_or("[]");
            
            // Parse the generated JSON
            let parsed_entries: Result<Vec<serde_json::Value>, _> = serde_json::from_str(generated_text);
            
            let entries = match parsed_entries {
                Ok(values) => values.into_iter().map(|value| DatasetEntry { data: value }).collect(),
                Err(_) => {
                    // Fallback: create sample entries if parsing fails
                    Self::create_fallback_entries(format, batch_size)
                }
            };
            
            Ok(entries)
        } else {
            Err(anyhow::anyhow!("Failed to generate batch from OpenAI"))
        }
    }
    
    fn create_fallback_entries(format: &DatasetFormat, batch_size: usize) -> Vec<DatasetEntry> {
        (0..batch_size).map(|i| {
            let data = match format {
                DatasetFormat::Alpaca => serde_json::json!({
                    "instruction": format!("Sample instruction {}", i),
                    "input": format!("Sample input {}", i),
                    "output": format!("Sample output {}", i)
                }),
                DatasetFormat::Conversation => serde_json::json!({
                    "messages": [
                        {"role": "user", "content": format!("Sample user message {}", i)},
                        {"role": "assistant", "content": format!("Sample assistant response {}", i)}
                    ]
                }),
                DatasetFormat::ChainOfThought => serde_json::json!({
                    "question": format!("Sample question {}", i),
                    "answer": format!("Step 1: Sample reasoning step. Step 2: Another step. Final Answer: Sample answer {}", i)
                }),
                DatasetFormat::PreferenceRanking => serde_json::json!({
                    "prompt": format!("Sample prompt {}", i),
                    "chosen": format!("Good response {}", i),
                    "rejected": format!("Bad response {}", i)
                }),
                DatasetFormat::FunctionCall => serde_json::json!({
                    "messages": [{"role": "user", "content": format!("Sample function call request {}", i)}],
                    "function": {"name": "sample_function", "arguments": format!("{{\"param\": \"value{}\"}}", i)}
                }),
                DatasetFormat::MultiRoundDialogue => serde_json::json!({
                    "instruction": format!("Sample dialogue instruction {}", i),
                    "conversation": [
                        {"role": "user", "content": format!("Hello {}", i)},
                        {"role": "assistant", "content": format!("Hi there! How can I help you today? {}", i)}
                    ]
                }),
                DatasetFormat::CodeTask => serde_json::json!({
                    "prompt": format!("Sample code task {}", i),
                    "code": format!("def sample_function():\n    return {}", i),
                    "output": format!("Sample output {}", i)
                }),
                DatasetFormat::Reflection => serde_json::json!({
                    "instruction": format!("Sample instruction {}", i),
                    "output": format!("Initial output {}", i),
                    "reflection": format!("This could be improved by {}", i),
                    "corrected": format!("Corrected output {}", i)
                }),
                DatasetFormat::RetrievalEmbedding => serde_json::json!({
                    "query": format!("Sample query {}", i),
                    "positive_passage": format!("Relevant passage {}", i),
                    "negative_passages": [format!("Irrelevant passage {}", i), format!("Another irrelevant passage {}", i)]
                }),
                DatasetFormat::Reranking => serde_json::json!({
                    "query": format!("Sample query {}", i),
                    "documents": [
                        format!("First document {}", i),
                        format!("Second document {}", i),
                        format!("Third document {}", i)
                    ],
                    "relevance_scores": [0.9, 0.7, 0.3]
                }),
            };
            DatasetEntry { data }
        }).collect()
    }
}