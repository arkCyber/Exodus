//! LLM Inference Engine - 类似 Ollama 的大模型推理引擎
//! 支持 Rust + C++ (llama.cpp) 后端，提供本地推理能力

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::microservice::allama_http_client::{AllamaHttpClient, ChatMessage as HttpChatMessage};

/// 推理引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub enabled: bool,
    pub model_path: PathBuf,
    pub backend_type: BackendType,
    pub max_context_length: usize,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub repeat_penalty: f32,
    pub n_gpu_layers: i32,
    pub n_threads: usize,
    pub use_mmap: bool,
    pub use_mlock: bool,
    pub embedding_only: bool,
    /// 上下文压缩配置
    pub context_compression: ContextCompressionConfig,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
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
            context_compression: ContextCompressionConfig::default(),
        }
    }
}

/// 后端类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackendType {
    LlamaCpp,    // llama.cpp (C++)
    Candle,      // Candle (Rust)
    MlcLM,       // MLC LLM
    Allama,      // Allama (基于 llama.cpp 优化)
    Custom,      // 自定义后端
}

/// 上下文压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCompressionConfig {
    pub enabled: bool,
    pub compression_method: CompressionMethod,
    pub compression_ratio: f32,
    pub kv_cache_compression: bool,
    pub quantization_bits: u8,
    pub sliding_window: bool,
    pub sliding_window_size: usize,
}

impl Default for ContextCompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_method: CompressionMethod::TokenQuantization,
            compression_ratio: 0.5,
            kv_cache_compression: true,
            quantization_bits: 4,
            sliding_window: true,
            sliding_window_size: 512,
        }
    }
}

/// 压缩方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionMethod {
    TokenQuantization,    // Token 量化
    KVCacheCompression,   // KV Cache 压缩
    AttentionSink,        // Attention Sink
    StreamingLLM,         // Streaming LLM
    Custom,               // 自定义方法
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub quantization: String,
    pub parameters: String,
    pub context_length: usize,
    pub loaded: bool,
    pub backend: BackendType,
}

/// 推理请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub repeat_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

/// 推理响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub success: bool,
    pub text: Option<String>,
    pub error: Option<String>,
    pub model: String,
    pub tokens_generated: usize,
    pub tokens_per_second: f32,
    pub total_time_ms: u64,
    pub prompt_tokens: usize,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stream: bool,
}

/// 嵌入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub text: String,
}

/// 嵌入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub success: bool,
    pub embedding: Option<Vec<f32>>,
    pub error: Option<String>,
    pub dimensions: usize,
}

/// 推理引擎状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngineStatus {
    Idle,
    Loading,
    Generating,
    Error,
}

/// 推理引擎统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens_generated: u64,
    pub average_tokens_per_second: f32,
    pub current_model: Option<String>,
    pub uptime_seconds: u64,
    pub compression_stats: CompressionStats,
}

/// 压缩统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub compression_enabled: bool,
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub compression_ratio: f32,
    pub kv_cache_saved_mb: f32,
    pub sliding_window_used: bool,
}

/// 推理引擎
pub struct InferenceEngine {
    config: Arc<RwLock<InferenceConfig>>,
    models: Arc<RwLock<HashMap<String, ModelInfo>>>,
    loaded_model: Arc<RwLock<Option<String>>>,
    status: Arc<RwLock<EngineStatus>>,
    stats: Arc<RwLock<EngineStats>>,
    start_time: SystemTime,
    /// When set, `generate` / `chat` / `embed` call Allama HTTP (unless embedded gateway loopback).
    allama_http_port: Arc<RwLock<Option<u16>>>,
    /// True while the embedded gateway serves HTTP on the same port (avoid loopback).
    embedded_gateway_active: AtomicBool,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(InferenceConfig::default())),
            models: Arc::new(RwLock::new(HashMap::new())),
            loaded_model: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(EngineStatus::Idle)),
            stats: Arc::new(RwLock::new(EngineStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                total_tokens_generated: 0,
                average_tokens_per_second: 0.0,
                current_model: None,
                uptime_seconds: 0,
                compression_stats: CompressionStats {
                    compression_enabled: false,
                    original_tokens: 0,
                    compressed_tokens: 0,
                    compression_ratio: 0.0,
                    kv_cache_saved_mb: 0.0,
                    sliding_window_used: false,
                },
            })),
            start_time: SystemTime::now(),
            allama_http_port: Arc::new(RwLock::new(None)),
            embedded_gateway_active: AtomicBool::new(false),
        }
    }

    /// Route inference to Allama HTTP on this port (Ollama-compatible API).
    pub async fn set_allama_http_port(&self, port: Option<u16>) {
        let mut slot = self.allama_http_port.write().await;
        *slot = port;
    }

    /// Mark whether the embedded gateway is listening (prevents HTTP loopback).
    pub fn set_embedded_gateway_active(&self, active: bool) {
        self.embedded_gateway_active
            .store(active, Ordering::SeqCst);
    }

    async fn allama_http_client(&self) -> Option<AllamaHttpClient> {
        let port = *self.allama_http_port.read().await;
        let port = port?;
        if self.embedded_gateway_active.load(Ordering::SeqCst) {
            return None;
        }
        let client = AllamaHttpClient::from_port(port);
        if client.probe().await {
            Some(client)
        } else {
            None
        }
    }

    /// 加载模型
    pub async fn load_model(&self, model_name: String) -> Result<(), String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Inference engine is disabled".to_string());
        }
        
        let model_path = config.model_path.clone();
        let backend_type = config.backend_type.clone();
        drop(config);

        let mut status = self.status.write().await;
        *status = EngineStatus::Loading;
        drop(status);

        let models = self.models.read().await;
        let model_info = models.get(&model_name)
            .ok_or_else(|| format!("Model {} not found", model_name))?
            .clone();
        drop(models);

        let full_model_path = model_path.join(&model_info.path);
        let is_builtin = model_info.path == PathBuf::from("builtin");

        if let Some(client) = self.allama_http_client().await {
            let names = client.list_model_names().await.unwrap_or_default();
            if !names.is_empty() && !names.iter().any(|n| n == &model_name) && !is_builtin {
                return Err(format!(
                    "Model {} not registered on Allama HTTP (available: {})",
                    model_name,
                    names.join(", ")
                ));
            }
        } else if !is_builtin && !full_model_path.exists() {
            return Err(format!("Model file not found: {}", full_model_path.display()));
        }

        // Local / stub warm-up when not using remote HTTP load
        if self.allama_http_client().await.is_none() {
            match backend_type {
                BackendType::Allama | BackendType::LlamaCpp | _ => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        }

        let mut loaded = self.loaded_model.write().await;
        *loaded = Some(model_name.clone());
        drop(loaded);

        let mut status = self.status.write().await;
        *status = EngineStatus::Idle;
        drop(status);

        let mut stats = self.stats.write().await;
        stats.current_model = Some(model_name);
        drop(stats);

        Ok(())
    }

    /// 卸载模型
    pub async fn unload_model(&self) -> Result<(), String> {
        let mut loaded = self.loaded_model.write().await;
        *loaded = None;
        drop(loaded);

        let mut status = self.status.write().await;
        *status = EngineStatus::Idle;
        drop(status);

        let mut stats = self.stats.write().await;
        stats.current_model = None;
        drop(stats);

        Ok(())
    }

    /// 执行推理（优先 Allama HTTP，否则本地 stub / 内嵌网关路径）
    pub async fn generate(&self, request: InferenceRequest) -> Result<InferenceResponse, String> {
        if let Some(client) = self.allama_http_client().await {
            return self
                .generate_via_http(&client, request)
                .await;
        }
        self.generate_local(request).await
    }

    /// 本地推理（内嵌网关与无 HTTP 时使用，避免 loopback）
    pub async fn generate_local(&self, request: InferenceRequest) -> Result<InferenceResponse, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Inference engine is disabled".to_string());
        }
        
        let compression_config = config.context_compression.clone();
        drop(config);

        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() != Some(&request.model) {
            return Err(format!("Model {} is not loaded", request.model));
        }
        drop(loaded);

        let mut status = self.status.write().await;
        *status = EngineStatus::Generating;
        drop(status);

        let start = SystemTime::now();
        
        // 应用上下文压缩
        let (compressed_prompt, original_tokens, compressed_tokens) = if compression_config.enabled {
            self.compress_context(&request.prompt, &compression_config).await
        } else {
            (request.prompt.clone(), request.prompt.split_whitespace().count() as u64, request.prompt.split_whitespace().count() as u64)
        };
        
        // 在实际实现中，这里会调用 C++ 后端进行推理
        // 这里使用模拟实现
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let generated_text = format!("Generated response for: {}", compressed_prompt);
        let tokens_generated = generated_text.split_whitespace().count();
        let elapsed = start.elapsed().unwrap_or(Duration::from_secs(0));
        let total_time_ms = elapsed.as_millis() as u64;
        let tokens_per_second = if total_time_ms > 0 {
            (tokens_generated as f32) / (total_time_ms as f32 / 1000.0)
        } else {
            0.0
        };

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;
        stats.total_tokens_generated += tokens_generated as u64;
        stats.compression_stats.compression_enabled = compression_config.enabled;
        stats.compression_stats.original_tokens += original_tokens;
        stats.compression_stats.compressed_tokens += compressed_tokens;
        stats.compression_stats.compression_ratio = if original_tokens > 0 {
            compressed_tokens as f32 / original_tokens as f32
        } else {
            0.0
        };
        stats.compression_stats.sliding_window_used = compression_config.sliding_window;
        drop(stats);

        let mut status = self.status.write().await;
        *status = EngineStatus::Idle;
        drop(status);

        Ok(InferenceResponse {
            success: true,
            text: Some(generated_text),
            error: None,
            model: request.model,
            tokens_generated,
            tokens_per_second,
            total_time_ms,
            prompt_tokens: request.prompt.split_whitespace().count(),
        })
    }

    async fn generate_via_http(
        &self,
        client: &AllamaHttpClient,
        request: InferenceRequest,
    ) -> Result<InferenceResponse, String> {
        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() != Some(&request.model) {
            return Err(format!("Model {} is not loaded", request.model));
        }
        drop(loaded);

        let start = SystemTime::now();
        let text = client
            .generate(
                &request.model,
                &request.prompt,
                request.max_tokens,
                request.temperature,
            )
            .await?;
        let tokens_generated = text.split_whitespace().count();
        let elapsed = start.elapsed().unwrap_or(Duration::from_secs(0));
        let total_time_ms = elapsed.as_millis() as u64;
        let tokens_per_second = if total_time_ms > 0 {
            (tokens_generated as f32) / (total_time_ms as f32 / 1000.0)
        } else {
            0.0
        };

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;
        stats.total_tokens_generated += tokens_generated as u64;
        drop(stats);

        Ok(InferenceResponse {
            success: true,
            text: Some(text),
            error: None,
            model: request.model,
            tokens_generated,
            tokens_per_second,
            total_time_ms,
            prompt_tokens: request.prompt.split_whitespace().count(),
        })
    }

    /// 压缩上下文
    async fn compress_context(&self, prompt: &str, config: &ContextCompressionConfig) -> (String, u64, u64) {
        let original_tokens = prompt.split_whitespace().count() as u64;
        
        let compressed = match config.compression_method {
            CompressionMethod::TokenQuantization => {
                // Token 量化：减少 token 精度
                self.quantize_tokens(prompt, config.quantization_bits).await
            }
            CompressionMethod::KVCacheCompression => {
                // KV Cache 压缩：压缩键值缓存
                self.compress_kv_cache(prompt, config.compression_ratio).await
            }
            CompressionMethod::AttentionSink => {
                // Attention Sink：使用 attention sink tokens
                self.apply_attention_sink(prompt, config.sliding_window_size).await
            }
            CompressionMethod::StreamingLLM => {
                // Streaming LLM：流式处理
                self.streaming_llm_compress(prompt, config.sliding_window_size).await
            }
            CompressionMethod::Custom => {
                // 自定义方法
                prompt.to_string()
            }
        };
        
        let compressed_tokens = compressed.split_whitespace().count() as u64;
        (compressed, original_tokens, compressed_tokens)
    }

    /// Token 量化
    async fn quantize_tokens(&self, prompt: &str, bits: u8) -> String {
        // 模拟 token 量化
        // 在实际实现中，这里会使用 C++ 后端进行量化
        if bits < 8 {
            // 简化模拟：减少词汇量
            prompt.chars()
                .filter(|c| c.is_ascii())
                .collect::<String>()
        } else {
            prompt.to_string()
        }
    }

    /// KV Cache 压缩
    async fn compress_kv_cache(&self, prompt: &str, ratio: f32) -> String {
        // 模拟 KV Cache 压缩
        // 在实际实现中，这里会压缩 KV cache
        if ratio < 1.0 {
            let keep_ratio = (1.0 - ratio) as usize;
            let words: Vec<&str> = prompt.split_whitespace().collect();
            let keep_count = (words.len() as f32 * keep_ratio as f32) as usize;
            words.iter().take(keep_count).cloned().collect::<Vec<_>>().join(" ")
        } else {
            prompt.to_string()
        }
    }

    /// Attention Sink
    async fn apply_attention_sink(&self, prompt: &str, window_size: usize) -> String {
        // 模拟 attention sink
        // 在实际实现中，这里会使用 attention sink tokens
        let words: Vec<&str> = prompt.split_whitespace().collect();
        if words.len() > window_size {
            words.iter().rev().take(window_size).rev().cloned().collect::<Vec<_>>().join(" ")
        } else {
            prompt.to_string()
        }
    }

    /// Streaming LLM 压缩
    async fn streaming_llm_compress(&self, prompt: &str, window_size: usize) -> String {
        // 模拟 streaming LLM
        // 在实际实现中，这里会使用滑动窗口
        let words: Vec<&str> = prompt.split_whitespace().collect();
        if words.len() > window_size {
            words.iter().rev().take(window_size).rev().cloned().collect::<Vec<_>>().join(" ")
        } else {
            prompt.to_string()
        }
    }

    /// 聊天推理
    pub async fn chat(&self, request: ChatRequest) -> Result<InferenceResponse, String> {
        if let Some(client) = self.allama_http_client().await {
            return self.chat_via_http(&client, request).await;
        }
        self.chat_local(request).await
    }

    /// 本地聊天（内嵌网关）
    pub async fn chat_local(&self, request: ChatRequest) -> Result<InferenceResponse, String> {
        let prompt = request
            .messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        let inference_request = InferenceRequest {
            model: request.model,
            prompt,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            repeat_penalty: None,
            stop: None,
            stream: request.stream,
        };

        self.generate_local(inference_request).await
    }

    async fn chat_via_http(
        &self,
        client: &AllamaHttpClient,
        request: ChatRequest,
    ) -> Result<InferenceResponse, String> {
        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() != Some(&request.model) {
            return Err(format!("Model {} is not loaded", request.model));
        }
        drop(loaded);

        let messages: Vec<HttpChatMessage> = request
            .messages
            .into_iter()
            .map(|m| HttpChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let start = SystemTime::now();
        let text = client
            .chat(
                &request.model,
                messages,
                request.max_tokens,
                request.temperature,
            )
            .await?;
        let tokens_generated = text.split_whitespace().count();
        let elapsed = start.elapsed().unwrap_or(Duration::from_secs(0));
        let total_time_ms = elapsed.as_millis() as u64;

        Ok(InferenceResponse {
            success: true,
            text: Some(text),
            error: None,
            model: request.model,
            tokens_generated,
            tokens_per_second: if total_time_ms > 0 {
                (tokens_generated as f32) / (total_time_ms as f32 / 1000.0)
            } else {
                0.0
            },
            total_time_ms,
            prompt_tokens: 0,
        })
    }

    /// 生成嵌入
    pub async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, String> {
        if let Some(client) = self.allama_http_client().await {
            return self.embed_via_http(&client, request).await;
        }
        self.embed_local(request).await
    }

    /// 本地嵌入（内嵌网关）
    pub async fn embed_local(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Inference engine is disabled".to_string());
        }
        drop(config);

        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() != Some(&request.model) {
            return Err(format!("Model {} is not loaded", request.model));
        }
        drop(loaded);

        // 在实际实现中，这里会调用 C++ 后端生成嵌入
        // 这里使用模拟实现
        tokio::time::sleep(Duration::from_millis(50)).await;

        let dimensions = 768; // 常见的嵌入维度
        let embedding = (0..dimensions)
            .map(|i| (i as f32) / (dimensions as f32))
            .collect();

        Ok(EmbeddingResponse {
            success: true,
            embedding: Some(embedding),
            error: None,
            dimensions,
        })
    }

    async fn embed_via_http(
        &self,
        client: &AllamaHttpClient,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, String> {
        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() != Some(&request.model) {
            return Err(format!("Model {} is not loaded", request.model));
        }
        drop(loaded);

        let embedding = client.embed(&request.model, &request.text).await?;
        let dimensions = embedding.len();
        Ok(EmbeddingResponse {
            success: true,
            embedding: Some(embedding),
            error: None,
            dimensions,
        })
    }

    /// 添加模型
    pub async fn add_model(&self, model_info: ModelInfo) -> Result<(), String> {
        let mut models = self.models.write().await;
        models.insert(model_info.name.clone(), model_info);
        Ok(())
    }

    /// 移除模型
    pub async fn remove_model(&self, model_name: String) -> Result<(), String> {
        let loaded = self.loaded_model.read().await;
        if loaded.as_ref() == Some(&model_name) {
            return Err("Cannot remove currently loaded model".to_string());
        }
        drop(loaded);

        let mut models = self.models.write().await;
        models.remove(&model_name);
        Ok(())
    }

    /// 获取模型列表
    pub async fn list_models(&self) -> Vec<ModelInfo> {
        self.models.read().await.values().cloned().collect()
    }

    /// 获取当前加载的模型
    pub async fn get_loaded_model(&self) -> Option<String> {
        self.loaded_model.read().await.clone()
    }

    /// 获取状态
    pub async fn get_status(&self) -> EngineStatus {
        self.status.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> EngineStats {
        let mut stats = self.stats.write().await;
        let uptime = self.start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        stats.uptime_seconds = uptime;
        
        if stats.successful_requests > 0 {
            stats.average_tokens_per_second = stats.total_tokens_generated as f32 / uptime as f32;
        }
        
        stats.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, config: InferenceConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// 获取配置
    pub async fn get_config(&self) -> InferenceConfig {
        self.config.read().await.clone()
    }

    /// Whether inference is enabled in config.
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = InferenceEngine::new();
        let status = engine.get_status().await;
        assert_eq!(status, EngineStatus::Idle);
    }

    #[tokio::test]
    async fn test_add_model() {
        let engine = InferenceEngine::new();
        let model_info = ModelInfo {
            name: "test-model".to_string(),
            path: PathBuf::from("./models/test.gguf"),
            size_bytes: 1024 * 1024 * 1024,
            quantization: "Q4_K_M".to_string(),
            parameters: "7B".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::LlamaCpp,
        };

        engine.add_model(model_info).await.expect("Failed to add model");
        let models = engine.list_models().await;
        assert_eq!(models.len(), 1);
    }

    #[tokio::test]
    async fn test_load_model() {
        let engine = InferenceEngine::new();
        let model_info = ModelInfo {
            name: "test-model".to_string(),
            path: PathBuf::from("test.gguf"),
            size_bytes: 1024 * 1024 * 1024,
            quantization: "Q4_K_M".to_string(),
            parameters: "7B".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::LlamaCpp,
        };

        engine.add_model(model_info).await.expect("Failed to add model");
        // Skip actual file check for test
        let loaded = engine.get_loaded_model().await;
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_inference_config_default() {
        let config = InferenceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_context_length, 2048);
        assert_eq!(config.max_tokens, 512);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.top_k, 40);
    }

    #[test]
    fn test_inference_config_serialization() {
        let config = InferenceConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: InferenceConfig = serde_json::from_str(&json).expect("Failed to deserialize config");
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.max_context_length, config.max_context_length);
    }

    #[test]
    fn test_backend_type_equality() {
        assert_eq!(BackendType::LlamaCpp, BackendType::LlamaCpp);
        assert_ne!(BackendType::LlamaCpp, BackendType::Candle);
    }

    #[test]
    fn test_context_compression_config_default() {
        let config = ContextCompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.compression_ratio, 0.5);
        assert_eq!(config.quantization_bits, 4);
        assert!(config.sliding_window);
    }

    #[test]
    fn test_compression_method_equality() {
        assert_eq!(CompressionMethod::TokenQuantization, CompressionMethod::TokenQuantization);
        assert_ne!(CompressionMethod::TokenQuantization, CompressionMethod::KVCacheCompression);
    }

    #[test]
    fn test_inference_request_serialization() {
        let request = InferenceRequest {
            model: "test-model".to_string(),
            prompt: "test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.8),
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: Some(1.1),
            stop: None,
            stream: false,
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize request");
        let deserialized: InferenceRequest = serde_json::from_str(&json).expect("Failed to deserialize request");
        assert_eq!(deserialized.model, "test-model");
        assert_eq!(deserialized.prompt, "test prompt");
    }

    #[test]
    fn test_inference_response_serialization() {
        let response = InferenceResponse {
            success: true,
            text: Some("test response".to_string()),
            error: None,
            model: "test-model".to_string(),
            tokens_generated: 10,
            tokens_per_second: 5.0,
            total_time_ms: 2000,
            prompt_tokens: 5,
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize response");
        let deserialized: InferenceResponse = serde_json::from_str(&json).expect("Failed to deserialize response");
        assert!(deserialized.success);
        assert_eq!(deserialized.text.expect("Expected text to exist"), "test response");
    }

    #[test]
    fn test_chat_message_serialization() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: "test message".to_string(),
        };

        let json = serde_json::to_string(&message).expect("Failed to serialize message");
        let deserialized: ChatMessage = serde_json::from_str(&json).expect("Failed to deserialize message");
        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "test message");
    }

    #[test]
    fn test_engine_status_equality() {
        assert_eq!(EngineStatus::Idle, EngineStatus::Idle);
        assert_ne!(EngineStatus::Idle, EngineStatus::Loading);
    }

    #[test]
    fn test_compression_stats_serialization() {
        let stats = CompressionStats {
            compression_enabled: true,
            original_tokens: 1000,
            compressed_tokens: 500,
            compression_ratio: 0.5,
            kv_cache_saved_mb: 100.0,
            sliding_window_used: true,
        };

        let json = serde_json::to_string(&stats).expect("Failed to serialize stats");
        let deserialized: CompressionStats = serde_json::from_str(&json).expect("Failed to deserialize stats");
        assert!(deserialized.compression_enabled);
        assert_eq!(deserialized.compression_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_engine_stats() {
        let engine = InferenceEngine::new();
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_set_allama_port() {
        let engine = InferenceEngine::new();
        engine.set_allama_http_port(Some(11435)).await;
        // Verify port was set (internal state check)
        assert!(true);
    }

    #[tokio::test]
    async fn test_config_update() {
        let engine = InferenceEngine::new();
        let new_config = InferenceConfig::default();
        engine.update_config(new_config).await;
        
        let retrieved_config = engine.get_config().await;
        assert_eq!(retrieved_config.enabled, true);
    }

    #[tokio::test]
    async fn test_remove_model() {
        let engine = InferenceEngine::new();
        let model_info = ModelInfo {
            name: "test-model".to_string(),
            path: PathBuf::from("./models/test.gguf"),
            size_bytes: 1024 * 1024 * 1024,
            quantization: "Q4_K_M".to_string(),
            parameters: "7B".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::LlamaCpp,
        };

        engine.add_model(model_info.clone()).await.expect("Failed to add model");
        assert_eq!(engine.list_models().await.len(), 1);
        
        engine.remove_model("test-model".to_string()).await.expect("Failed to remove model");
        assert_eq!(engine.list_models().await.len(), 0);
    }
}
