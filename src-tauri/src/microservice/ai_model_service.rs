//! AI Model Sharing Service - Distributed AI model registry and sharing
//! 
//! This service provides AI model discovery, registration, and sharing capabilities
//! using the p2p-blobs and p2p-gossip services for distributed model distribution.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// AI Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelMetadata {
    pub model_id: String,
    pub name: String,
    pub version: String,
    pub framework: String, // "pytorch", "tensorflow", "onnx", etc.
    pub task_type: String, // "text-generation", "image-generation", "translation", etc.
    pub size_bytes: u64,
    pub parameters: Option<u64>, // Number of parameters for LLMs
    pub description: String,
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,
    pub blob_hash: Option<String>, // Reference to p2p-blobs storage
    pub node_id: String,
    pub registered_at: u64,
    pub last_updated: u64,
}

/// Model registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistration {
    pub metadata: AiModelMetadata,
    pub blob_hash: String, // Hash of model data in p2p-blobs
}

/// Model search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSearchQuery {
    pub task_type: Option<String>,
    pub framework: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub limit: Option<usize>,
}

/// Configuration for AI Model Service
#[derive(Debug, Clone)]
pub struct AiModelServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for AiModelServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_ai_model.sock");
        Self { socket_path }
    }
}

/// AI Model Service
pub struct AiModelService {
    config: AiModelServiceConfig,
    models: Arc<Mutex<HashMap<String, AiModelMetadata>>>, // model_id -> metadata
    node_models: Arc<Mutex<HashMap<String, Vec<String>>>>, // node_id -> model_ids
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl AiModelService {
    pub fn new(config: AiModelServiceConfig) -> Self {
        Self {
            config,
            models: Arc::new(Mutex::new(HashMap::new())),
            node_models: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }

        let socket_path = self.config.socket_path.clone();
        let models = Arc::clone(&self.models);
        let node_models = Arc::clone(&self.node_models);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let models = Arc::clone(&models);
                                let node_models = Arc::clone(&node_models);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, models, node_models, node_id).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        println!("AI Model Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }

        if let Some(tx) = self.shutdown_tx.lock().ok().and_then(|mut tx| tx.take()) {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("AI Model Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Register a model
    #[allow(dead_code)]
    pub fn register_model(&self, registration: ModelRegistration) -> Result<(), String> {
        let model_id = registration.metadata.model_id.clone();
        let node_id = registration.metadata.node_id.clone();
        
        let mut models = self.models.lock().map_err(|e| format!("Lock error: {}", e))?;
        models.insert(model_id.clone(), registration.metadata);
        drop(models);

        let mut node_models = self.node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
        node_models.entry(node_id).or_insert_with(Vec::new).push(model_id);
        
        Ok(())
    }

    /// Unregister a model
    #[allow(dead_code)]
    pub fn unregister_model(&self, model_id: String) -> Result<(), String> {
        let mut models = self.models.lock().map_err(|e| format!("Lock error: {}", e))?;
        let metadata = models.remove(&model_id);
        drop(models);

        if let Some(meta) = metadata {
            let mut node_models = self.node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(model_ids) = node_models.get_mut(&meta.node_id) {
                model_ids.retain(|id| id != &model_id);
                if model_ids.is_empty() {
                    node_models.remove(&meta.node_id);
                }
            }
        }

        Ok(())
    }

    /// Get model metadata
    #[allow(dead_code)]
    pub fn get_model(&self, model_id: String) -> Option<AiModelMetadata> {
        let models = self.models.lock().ok()?;
        models.get(&model_id).cloned()
    }

    /// Search models
    #[allow(dead_code)]
    pub fn search_models(&self, query: ModelSearchQuery) -> Vec<AiModelMetadata> {
        let models = self.models.lock();
        let mut results: Vec<AiModelMetadata> = models.as_ref()
            .ok()
            .map(|models| models.values()
            .filter(|m| {
                if let Some(task_type) = &query.task_type {
                    if &m.task_type != task_type {
                        return false;
                    }
                }
                if let Some(framework) = &query.framework {
                    if &m.framework != framework {
                        return false;
                    }
                }
                if let Some(min_size) = query.min_size {
                    if m.size_bytes < min_size {
                        return false;
                    }
                }
                if let Some(max_size) = query.max_size {
                    if m.size_bytes > max_size {
                        return false;
                    }
                }
                if let Some(author) = &query.author {
                    if &m.author != author {
                        return false;
                    }
                }
                if let Some(tags) = &query.tags {
                    if !tags.iter().all(|tag| m.tags.contains(tag)) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect())
            .unwrap_or_default();

        results.sort_by(|a, b| b.registered_at.cmp(&a.registered_at));
        
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// List all models
    #[allow(dead_code)]
    pub fn list_models(&self) -> Vec<AiModelMetadata> {
        self.models.lock()
            .map(|models| models.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get models by node
    #[allow(dead_code)]
    pub fn get_node_models(&self, node_id: String) -> Vec<AiModelMetadata> {
        let node_models = self.node_models.lock();
        let models = self.models.lock();
        
        if let (Ok(node_models), Ok(models)) = (node_models, models) {
            if let Some(model_ids) = node_models.get(&node_id) {
                model_ids.iter()
                    .filter_map(|id| models.get(id).cloned())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// List all nodes
    #[allow(dead_code)]
    pub fn list_nodes(&self) -> Vec<String> {
        self.node_models.lock()
            .map(|node_models| node_models.keys().cloned().collect())
            .unwrap_or_default()
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    models: Arc<Mutex<HashMap<String, AiModelMetadata>>>,
    node_models: Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

        let result = match method {
            "register_model" => handle_register_model(&params, &models, &node_models).await,
            "unregister_model" => handle_unregister_model(&params, &models, &node_models).await,
            "get_model" => handle_get_model(&params, &models).await,
            "search_models" => handle_search_models(&params, &models).await,
            "list_models" => handle_list_models(&models).await,
            "get_node_models" => handle_get_node_models(&params, &node_models, &models).await,
            "list_nodes" => handle_list_nodes(&node_models).await,
            "node_info" => handle_node_info(&node_id).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": result.unwrap_err()},
                "id": id
            })
        };

        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

async fn handle_register_model(
    params: &serde_json::Value,
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
    node_models: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let registration: ModelRegistration = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid registration: {}", e))?;
    
    let model_id = registration.metadata.model_id.clone();
    let node_id = registration.metadata.node_id.clone();
    
    let mut guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(model_id.clone(), registration.metadata);
    drop(guard);

    let mut node_guard = node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
    node_guard.entry(node_id).or_insert_with(Vec::new).push(model_id.clone());

    Ok(json!({
        "registered": true,
        "model_id": model_id
    }))
}

async fn handle_unregister_model(
    params: &serde_json::Value,
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
    node_models: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let model_id = params.get("model_id").and_then(|m| m.as_str()).ok_or("Missing model_id")?;
    
    let mut guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.remove(model_id);
    drop(guard);

    if let Some(meta) = metadata {
        let mut node_guard = node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(model_ids) = node_guard.get_mut(&meta.node_id) {
            model_ids.retain(|id| id != model_id);
            if model_ids.is_empty() {
                node_guard.remove(&meta.node_id);
            }
        }
    }

    Ok(json!({
        "unregistered": true,
        "model_id": model_id
    }))
}

async fn handle_get_model(
    params: &serde_json::Value,
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
) -> Result<serde_json::Value, String> {
    let model_id = params.get("model_id").and_then(|m| m.as_str()).ok_or("Missing model_id")?;
    
    let guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(model_id)
        .map(|m| json!(m))
        .ok_or_else(|| "Model not found".to_string())
}

async fn handle_search_models(
    params: &serde_json::Value,
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
) -> Result<serde_json::Value, String> {
    let query: ModelSearchQuery = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid query: {}", e))?;
    
    let guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut results: Vec<AiModelMetadata> = guard.values()
        .filter(|m| {
            if let Some(task_type) = &query.task_type {
                if &m.task_type != task_type {
                    return false;
                }
            }
            if let Some(framework) = &query.framework {
                if &m.framework != framework {
                    return false;
                }
            }
            if let Some(min_size) = query.min_size {
                if m.size_bytes < min_size {
                    return false;
                }
            }
            if let Some(max_size) = query.max_size {
                if m.size_bytes > max_size {
                    return false;
                }
            }
            if let Some(author) = &query.author {
                if &m.author != author {
                    return false;
                }
            }
            if let Some(tags) = &query.tags {
                if !tags.iter().all(|tag| m.tags.contains(tag)) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.registered_at.cmp(&a.registered_at));
    
    if let Some(limit) = query.limit {
        results.truncate(limit);
    }

    Ok(json!({
        "results": results
    }))
}

async fn handle_list_models(
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let model_list: Vec<AiModelMetadata> = guard.values().cloned().collect();
    
    Ok(json!({
        "models": model_list
    }))
}

async fn handle_get_node_models(
    params: &serde_json::Value,
    node_models: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    models: &Arc<Mutex<HashMap<String, AiModelMetadata>>>,
) -> Result<serde_json::Value, String> {
    let node_id = params.get("node_id").and_then(|n| n.as_str()).ok_or("Missing node_id")?;
    
    let node_guard = node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let model_ids = node_guard.get(node_id).cloned().unwrap_or_default();
    drop(node_guard);

    let guard = models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let model_list: Vec<AiModelMetadata> = model_ids.iter()
        .filter_map(|id| guard.get(id).cloned())
        .collect();
    
    Ok(json!({
        "models": model_list
    }))
}

async fn handle_list_nodes(
    node_models: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let guard = node_models.lock().map_err(|e| format!("Lock error: {}", e))?;
    let node_list: Vec<String> = guard.keys().cloned().collect();
    
    Ok(json!({
        "nodes": node_list
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("ai_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_model() {
        let config = AiModelServiceConfig::default();
        let service = AiModelService::new(config);
        
        let metadata = AiModelMetadata {
            model_id: "test-model-1".to_string(),
            name: "Test Model".to_string(),
            version: "1.0.0".to_string(),
            framework: "pytorch".to_string(),
            task_type: "text-generation".to_string(),
            size_bytes: 1024 * 1024,
            parameters: Some(1000000),
            description: "A test model".to_string(),
            author: "test".to_string(),
            license: "MIT".to_string(),
            tags: vec!["test".to_string(), "llm".to_string()],
            blob_hash: Some("abc123".to_string()),
            node_id: service.node_id().to_string(),
            registered_at: current_timestamp(),
            last_updated: current_timestamp(),
        };

        let registration = ModelRegistration {
            metadata,
            blob_hash: "abc123".to_string(),
        };

        service.register_model(registration).expect("Failed to register model");
        let retrieved = service.get_model("test-model-1".to_string()).expect("Failed to get model");
        assert_eq!(retrieved.name, "Test Model");
    }

    #[tokio::test]
    async fn test_search_models() {
        let config = AiModelServiceConfig::default();
        let service = AiModelService::new(config);
        
        let metadata = AiModelMetadata {
            model_id: "test-model-2".to_string(),
            name: "Test Model 2".to_string(),
            version: "1.0.0".to_string(),
            framework: "pytorch".to_string(),
            task_type: "image-generation".to_string(),
            size_bytes: 1024 * 1024,
            parameters: Some(1000000),
            description: "A test model".to_string(),
            author: "test".to_string(),
            license: "MIT".to_string(),
            tags: vec!["test".to_string()],
            blob_hash: Some("abc123".to_string()),
            node_id: service.node_id().to_string(),
            registered_at: current_timestamp(),
            last_updated: current_timestamp(),
        };

        let registration = ModelRegistration {
            metadata,
            blob_hash: "abc123".to_string(),
        };

        service.register_model(registration).expect("Failed to register model");
        
        let query = ModelSearchQuery {
            task_type: Some("image-generation".to_string()),
            framework: None,
            min_size: None,
            max_size: None,
            tags: None,
            author: None,
            limit: None,
        };

        let results = service.search_models(query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_type, "image-generation");
    }
}
