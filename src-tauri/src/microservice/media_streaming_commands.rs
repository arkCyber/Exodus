//! Tauri commands for Media Streaming Service
//! 
//! These commands allow the frontend to interact with the Media Streaming Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{MediaStreamingService, MediaStreamingServiceConfig, StreamSession, StreamViewer, StreamQuality, AudioQuality, AudioEffects, AudioAnalysis, AudioMixerConfig};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Media Streaming Service instance
pub struct ManagedMediaStreamingService {
    service: Arc<MediaStreamingService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedMediaStreamingService {
    pub fn new(service: MediaStreamingService) -> Self {
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

/// Send JSON-RPC request to Media Streaming Service
async fn send_media_streaming_request(
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
        .map_err(|e| format!("Failed to connect to Media Streaming Service: {}", e))?;

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
        return Err(format!("Media Streaming Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Media Streaming Service
#[tauri::command]
pub async fn media_streaming_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = MediaStreamingService::new(config)
        .map_err(|e| format!("Failed to create Media Streaming Service: {}", e))?;
    
    let managed = ManagedMediaStreamingService::new(service);
    managed.start().await?;
    
    let _ = app.emit("media-streaming-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Media Streaming Service
#[tauri::command]
pub async fn media_streaming_service_stop() -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let service = MediaStreamingService::new(config)
        .map_err(|e| format!("Failed to create Media Streaming Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a new stream session
#[tauri::command]
pub async fn media_stream_create(session: StreamSession) -> Result<String, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!(session);
    let result = send_media_streaming_request(&config.socket_path, "create_session", params).await?;
    
    result.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid session_id response".to_string())
}

/// Update stream session
#[tauri::command]
pub async fn media_stream_update(session: StreamSession) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!(session);
    send_media_streaming_request(&config.socket_path, "update_session", params).await?;
    Ok(())
}

/// End stream session
#[tauri::command]
pub async fn media_stream_end(session_id: String) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "session_id": session_id });
    send_media_streaming_request(&config.socket_path, "end_session", params).await?;
    Ok(())
}

/// Get stream session
#[tauri::command]
pub async fn media_stream_get(session_id: String) -> Result<StreamSession, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "session_id": session_id });
    let result = send_media_streaming_request(&config.socket_path, "get_session", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse session: {}", e))
}

/// List all active streams
#[tauri::command]
pub async fn media_stream_list_active() -> Result<Vec<StreamSession>, String> {
    let config = MediaStreamingServiceConfig::default();
    let result = send_media_streaming_request(&config.socket_path, "list_active_streams", json!(null)).await?;
    
    let streams = result.get("streams")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid streams response".to_string())?;
    
    streams.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// List streams by user
#[tauri::command]
pub async fn media_stream_list_user(user_id: String) -> Result<Vec<StreamSession>, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_media_streaming_request(&config.socket_path, "list_user_streams", params).await?;
    
    let streams = result.get("streams")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid streams response".to_string())?;
    
    streams.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Join stream as viewer
#[tauri::command]
pub async fn media_stream_join(viewer: StreamViewer) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!(viewer);
    send_media_streaming_request(&config.socket_path, "join_stream", params).await?;
    Ok(())
}

/// Leave stream
#[tauri::command]
pub async fn media_stream_leave(session_id: String, viewer_id: String) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ 
        "session_id": session_id,
        "viewer_id": viewer_id
    });
    send_media_streaming_request(&config.socket_path, "leave_stream", params).await?;
    Ok(())
}

/// Get stream viewers
#[tauri::command]
pub async fn media_stream_get_viewers(session_id: String) -> Result<Vec<StreamViewer>, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "session_id": session_id });
    let result = send_media_streaming_request(&config.socket_path, "get_viewers", params).await?;
    
    let viewers = result.get("viewers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid viewers response".to_string())?;
    
    viewers.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get available qualities
#[tauri::command]
pub async fn media_stream_get_qualities() -> Result<Vec<StreamQuality>, String> {
    let config = MediaStreamingServiceConfig::default();
    let result = send_media_streaming_request(&config.socket_path, "get_qualities", json!(null)).await?;
    
    let qualities = result.get("qualities")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid qualities response".to_string())?;
    
    qualities.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Search streams
#[tauri::command]
pub async fn media_stream_search(query: String, limit: Option<usize>) -> Result<Vec<StreamSession>, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ 
        "query": query,
        "limit": limit
    });
    let result = send_media_streaming_request(&config.socket_path, "search_streams", params).await?;
    
    let streams = result.get("streams")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid streams response".to_string())?;
    
    streams.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get trending streams
#[tauri::command]
pub async fn media_stream_get_trending(limit: Option<usize>) -> Result<Vec<StreamSession>, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "limit": limit });
    let result = send_media_streaming_request(&config.socket_path, "get_trending_streams", params).await?;
    
    let streams = result.get("streams")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid streams response".to_string())?;
    
    streams.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

// Audio enhancement commands (Aster-style)

/// Get available audio qualities
#[tauri::command]
pub async fn media_stream_get_audio_qualities() -> Result<Vec<AudioQuality>, String> {
    let config = MediaStreamingServiceConfig::default();
    let result = send_media_streaming_request(&config.socket_path, "get_audio_qualities", json!(null)).await?;
    
    let audio_qualities = result.get("audio_qualities")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid audio_qualities response".to_string())?;
    
    audio_qualities.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Set audio effects for a session
#[tauri::command]
pub async fn media_stream_set_audio_effects(session_id: String, effects: AudioEffects) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let mut params = json!(effects);
    params["session_id"] = json!(session_id);
    send_media_streaming_request(&config.socket_path, "set_audio_effects", params).await?;
    Ok(())
}

/// Get audio effects for a session
#[tauri::command]
pub async fn media_stream_get_audio_effects(session_id: String) -> Result<AudioEffects, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "session_id": session_id });
    let result = send_media_streaming_request(&config.socket_path, "get_audio_effects", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse audio effects: {}", e))
}

/// Add audio analysis data
#[tauri::command]
pub async fn media_stream_add_audio_analysis(session_id: String, analysis: AudioAnalysis) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let mut params = json!(analysis);
    params["session_id"] = json!(session_id);
    send_media_streaming_request(&config.socket_path, "add_audio_analysis", params).await?;
    Ok(())
}

/// Get audio analysis history
#[tauri::command]
pub async fn media_stream_get_audio_analysis(session_id: String, limit: Option<usize>) -> Result<Vec<AudioAnalysis>, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "session_id": session_id, "limit": limit });
    let result = send_media_streaming_request(&config.socket_path, "get_audio_analysis", params).await?;
    
    let analysis = result.get("analysis")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid analysis response".to_string())?;
    
    analysis.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Create audio mixer
#[tauri::command]
pub async fn media_stream_create_audio_mixer(config: AudioMixerConfig) -> Result<(), String> {
    let service_config = MediaStreamingServiceConfig::default();
    let params = json!(config);
    send_media_streaming_request(&service_config.socket_path, "create_audio_mixer", params).await?;
    Ok(())
}

/// Get audio mixer config
#[tauri::command]
pub async fn media_stream_get_audio_mixer(mixer_id: String) -> Result<AudioMixerConfig, String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "mixer_id": mixer_id });
    let result = send_media_streaming_request(&config.socket_path, "get_audio_mixer", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse mixer config: {}", e))
}

/// Update audio mixer gain
#[tauri::command]
pub async fn media_stream_update_mixer_gain(mixer_id: String, channel: String, gain: f32) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "mixer_id": mixer_id, "channel": channel, "gain": gain });
    send_media_streaming_request(&config.socket_path, "update_mixer_gain", params).await?;
    Ok(())
}

/// Update audio mixer panning
#[tauri::command]
pub async fn media_stream_update_mixer_panning(mixer_id: String, channel: String, panning: f32) -> Result<(), String> {
    let config = MediaStreamingServiceConfig::default();
    let params = json!({ "mixer_id": mixer_id, "channel": channel, "panning": panning });
    send_media_streaming_request(&config.socket_path, "update_mixer_panning", params).await?;
    Ok(())
}
