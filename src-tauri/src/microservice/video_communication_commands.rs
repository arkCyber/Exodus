//! Tauri commands for Video Communication Service
//! 
//! These commands allow the frontend to interact with the Video Communication Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{VideoCommunicationService, VideoCommunicationServiceConfig, VideoCallConfig, VideoCallMetadata, VideoFrame, AudioFrame};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::time::Duration;
/// Managed Video Communication Service instance
pub struct ManagedVideoCommunicationService {
    service: Arc<VideoCommunicationService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedVideoCommunicationService {
    pub fn new(service: VideoCommunicationService) -> Self {
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

/// Send JSON-RPC request to Video Communication Service
async fn send_video_communication_request(
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
        .map_err(|e| format!("Failed to connect to Video Communication Service: {}", e))?;

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
        return Err(format!("Video Communication Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Video Communication Service
#[tauri::command]
pub async fn video_communication_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = VideoCommunicationService::new(config)
        .map_err(|e| format!("Failed to create Video Communication Service: {}", e))?;
    
    let managed = ManagedVideoCommunicationService::new(service);
    managed.start().await?;
    
    let _ = app.emit("video-communication-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Video Communication Service
#[tauri::command]
pub async fn video_communication_service_stop() -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let service = VideoCommunicationService::new(config)
        .map_err(|e| format!("Failed to create Video Communication Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Initiate a video call
#[tauri::command]
pub async fn video_call_initiate(config: VideoCallConfig) -> Result<String, String> {
    let service_config = VideoCommunicationServiceConfig::default();
    let params = json!(config);
    let result = send_video_communication_request(&service_config.socket_path, "initiate_call", params).await?;
    
    result.get("call_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid call_id response".to_string())
}

/// Accept a video call
#[tauri::command]
pub async fn video_call_accept(call_id: String) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    send_video_communication_request(&config.socket_path, "accept_call", params).await?;
    Ok(())
}

/// End a video call
#[tauri::command]
pub async fn video_call_end(call_id: String) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    send_video_communication_request(&config.socket_path, "end_call", params).await?;
    Ok(())
}

/// Get video call info
#[tauri::command]
pub async fn video_call_get(call_id: String) -> Result<VideoCallMetadata, String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    let result = send_video_communication_request(&config.socket_path, "get_call", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse call metadata: {}", e))
}

/// List all video calls
#[tauri::command]
pub async fn video_call_list() -> Result<Vec<VideoCallMetadata>, String> {
    let config = VideoCommunicationServiceConfig::default();
    let result = send_video_communication_request(&config.socket_path, "list_calls", json!(null)).await?;
    
    let calls = result.get("calls")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid calls response".to_string())?;
    
    calls.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Add video frame
#[tauri::command]
pub async fn video_call_add_frame(frame: VideoFrame) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!(frame);
    send_video_communication_request(&config.socket_path, "add_video_frame", params).await?;
    Ok(())
}

/// Get video frames
#[tauri::command]
pub async fn video_call_get_frames(call_id: String) -> Result<Vec<VideoFrame>, String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    let result = send_video_communication_request(&config.socket_path, "get_video_frames", params).await?;
    
    let frames = result.get("frames")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid frames response".to_string())?;
    
    frames.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Add audio frame
#[tauri::command]
pub async fn video_call_add_audio_frame(frame: AudioFrame) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!(frame);
    send_video_communication_request(&config.socket_path, "add_audio_frame", params).await?;
    Ok(())
}

/// Get audio frames
#[tauri::command]
pub async fn video_call_get_audio_frames(call_id: String) -> Result<Vec<AudioFrame>, String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    let result = send_video_communication_request(&config.socket_path, "get_audio_frames", params).await?;
    
    let frames = result.get("frames")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid frames response".to_string())?;
    
    frames.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Initiate video call using 12-digit number
#[tauri::command]
pub async fn video_call_initiate_by_digit(digit_id: String, caller_id: String, caller_name: String) -> Result<String, String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "digit_id": digit_id, "caller_id": caller_id, "caller_name": caller_name });
    let result = send_video_communication_request(&config.socket_path, "initiate_call_by_digit", params).await?;
    
    result.get("call_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid call_id response".to_string())
}

/// Get 12-digit number for a call
#[tauri::command]
pub async fn video_call_get_digit(call_id: String) -> Result<String, String> {
    let config = VideoCommunicationServiceConfig::default();
    let params = json!({ "call_id": call_id });
    let result = send_video_communication_request(&config.socket_path, "get_digit_for_call", params).await?;
    
    result.get("digit_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid digit_id response".to_string())
}

/// Video render loop - processes video frames and emits them to frontend
#[tauri::command]
pub async fn video_render_loop(
    call_id: String,
    app: AppHandle,
) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    
    // Start a background task to continuously poll for video frames
    tokio::spawn(async move {
        let mut last_timestamp = 0u64;
        
        loop {
            // Get video frames for the call
            let params = json!({ "call_id": call_id.clone(), "since_timestamp": last_timestamp });
            
            match send_video_communication_request(&config.socket_path, "get_video_frames", params).await {
                Ok(result) => {
                    if let Some(frames) = result.get("frames").and_then(|f| f.as_array()) {
                        for frame_value in frames {
                            if let Ok(frame) = serde_json::from_value::<crate::microservice::VideoFrame>(frame_value.clone()) {
                                // Emit frame to frontend for rendering
                                let _ = app.emit("video-frame", &frame);
                                last_timestamp = frame.timestamp;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error getting video frames: {}", e);
                    // Wait before retrying
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            }
            
            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await; // ~30 FPS
        }
    });
    
    Ok(())
}

/// Audio render loop - processes audio frames and emits them to frontend
#[tauri::command]
pub async fn audio_render_loop(
    call_id: String,
    app: AppHandle,
) -> Result<(), String> {
    let config = VideoCommunicationServiceConfig::default();
    
    // Start a background task to continuously poll for audio frames
    tokio::spawn(async move {
        let mut last_timestamp = 0u64;
        
        loop {
            // Get audio frames for the call
            let params = json!({ "call_id": call_id.clone(), "since_timestamp": last_timestamp });
            
            match send_video_communication_request(&config.socket_path, "get_audio_frames", params).await {
                Ok(result) => {
                    if let Some(frames) = result.get("frames").and_then(|f| f.as_array()) {
                        for frame_value in frames {
                            if let Ok(frame) = serde_json::from_value::<crate::microservice::AudioFrame>(frame_value.clone()) {
                                // Emit frame to frontend for audio playback
                                let _ = app.emit("audio-frame", &frame);
                                last_timestamp = frame.timestamp;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error getting audio frames: {}", e);
                    // Wait before retrying
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            }
            
            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // ~100 Hz for audio
        }
    });
    
    Ok(())
}
