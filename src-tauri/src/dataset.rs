use crate::types::{DatasetEntry, ModelProvider};

pub struct DatasetGenerator;

impl DatasetGenerator {
    pub async fn generate_batch(
        model_id: &str,
        provider: &ModelProvider,
        goal: &str,
        batch_size: usize,
        existing_entries: &[DatasetEntry],
    ) -> anyhow::Result<Vec<DatasetEntry>> {
        match provider {
            ModelProvider::Ollama => Self::generate_ollama_batch(model_id, goal, batch_size, existing_entries).await,
            ModelProvider::OpenAI => Self::generate_openai_batch(model_id, goal, batch_size, existing_entries).await,
        }
    }
    
    async fn generate_ollama_batch(
        model_id: &str,
        goal: &str,
        batch_size: usize,
        existing_entries: &[DatasetEntry],
    ) -> anyhow::Result<Vec<DatasetEntry>> {
        let client = reqwest::Client::new();
        
        let context = if existing_entries.is_empty() {
            "This is the first batch.".to_string()
        } else {
            format!("Previous entries exist: {}", existing_entries.len())
        };
        
        let prompt = format!(
            "Generate {} training examples for fine-tuning goal: {}. Context: {}. 
            
            Format each as JSON with fields: instruction, input, output.
            Return as a JSON array of objects.
            
            Goal: {}",
            batch_size, goal, context, goal
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
            
            // Parse the generated JSON (simplified for demo)
            let entries: Vec<DatasetEntry> = serde_json::from_str(generated_text)
                .unwrap_or_else(|_| {
                    // Fallback: create sample entries if parsing fails
                    (0..batch_size).map(|i| DatasetEntry {
                        instruction: format!("Sample instruction {}", i),
                        input: format!("Sample input {}", i),
                        output: format!("Sample output {}", i),
                    }).collect()
                });
            
            Ok(entries)
        } else {
            Err(anyhow::anyhow!("Failed to generate batch from Ollama"))
        }
    }
    
    async fn generate_openai_batch(
        model_id: &str,
        goal: &str,
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
        
        let prompt = format!(
            "Generate {} training examples for fine-tuning goal: {}. Context: {}. 
            
            Format each as JSON with fields: instruction, input, output.
            Return as a JSON array of objects.
            
            Goal: {}",
            batch_size, goal, context, goal
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
            
            // Parse the generated JSON (simplified for demo)
            let entries: Vec<DatasetEntry> = serde_json::from_str(generated_text)
                .unwrap_or_else(|_| {
                    // Fallback: create sample entries if parsing fails
                    (0..batch_size).map(|i| DatasetEntry {
                        instruction: format!("Sample instruction {}", i),
                        input: format!("Sample input {}", i),
                        output: format!("Sample output {}", i),
                    }).collect()
                });
            
            Ok(entries)
        } else {
            Err(anyhow::anyhow!("Failed to generate batch from OpenAI"))
        }
    }
}