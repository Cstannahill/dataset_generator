use crate::types::{Model, ModelProvider};

pub struct ModelManager;

impl ModelManager {
    pub async fn discover_ollama_models() -> anyhow::Result<Vec<Model>> {
        let client = reqwest::Client::new();
        
        // Ollama API endpoint for listing models
        let response = client
            .get("http://localhost:11434/api/tags")
            .send()
            .await?;
        
        if response.status().is_success() {
            let ollama_response: serde_json::Value = response.json().await?;
            let empty_vec = vec![];
            let models = ollama_response["models"].as_array().unwrap_or(&empty_vec);
            
            let mut discovered_models = Vec::new();
            for model in models {
                discovered_models.push(Model {
                    id: model["name"].as_str().unwrap_or("unknown").to_string(),
                    name: model["name"].as_str().unwrap_or("unknown").to_string(),
                    size: model["size"].as_str().unwrap_or("unknown").to_string(),
                    modified: model["modified_at"].as_str().unwrap_or("unknown").to_string(),
                    provider: ModelProvider::Ollama,
                    capabilities: vec!["text-generation".to_string()],
                });
            }
            
            Ok(discovered_models)
        } else {
            Err(anyhow::anyhow!("Failed to connect to Ollama service"))
        }
    }
    
    pub async fn get_openai_models() -> anyhow::Result<Vec<Model>> {
        // OpenAI models we'll support (latest models as of 2025)
        let openai_models = vec![
            Model {
                id: "gpt-4.1-nano".to_string(),
                name: "GPT-4.1-nano".to_string(),
                size: "nano".to_string(),
                modified: "2025".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec!["text-generation".to_string(), "instruction-following".to_string(), "fast-inference".to_string()],
            },
            Model {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                size: "multimodal".to_string(),
                modified: "2024".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec!["text-generation".to_string(), "instruction-following".to_string(), "multimodal".to_string()],
            },
            Model {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o-mini".to_string(),
                size: "efficient".to_string(),
                modified: "2024".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec!["text-generation".to_string(), "instruction-following".to_string(), "fast-inference".to_string()],
            },
            Model {
                id: "gpt-4.1-mini".to_string(),
                name: "GPT-4.1-mini".to_string(),
                size: "mini".to_string(),
                modified: "2025".to_string(),
                provider: ModelProvider::OpenAI,
                capabilities: vec!["text-generation".to_string(), "instruction-following".to_string(), "enhanced-reasoning".to_string()],
            },
        ];
        
        Ok(openai_models)
    }
}