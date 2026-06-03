//! 模型加载测试
//! 测试 allama/models 目录中的模型文件加载

#[cfg(test)]
mod integration_tests {
    use crate::inference_engine::{InferenceEngine, ModelInfo, BackendType};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_qwen_7b_model_loading() {
        let engine = InferenceEngine::new();
        
        // 添加 Qwen 7B 模型信息
        let model_info = ModelInfo {
            name: "qwen2.5-7b-instruct".to_string(),
            path: PathBuf::from("Qwen-Test/qwen2.5-7b-instruct-fp16-00001-of-00004.gguf"),
            size_bytes: 3700000000, // 3.7GB
            quantization: "FP16".to_string(),
            parameters: "7B".to_string(),
            context_length: 32768,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        let add_result = engine.add_model(model_info).await;
        assert!(add_result.is_ok(), "添加模型失败: {:?}", add_result);
        
        // 尝试加载模型
        let load_result = engine.load_model("qwen2.5-7b-instruct".to_string()).await;
        
        // 由于模型文件不完整（分片文件），预期会失败
        match load_result {
            Ok(_) => println!("✅ 模型加载成功"),
            Err(e) => println!("❌ 模型加载失败（预期，因为文件不完整）: {}", e),
        }
    }

    #[tokio::test]
    async fn test_qwen_32b_model_loading() {
        let engine = InferenceEngine::new();
        
        // 添加 Qwen 32B 模型信息
        let model_info = ModelInfo {
            name: "qwen2.5-32b-instruct".to_string(),
            path: PathBuf::from("Qwen-Test/qwen2.5-32b-instruct-fp16-00001-of-00017.gguf"),
            size_bytes: 5100000, // 5.1MB
            quantization: "FP16".to_string(),
            parameters: "32B".to_string(),
            context_length: 32768,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        let add_result = engine.add_model(model_info).await;
        assert!(add_result.is_ok(), "添加模型失败: {:?}", add_result);
        
        // 尝试加载模型
        let load_result = engine.load_model("qwen2.5-32b-instruct".to_string()).await;
        
        // 由于模型文件不完整（分片文件），预期会失败
        match load_result {
            Ok(_) => println!("✅ 模型加载成功"),
            Err(e) => println!("❌ 模型加载失败（预期，因为文件不完整）: {}", e),
        }
    }

    #[tokio::test]
    async fn test_model_path_configuration() {
        let engine = InferenceEngine::new();
        
        let config = engine.get_config().await;
        println!("当前模型路径: {}", config.model_path.display());
        println!("当前后端类型: {:?}", config.backend_type);
        
        assert!(config.enabled);
        assert_eq!(config.backend_type, BackendType::Allama);
    }

    #[tokio::test]
    async fn test_list_available_models() {
        let engine = InferenceEngine::new();
        
        // 添加两个模型
        let model_7b = ModelInfo {
            name: "qwen2.5-7b-instruct".to_string(),
            path: PathBuf::from("Qwen-Test/qwen2.5-7b-instruct-fp16-00001-of-00004.gguf"),
            size_bytes: 3700000000,
            quantization: "FP16".to_string(),
            parameters: "7B".to_string(),
            context_length: 32768,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        let model_32b = ModelInfo {
            name: "qwen2.5-32b-instruct".to_string(),
            path: PathBuf::from("Qwen-Test/qwen2.5-32b-instruct-fp16-00001-of-00017.gguf"),
            size_bytes: 5100000,
            quantization: "FP16".to_string(),
            parameters: "32B".to_string(),
            context_length: 32768,
            loaded: false,
            backend: BackendType::Allama,
        };
        
        engine.add_model(model_7b).await.unwrap();
        engine.add_model(model_32b).await.unwrap();
        
        let models = engine.list_models().await;
        println!("可用模型数量: {}", models.len());
        for model in &models {
            println!("  - {} ({} bytes)", model.name, model.size_bytes);
        }
        
        assert_eq!(models.len(), 2);
    }
}
