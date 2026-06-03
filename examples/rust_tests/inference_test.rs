//! 推理引擎集成测试
//! 测试 Allama 推理引擎的完整功能

#[cfg(test)]
mod integration_tests {
    use std::path::PathBuf;
    use crate::inference_engine::{
        InferenceEngine, InferenceRequest, ChatMessage, ChatRequest,
        ModelInfo, BackendType, InferenceConfig, ContextCompressionConfig,
        CompressionMethod
    };

    #[tokio::test]
    async fn test_inference_engine_full_workflow() {
        let engine = InferenceEngine::new();
        
        // 1. 添加模型
        let model_info = ModelInfo {
            name: "test-model".to_string(),
            path: PathBuf::from("test.gguf"),
            size_bytes: 1024 * 1024 * 1024,
            quantization: "Q4_K_M".to_string(),
            parameters: "7B".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        assert!(engine.add_model(model_info.clone()).await.is_ok());
        
        // 2. 列出模型
        let models = engine.list_models().await;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test-model");
        
        // 3. 获取配置
        let config = engine.get_config().await;
        assert!(config.enabled);
        assert_eq!(config.backend_type, BackendType::Allama);
        
        // 4. 获取状态
        let status = engine.get_status().await;
        assert_eq!(format!("{:?}", status), "Idle");
        
        // 5. 获取统计
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
    }

    #[tokio::test]
    async fn test_context_compression_config() {
        let engine = InferenceEngine::new();
        
        let compression_config = ContextCompressionConfig {
            enabled: true,
            compression_method: CompressionMethod::TokenQuantization,
            compression_ratio: 0.5,
            kv_cache_compression: true,
            quantization_bits: 4,
            sliding_window: true,
            sliding_window_size: 512,
        };
        
        let new_config = InferenceConfig {
            enabled: true,
            model_path: PathBuf::from("./allama/models"),
            backend_type: BackendType::Allama,
            max_context_length: 2048,
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            n_gpu_layers: 0,
            n_threads: 4,
            use_mmap: true,
            use_mlock: false,
            embedding_only: false,
            context_compression: compression_config,
        };
        
        engine.update_config(new_config).await;
        
        let retrieved_config = engine.get_config().await;
        assert!(retrieved_config.context_compression.enabled);
        assert_eq!(retrieved_config.context_compression.quantization_bits, 4);
    }

    #[tokio::test]
    async fn test_inference_request_creation() {
        let request = InferenceRequest {
            model: "test-model".to_string(),
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: Some(1.1),
            stop: None,
            stream: false,
        };
        
        assert_eq!(request.model, "test-model");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_chat_request_creation() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Hi!".to_string(),
                },
            ],
            max_tokens: Some(500),
            temperature: Some(0.7),
            top_p: None,
            stream: false,
        };
        
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "user");
    }

    #[tokio::test]
    async fn test_backend_type_comparison() {
        let llama_cpp = BackendType::LlamaCpp;
        let allama = BackendType::Allama;
        let candle = BackendType::Candle;
        
        assert_ne!(llama_cpp, allama);
        assert_eq!(allama, BackendType::Allama);
    }

    #[tokio::test]
    async fn test_compression_methods() {
        let methods = vec![
            CompressionMethod::TokenQuantization,
            CompressionMethod::KVCacheCompression,
            CompressionMethod::AttentionSink,
            CompressionMethod::StreamingLLM,
            CompressionMethod::Custom,
        ];
        
        assert_eq!(methods.len(), 5);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = InferenceConfig::default();
        
        assert!(config.enabled);
        assert_eq!(config.backend_type, BackendType::Allama);
        assert_eq!(config.max_context_length, 2048);
        assert_eq!(config.max_tokens, 512);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.top_k, 40);
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let engine = InferenceEngine::new();
        let stats = engine.get_stats().await;
        
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.total_tokens_generated, 0);
        assert!(stats.current_model.is_none());
        assert_eq!(stats.compression_stats.compression_enabled, false);
    }

    #[tokio::test]
    async fn test_model_info_serialization() {
        let model_info = ModelInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.gguf"),
            size_bytes: 1000,
            quantization: "Q4_K_M".to_string(),
            parameters: "7B".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        // 测试序列化和反序列化
        let json = serde_json::to_string(&model_info).unwrap();
        let deserialized: ModelInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.name, model_info.name);
        assert_eq!(deserialized.quantization, model_info.quantization);
    }
}
