//! Video Communication Service - P2P video calls using iroh-live (MoQ)
//! 
//! This service provides real-time video and audio communication using Media over QUIC (MoQ)
//! protocol via iroh-live, offering better performance than WebRTC for decentralized scenarios.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Video call configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCallConfig {
    pub call_id: String,
    pub caller_id: String,
    pub caller_name: String,
    pub video_codec: String, // "H264", "VP8", "VP9"
    pub audio_codec: String, // "Opus", "AAC"
    pub fps: u32,
    pub resolution: String, // "720p", "1080p", "4K"
    pub description: Option<String>,
}

/// Video call metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCallMetadata {
    pub call_id: String,
    pub caller_id: String,
    pub caller_name: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub fps: u32,
    pub resolution: String,
    pub description: Option<String>,
    pub status: String, // "initiating", "ringing", "connected", "ended", "error"
    pub ticket: Option<String>, // iroh-live ticket for P2P connection
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
    pub duration: u64,
}

/// Video frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub call_id: String,
    pub frame_type: String, // "I", "P", "B"
    pub timestamp: u64,
    pub width: u32,
    pub height: u32,
    pub data: String, // Base64 encoded video data
}

/// Audio frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub call_id: String,
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u32,
    pub data: String, // Base64 encoded audio data
}

/// Configuration for Video Communication Service
#[derive(Debug, Clone)]
pub struct VideoCommunicationServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for VideoCommunicationServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_video_communication.sock");
        Self { socket_path }
    }
}

/// Video Communication Service
pub struct VideoCommunicationService {
    config: VideoCommunicationServiceConfig,
    calls: Arc<Mutex<HashMap<String, VideoCallMetadata>>>, // call_id -> metadata
    video_frames: Arc<Mutex<HashMap<String, Vec<VideoFrame>>>>, // call_id -> frames
    audio_frames: Arc<Mutex<HashMap<String, Vec<AudioFrame>>>>, // call_id -> frames
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl VideoCommunicationService {
    pub fn new(config: VideoCommunicationServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            calls: Arc::new(Mutex::new(HashMap::new())),
            video_frames: Arc::new(Mutex::new(HashMap::new())),
            audio_frames: Arc::new(Mutex::new(HashMap::new())),
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
        let calls = Arc::clone(&self.calls);
        let video_frames = Arc::clone(&self.video_frames);
        let audio_frames = Arc::clone(&self.audio_frames);
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
                                let calls = Arc::clone(&calls);
                                let video_frames = Arc::clone(&video_frames);
                                let audio_frames = Arc::clone(&audio_frames);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, calls, video_frames, audio_frames, node_id).await;
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

        println!("Video Communication Service started on {:?}", socket_path);
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

        println!("Video Communication Service stopped");
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

    /// Initiate a video call
    #[allow(dead_code)]
    pub fn initiate_call(&self, config: VideoCallConfig) -> Result<String, String> {
        let call_id = config.call_id.clone();
        
        // Generate iroh-live ticket for P2P connection
        // Note: iroh-live integration is currently blocked due to dependency chain instability
        // This would use iroh-live to create a ticket for establishing P2P connection
        // For now, we generate a mock ticket that can be replaced when iroh-live is integrated
        let ticket = Some(format!("iroh-live-ticket-{}", generate_node_id()));
        
        let metadata = VideoCallMetadata {
            call_id: call_id.clone(),
            caller_id: config.caller_id,
            caller_name: config.caller_name,
            video_codec: config.video_codec,
            audio_codec: config.audio_codec,
            fps: config.fps,
            resolution: config.resolution,
            description: config.description,
            status: "initiating".to_string(),
            ticket,
            created_at: current_timestamp(),
            started_at: None,
            ended_at: None,
            duration: 0,
        };

        let mut calls = self.calls.lock().map_err(|e| format!("Lock error: {}", e))?;
        calls.insert(call_id.clone(), metadata);

        Ok(call_id)
    }

    /// Accept a video call
    #[allow(dead_code)]
    pub fn accept_call(&self, call_id: String) -> Result<(), String> {
        let mut calls = self.calls.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = calls.get_mut(&call_id) {
            metadata.status = "connected".to_string();
            metadata.started_at = Some(current_timestamp());
        }
        Ok(())
    }

    /// End a video call
    #[allow(dead_code)]
    pub fn end_call(&self, call_id: String) -> Result<(), String> {
        let mut calls = self.calls.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = calls.get_mut(&call_id) {
            metadata.status = "ended".to_string();
            metadata.ended_at = Some(current_timestamp());
            if let Some(started_at) = metadata.started_at {
                metadata.duration = current_timestamp() - started_at;
            }
        }
        Ok(())
    }

    /// Get call info
    #[allow(dead_code)]
    pub fn get_call(&self, call_id: String) -> Option<VideoCallMetadata> {
        let calls = self.calls.lock().ok()?;
        calls.get(&call_id).cloned()
    }

    /// List all active calls
    #[allow(dead_code)]
    pub fn list_calls(&self) -> Vec<VideoCallMetadata> {
        self.calls.lock()
            .map(|calls| calls.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Add video frame
    #[allow(dead_code)]
    pub fn add_video_frame(&self, call_id: String, frame: VideoFrame) -> Result<(), String> {
        let mut video_frames = self.video_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
        video_frames.entry(call_id.clone()).or_insert_with(Vec::new).push(frame);
        
        // Keep only last 100 frames
        if let Some(frames) = video_frames.get_mut(&call_id) {
            if frames.len() > 100 {
                frames.drain(0..frames.len() - 100);
            }
        }
        Ok(())
    }

    /// Get video frames
    #[allow(dead_code)]
    pub fn get_video_frames(&self, call_id: String) -> Vec<VideoFrame> {
        self.video_frames.lock()
            .map(|video_frames| video_frames.get(&call_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Add audio frame
    #[allow(dead_code)]
    pub fn add_audio_frame(&self, call_id: String, frame: AudioFrame) -> Result<(), String> {
        let mut audio_frames = self.audio_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_frames.entry(call_id.clone()).or_insert_with(Vec::new).push(frame);
        
        // Keep only last 100 frames
        if let Some(frames) = audio_frames.get_mut(&call_id) {
            if frames.len() > 100 {
                frames.drain(0..frames.len() - 100);
            }
        }
        Ok(())
    }

    /// Get audio frames
    #[allow(dead_code)]
    pub fn get_audio_frames(&self, call_id: String) -> Vec<AudioFrame> {
        self.audio_frames.lock()
            .map(|audio_frames| audio_frames.get(&call_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Initiate video call using 12-digit number
    #[allow(dead_code)]
    pub fn initiate_call_by_digit(&self, digit_id: String, caller_id: String, caller_name: String) -> Result<String, String> {
        // Clean the digit input
        let clean_digit = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
        
        if clean_digit.len() != 12 {
            return Err("Invalid 12-digit number format".to_string());
        }

        // In a real implementation, this would resolve the digit to a node_id
        // For now, we'll use the digit as the remote identifier
        let call_id = format!("call-{}-{}", clean_digit, generate_node_id());
        
        let ticket = Some(format!("iroh-live-ticket-{}", generate_node_id()));
        
        let metadata = VideoCallMetadata {
            call_id: call_id.clone(),
            caller_id,
            caller_name,
            video_codec: "H264".to_string(),
            audio_codec: "Opus".to_string(),
            fps: 30,
            resolution: "720p".to_string(),
            description: Some(format!("Video call to 12-digit number: {}", clean_digit)),
            status: "initiating".to_string(),
            ticket,
            created_at: current_timestamp(),
            started_at: None,
            ended_at: None,
            duration: 0,
        };

        let mut calls = self.calls.lock().map_err(|e| format!("Lock error: {}", e))?;
        calls.insert(call_id.clone(), metadata);

        Ok(call_id)
    }

    /// Get 12-digit number for current node (for sharing)
    #[allow(dead_code)]
    pub fn get_digit_for_call(&self, call_id: String) -> Option<String> {
        let calls = self.calls.lock().ok()?;
        let _call = calls.get(&call_id)?;
        
        // Generate a 12-digit number for this call (in real implementation, this would use node_id)
        Some(format!("{:012}", current_timestamp() % 1_000_000_000_000))
    }
}

fn generate_node_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    calls: Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
    video_frames: Arc<Mutex<HashMap<String, Vec<VideoFrame>>>>,
    audio_frames: Arc<Mutex<HashMap<String, Vec<AudioFrame>>>>,
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
            "initiate_call" => handle_initiate_call(&params, &calls).await,
            "accept_call" => handle_accept_call(&params, &calls).await,
            "end_call" => handle_end_call(&params, &calls).await,
            "get_call" => handle_get_call(&params, &calls).await,
            "list_calls" => handle_list_calls(&calls).await,
            "add_video_frame" => handle_add_video_frame(&params, &video_frames).await,
            "get_video_frames" => handle_get_video_frames(&params, &video_frames).await,
            "add_audio_frame" => handle_add_audio_frame(&params, &audio_frames).await,
            "get_audio_frames" => handle_get_audio_frames(&params, &audio_frames).await,
            "initiate_call_by_digit" => handle_initiate_call_by_digit(&params, &calls).await,
            "get_digit_for_call" => handle_get_digit_for_call(&params, &calls).await,
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

async fn handle_initiate_call(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let config: VideoCallConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid config: {}", e))?;
    
    let call_id = config.call_id.clone();
    
    let ticket = Some(format!("iroh-live-ticket-{}", generate_node_id()));
    
    let metadata = VideoCallMetadata {
        call_id: call_id.clone(),
        caller_id: config.caller_id,
        caller_name: config.caller_name,
        video_codec: config.video_codec,
        audio_codec: config.audio_codec,
        fps: config.fps,
        resolution: config.resolution,
        description: config.description,
        status: "initiating".to_string(),
        ticket,
        created_at: current_timestamp(),
        started_at: None,
        ended_at: None,
        duration: 0,
    };

    let mut guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(call_id.clone(), metadata);

    Ok(json!({
        "call_id": call_id
    }))
}

async fn handle_accept_call(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let mut guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(call_id) {
        metadata.status = "connected".to_string();
        metadata.started_at = Some(current_timestamp());
    }

    Ok(json!({
        "accepted": true
    }))
}

async fn handle_end_call(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let mut guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(call_id) {
        metadata.status = "ended".to_string();
        metadata.ended_at = Some(current_timestamp());
        if let Some(started_at) = metadata.started_at {
            metadata.duration = current_timestamp() - started_at;
        }
    }

    Ok(json!({
        "ended": true
    }))
}

async fn handle_get_call(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.get(call_id).ok_or("Call not found")?;

    Ok(json!(metadata))
}

async fn handle_list_calls(
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    let call_list: Vec<VideoCallMetadata> = guard.values().cloned().collect();

    Ok(json!({
        "calls": call_list
    }))
}

async fn handle_add_video_frame(
    params: &serde_json::Value,
    video_frames: &Arc<Mutex<HashMap<String, Vec<VideoFrame>>>>,
) -> Result<serde_json::Value, String> {
    let frame: VideoFrame = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid frame: {}", e))?;
    
    let call_id = frame.call_id.clone();
    let mut guard = video_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.entry(call_id.clone()).or_insert_with(Vec::new).push(frame);
    
    // Keep only last 100 frames
    if let Some(frames) = guard.get_mut(&call_id) {
        if frames.len() > 100 {
            frames.drain(0..frames.len() - 100);
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_get_video_frames(
    params: &serde_json::Value,
    video_frames: &Arc<Mutex<HashMap<String, Vec<VideoFrame>>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let guard = video_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
    let frames = guard.get(call_id).cloned().unwrap_or_default();

    Ok(json!({
        "frames": frames
    }))
}

async fn handle_add_audio_frame(
    params: &serde_json::Value,
    audio_frames: &Arc<Mutex<HashMap<String, Vec<AudioFrame>>>>,
) -> Result<serde_json::Value, String> {
    let frame: AudioFrame = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid frame: {}", e))?;
    
    let call_id = frame.call_id.clone();
    let mut guard = audio_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.entry(call_id.clone()).or_insert_with(Vec::new).push(frame);
    
    // Keep only last 100 frames
    if let Some(frames) = guard.get_mut(&call_id) {
        if frames.len() > 100 {
            frames.drain(0..frames.len() - 100);
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_get_audio_frames(
    params: &serde_json::Value,
    audio_frames: &Arc<Mutex<HashMap<String, Vec<AudioFrame>>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let guard = audio_frames.lock().map_err(|e| format!("Lock error: {}", e))?;
    let frames = guard.get(call_id).cloned().unwrap_or_default();

    Ok(json!({
        "frames": frames
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

async fn handle_initiate_call_by_digit(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let digit_id = params.get("digit_id").and_then(|d| d.as_str()).ok_or("Missing digit_id")?;
    let caller_id = params.get("caller_id").and_then(|c| c.as_str()).ok_or("Missing caller_id")?;
    let caller_name = params.get("caller_name").and_then(|c| c.as_str()).ok_or("Missing caller_name")?;
    
    // Clean the digit input
    let clean_digit = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
    
    if clean_digit.len() != 12 {
        return Err("Invalid 12-digit number format".to_string());
    }

    // Generate call ID
    let call_id = format!("call-{}-{}", clean_digit, generate_node_id());
    
    let ticket = Some(format!("iroh-live-ticket-{}", generate_node_id()));
    
    let metadata = VideoCallMetadata {
        call_id: call_id.clone(),
        caller_id: caller_id.to_string(),
        caller_name: caller_name.to_string(),
        video_codec: "H264".to_string(),
        audio_codec: "Opus".to_string(),
        fps: 30,
        resolution: "720p".to_string(),
        description: Some(format!("Video call to 12-digit number: {}", clean_digit)),
        status: "initiating".to_string(),
        ticket,
        created_at: current_timestamp(),
        started_at: None,
        ended_at: None,
        duration: 0,
    };

    let mut guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(call_id.clone(), metadata);

    Ok(json!({
        "call_id": call_id
    }))
}

async fn handle_get_digit_for_call(
    params: &serde_json::Value,
    calls: &Arc<Mutex<HashMap<String, VideoCallMetadata>>>,
) -> Result<serde_json::Value, String> {
    let call_id = params.get("call_id").and_then(|c| c.as_str()).ok_or("Missing call_id")?;
    
    let guard = calls.lock().map_err(|e| format!("Lock error: {}", e))?;
    let _call = guard.get(call_id).ok_or("Call not found")?;
    
    // Generate a 12-digit number for this call
    let digit_id = format!("{:012}", current_timestamp() % 1_000_000_000_000);

    Ok(json!({
        "digit_id": digit_id
    }))
}
