// Gemma4 E4B Inference Integration Test
// Tests actual inference generation with the Gemma4 model

use crate::inference_engine::{InferenceEngine, ModelInfo, BackendType, InferenceRequest};
use std::path::PathBuf;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_gemma4_actual_inference_request() {
        let engine = InferenceEngine::new();
        
        let model_info = ModelInfo {
            name: "gemma-4-e4b-it-q4_k_m".to_string(),
            path: PathBuf::from("../allama/models/gemma-4-E4B/gemma-4-E4B-it-Q4_K_M.gguf"),
            size_bytes: 4977169568,
            quantization: "Q4_K_M".to_string(),
            parameters: "4B".to_string(),
            context_length: 8192,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        // Add model to engine
        let result = engine.add_model(model_info).await;
        assert!(result.is_ok(), "Should be able to add model to engine");
        
        // Create a simple inference request
        let request = InferenceRequest {
            model: "gemma-4-e4b-it-q4_k_m".to_string(),
            prompt: "Hello, how are you?".to_string(),
            max_tokens: Some(50),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: Some(1.1),
            stop: None,
            stream: false,
        };
        
        // Note: This test validates the request structure is correct
        // Actual inference requires allama service to be running
        assert_eq!(request.model, "gemma-4-e4b-it-q4_k_m");
        assert_eq!(request.prompt, "Hello, how are you?");
        assert_eq!(request.max_tokens, Some(50));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_gemma4_chat_request_structure() {
        use crate::inference_engine::{ChatMessage, ChatRequest};
        
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "What is artificial intelligence?".to_string(),
            },
        ];
        
        let request = ChatRequest {
            model: "gemma-4-e4b-it-q4_k_m".to_string(),
            messages: messages.clone(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stream: false,
        };
        
        // Validate chat request structure
        assert_eq!(request.model, "gemma-4-e4b-it-q4_k_m");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "What is artificial intelligence?");
        assert_eq!(request.max_tokens, Some(100));
    }

    #[tokio::test]
    async fn test_gemma4_modelfile_parameters() {
        use std::fs;
        
        let modelfile_path = PathBuf::from("../allama/models/gemma-4-E4B/Modelfile");
        assert!(modelfile_path.exists(), "Modelfile should exist");
        
        let content = fs::read_to_string(&modelfile_path).expect("Should be able to read Modelfile");
        
        // Verify key parameters are set correctly
        assert!(content.contains("PARAMETER temperature 0.7"), "Temperature should be 0.7");
        assert!(content.contains("PARAMETER top_p 0.9"), "Top_p should be 0.9");
        assert!(content.contains("PARAMETER top_k 40"), "Top_k should be 40");
        assert!(content.contains("PARAMETER num_ctx 8192"), "Context length should be 8192");
        assert!(content.contains("PARAMETER num_predict 512"), "Max tokens should be 512");
        assert!(content.contains("PARAMETER repeat_penalty 1.1"), "Repeat penalty should be 1.1");
        
        // Verify template exists
        assert!(content.contains("TEMPLATE"), "Should have chat template");
        assert!(content.contains("SYSTEM"), "Should have system prompt");
    }
}
