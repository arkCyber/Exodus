//! AI Video Stream Analysis Service
//! 
//! This service provides AI-powered video stream analysis capabilities,
//! enabling real-time object detection, scene recognition, and action analysis.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

use std::time::Duration;
/// Analysis task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTask {
    pub task_id: String,
    pub stream_id: String,
    pub analysis_type: String, // "object_detection", "scene_recognition", "action_analysis", "face_detection"
    pub model_id: String,
    pub enabled: bool,
    pub confidence_threshold: f32,
    pub created_at: u64,
}

/// Detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub result_id: String,
    pub task_id: String,
    pub timestamp: u64,
    pub detections: Vec<Detection>,
    pub confidence: f32,
}

/// Single detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub label: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub attributes: HashMap<String, String>,
}

/// Bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStats {
    pub task_id: String,
    pub total_frames_processed: u64,
    pub total_detections: u64,
    pub average_confidence: f32,
    pub processing_time_ms: f64,
}

/// Configuration for AI Video Analysis Service
#[derive(Debug, Clone)]
pub struct AiVideoAnalysisServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for AiVideoAnalysisServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_ai_video_analysis.sock");
        Self { socket_path }
    }
}

/// AI Video Analysis Service
pub struct AiVideoAnalysisService {
    config: AiVideoAnalysisServiceConfig,
    tasks: Arc<Mutex<HashMap<String, AnalysisTask>>>, // task_id -> task
    results: Arc<Mutex<HashMap<String, Vec<DetectionResult>>>>, // task_id -> results
    stats: Arc<Mutex<HashMap<String, AnalysisStats>>>, // task_id -> stats
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl AiVideoAnalysisService {
    pub fn new(config: AiVideoAnalysisServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
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
        let tasks = Arc::clone(&self.tasks);
        let results = Arc::clone(&self.results);
        let stats = Arc::clone(&self.stats);
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
                                let tasks = Arc::clone(&tasks);
                                let results = Arc::clone(&results);
                                let stats = Arc::clone(&stats);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, tasks, results, stats, node_id).await;
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

        println!("AI Video Analysis Service started on {:?}", socket_path);
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

        println!("AI Video Analysis Service stopped");
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

    /// Create an analysis task
    #[allow(dead_code)]
    pub fn create_task(&self, task: AnalysisTask) -> Result<(), String> {
        let task_id = task.task_id.clone();
        let mut tasks = self.tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
        tasks.insert(task_id.clone(), task);
        
        // Initialize stats
        let mut stats_guard = self.stats.lock().map_err(|e| format!("Lock error: {}", e))?;
        stats_guard.insert(task_id.clone(), AnalysisStats {
            task_id: task_id.clone(),
            total_frames_processed: 0,
            total_detections: 0,
            average_confidence: 0.0,
            processing_time_ms: 0.0,
        });
        
        Ok(())
    }

    /// Delete an analysis task
    #[allow(dead_code)]
    pub fn delete_task(&self, task_id: String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
        tasks.remove(&task_id);
        
        let mut results = self.results.lock().map_err(|e| format!("Lock error: {}", e))?;
        results.remove(&task_id);
        
        let mut stats = self.stats.lock().map_err(|e| format!("Lock error: {}", e))?;
        stats.remove(&task_id);
        
        Ok(())
    }

    /// Add detection result
    #[allow(dead_code)]
    pub fn add_detection_result(&self, result: DetectionResult) -> Result<(), String> {
        let task_id = result.task_id.clone();
        let mut results = self.results.lock().map_err(|e| format!("Lock error: {}", e))?;
        results.entry(task_id.clone()).or_insert_with(Vec::new).push(result.clone());
        
        // Keep only last 1000 results
        if let Some(task_results) = results.get_mut(&task_id) {
            if task_results.len() > 1000 {
                task_results.drain(0..task_results.len() - 1000);
            }
        }
        
        // Update stats
        let mut stats_guard = self.stats.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(stat) = stats_guard.get_mut(&task_id) {
            stat.total_frames_processed += 1;
            stat.total_detections += result.detections.len() as u64;
            
            // Update average confidence
            if stat.total_detections > 0 {
                let total_confidence: f32 = result.detections.iter().map(|d| d.confidence).sum();
                stat.average_confidence = (stat.average_confidence * (stat.total_detections - result.detections.len() as u64) as f32 + total_confidence) / stat.total_detections as f32;
            }
        }
        
        Ok(())
    }

    /// Get detection results
    #[allow(dead_code)]
    pub fn get_detection_results(&self, task_id: String, limit: Option<usize>) -> Vec<DetectionResult> {
        let results = self.results.lock();
        let task_results = results.as_ref().ok().and_then(|r| r.get(&task_id).cloned()).unwrap_or_default();
        
        if let Some(limit) = limit {
            task_results.into_iter().rev().take(limit).collect()
        } else {
            task_results.into_iter().rev().collect()
        }
    }

    /// Get task info
    #[allow(dead_code)]
    pub fn get_task(&self, task_id: String) -> Option<AnalysisTask> {
        let tasks = self.tasks.lock().ok()?;
        tasks.get(&task_id).cloned()
    }

    /// List all tasks
    #[allow(dead_code)]
    pub fn list_tasks(&self) -> Vec<AnalysisTask> {
        self.tasks.lock()
            .map(|tasks| tasks.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get analysis stats
    #[allow(dead_code)]
    pub fn get_stats(&self, task_id: String) -> Option<AnalysisStats> {
        let stats = self.stats.lock().ok()?;
        stats.get(&task_id).cloned()
    }

    /// Enable task
    #[allow(dead_code)]
    pub fn enable_task(&self, task_id: String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.enabled = true;
        }
        Ok(())
    }

    /// Disable task
    #[allow(dead_code)]
    pub fn disable_task(&self, task_id: String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.enabled = false;
        }
        Ok(())
    }
}

fn generate_node_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    tasks: Arc<Mutex<HashMap<String, AnalysisTask>>>,
    results: Arc<Mutex<HashMap<String, Vec<DetectionResult>>>>,
    stats: Arc<Mutex<HashMap<String, AnalysisStats>>>,
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
            "create_task" => handle_create_task(&params, &tasks, &stats).await,
            "delete_task" => handle_delete_task(&params, &tasks, &results, &stats).await,
            "add_detection_result" => handle_add_detection_result(&params, &results, &stats).await,
            "get_detection_results" => handle_get_detection_results(&params, &results).await,
            "get_task" => handle_get_task(&params, &tasks).await,
            "list_tasks" => handle_list_tasks(&tasks).await,
            "get_stats" => handle_get_stats(&params, &stats).await,
            "enable_task" => handle_enable_task(&params, &tasks).await,
            "disable_task" => handle_disable_task(&params, &tasks).await,
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

async fn handle_create_task(
    params: &serde_json::Value,
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
    stats: &Arc<Mutex<HashMap<String, AnalysisStats>>>,
) -> Result<serde_json::Value, String> {
    let task: AnalysisTask = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid task: {}", e))?;
    
    let task_id = task.task_id.clone();
    
    let mut tasks_guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    tasks_guard.insert(task_id.clone(), task.clone());
    
    let mut stats_guard = stats.lock().map_err(|e| format!("Lock error: {}", e))?;
    stats_guard.insert(task_id.clone(), AnalysisStats {
        task_id: task_id.clone(),
        total_frames_processed: 0,
        total_detections: 0,
        average_confidence: 0.0,
        processing_time_ms: 0.0,
    });

    Ok(json!({
        "task_id": task_id
    }))
}

async fn handle_delete_task(
    params: &serde_json::Value,
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
    results: &Arc<Mutex<HashMap<String, Vec<DetectionResult>>>>,
    stats: &Arc<Mutex<HashMap<String, AnalysisStats>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    
    let mut tasks_guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    tasks_guard.remove(task_id);
    
    let mut results_guard = results.lock().map_err(|e| format!("Lock error: {}", e))?;
    results_guard.remove(task_id);
    
    let mut stats_guard = stats.lock().map_err(|e| format!("Lock error: {}", e))?;
    stats_guard.remove(task_id);

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_add_detection_result(
    params: &serde_json::Value,
    results: &Arc<Mutex<HashMap<String, Vec<DetectionResult>>>>,
    stats: &Arc<Mutex<HashMap<String, AnalysisStats>>>,
) -> Result<serde_json::Value, String> {
    let result: DetectionResult = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid result: {}", e))?;
    
    let task_id = result.task_id.clone();
    
    let mut results_guard = results.lock().map_err(|e| format!("Lock error: {}", e))?;
    results_guard.entry(task_id.clone()).or_insert_with(Vec::new).push(result.clone());
    
    if let Some(task_results) = results_guard.get_mut(&task_id) {
        if task_results.len() > 1000 {
            task_results.drain(0..task_results.len() - 1000);
        }
    }
    
    let mut stats_guard = stats.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(stat) = stats_guard.get_mut(&task_id) {
        stat.total_frames_processed += 1;
        stat.total_detections += result.detections.len() as u64;
        
        if stat.total_detections > 0 {
            let total_confidence: f32 = result.detections.iter().map(|d| d.confidence).sum();
            stat.average_confidence = (stat.average_confidence * (stat.total_detections - result.detections.len() as u64) as f32 + total_confidence) / stat.total_detections as f32;
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_get_detection_results(
    params: &serde_json::Value,
    results: &Arc<Mutex<HashMap<String, Vec<DetectionResult>>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64());
    
    let guard = results.lock().map_err(|e| format!("Lock error: {}", e))?;
    let task_results = guard.get(task_id).cloned().unwrap_or_default();
    
    let items = if let Some(limit) = limit {
        task_results.into_iter().rev().take(limit as usize).collect::<Vec<_>>()
    } else {
        task_results.into_iter().rev().collect()
    };

    Ok(json!({
        "results": items
    }))
}

async fn handle_get_task(
    params: &serde_json::Value,
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    
    let guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    let task = guard.get(task_id).ok_or("Task not found")?;

    Ok(json!(task))
}

async fn handle_list_tasks(
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
) -> Result<serde_json::Value, String> {
    let guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    let task_list: Vec<AnalysisTask> = guard.values().cloned().collect();

    Ok(json!({
        "tasks": task_list
    }))
}

async fn handle_get_stats(
    params: &serde_json::Value,
    stats: &Arc<Mutex<HashMap<String, AnalysisStats>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    
    let guard = stats.lock().map_err(|e| format!("Lock error: {}", e))?;
    let stat = guard.get(task_id).ok_or("Stats not found")?;

    Ok(json!(stat))
}

async fn handle_enable_task(
    params: &serde_json::Value,
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    
    let mut guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(task) = guard.get_mut(task_id) {
        task.enabled = true;
    }

    Ok(json!({
        "enabled": true
    }))
}

async fn handle_disable_task(
    params: &serde_json::Value,
    tasks: &Arc<Mutex<HashMap<String, AnalysisTask>>>,
) -> Result<serde_json::Value, String> {
    let task_id = params.get("task_id").and_then(|t| t.as_str()).ok_or("Missing task_id")?;
    
    let mut guard = tasks.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(task) = guard.get_mut(task_id) {
        task.enabled = false;
    }

    Ok(json!({
        "disabled": true
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}
