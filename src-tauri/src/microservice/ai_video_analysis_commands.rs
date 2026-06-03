//! Tauri commands for AI Video Analysis Service
//! 
//! These commands allow the frontend to interact with the AI Video Analysis Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{AiVideoAnalysisService, AiVideoAnalysisServiceConfig, AnalysisTask, DetectionResult, AnalysisStats};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed AI Video Analysis Service instance
pub struct ManagedAiVideoAnalysisService {
    service: Arc<AiVideoAnalysisService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedAiVideoAnalysisService {
    pub fn new(service: AiVideoAnalysisService) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }
        
        self.service.start().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
}

/// Send JSON-RPC request to AI Video Analysis Service
async fn send_ai_video_analysis_request(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let socket_path_str = socket_path.to_string_lossy().to_string();
    let client = tokio::net::UnixStream::connect(&socket_path_str)
        .await
        .map_err(|e| format!("Failed to connect to AI Video Analysis Service: {}", e))?;

    let (mut reader, mut writer) = client.into_split();
    
    let request_str = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    
    writer.write_all(request_str.as_bytes()).await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut buf = [0u8; 8192];
    let n = reader.read(&mut buf).await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let response_str = String::from_utf8_lossy(&buf[..n]).to_string();
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(error) = response.get("error") {
        return Err(format!("AI Video Analysis Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the AI Video Analysis Service
#[tauri::command]
pub async fn ai_video_analysis_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = AiVideoAnalysisService::new(config)
        .map_err(|e| format!("Failed to create AI Video Analysis Service: {}", e))?;
    
    let managed = ManagedAiVideoAnalysisService::new(service);
    managed.start().await?;
    
    let _ = app.emit("ai-video-analysis-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the AI Video Analysis Service
#[tauri::command]
pub async fn ai_video_analysis_service_stop() -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let service = AiVideoAnalysisService::new(config)
        .map_err(|e| format!("Failed to create AI Video Analysis Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create an analysis task
#[tauri::command]
pub async fn ai_video_analysis_create_task(task: AnalysisTask) -> Result<String, String> {
    let service_config = AiVideoAnalysisServiceConfig::default();
    let params = json!(task);
    let result = send_ai_video_analysis_request(&service_config.socket_path, "create_task", params).await?;
    
    result.get("task_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid task_id response".to_string())
}

/// Delete an analysis task
#[tauri::command]
pub async fn ai_video_analysis_delete_task(task_id: String) -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id });
    send_ai_video_analysis_request(&config.socket_path, "delete_task", params).await?;
    Ok(())
}

/// Add detection result
#[tauri::command]
pub async fn ai_video_analysis_add_result(result: DetectionResult) -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!(result);
    send_ai_video_analysis_request(&config.socket_path, "add_detection_result", params).await?;
    Ok(())
}

/// Get detection results
#[tauri::command]
pub async fn ai_video_analysis_get_results(task_id: String, limit: Option<usize>) -> Result<Vec<DetectionResult>, String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id, "limit": limit });
    let result = send_ai_video_analysis_request(&config.socket_path, "get_detection_results", params).await?;
    
    let results = result.get("results")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid results response".to_string())?;
    
    results.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get task info
#[tauri::command]
pub async fn ai_video_analysis_get_task(task_id: String) -> Result<AnalysisTask, String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id });
    let result = send_ai_video_analysis_request(&config.socket_path, "get_task", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse task: {}", e))
}

/// List all tasks
#[tauri::command]
pub async fn ai_video_analysis_list_tasks() -> Result<Vec<AnalysisTask>, String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let result = send_ai_video_analysis_request(&config.socket_path, "list_tasks", json!(null)).await?;
    
    let tasks = result.get("tasks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid tasks response".to_string())?;
    
    tasks.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get analysis stats
#[tauri::command]
pub async fn ai_video_analysis_get_stats(task_id: String) -> Result<AnalysisStats, String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id });
    let result = send_ai_video_analysis_request(&config.socket_path, "get_stats", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse stats: {}", e))
}

/// Enable task
#[tauri::command]
pub async fn ai_video_analysis_enable_task(task_id: String) -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id });
    send_ai_video_analysis_request(&config.socket_path, "enable_task", params).await?;
    Ok(())
}

/// Disable task
#[tauri::command]
pub async fn ai_video_analysis_disable_task(task_id: String) -> Result<(), String> {
    let config = AiVideoAnalysisServiceConfig::default();
    let params = json!({ "task_id": task_id });
    send_ai_video_analysis_request(&config.socket_path, "disable_task", params).await?;
    Ok(())
}
