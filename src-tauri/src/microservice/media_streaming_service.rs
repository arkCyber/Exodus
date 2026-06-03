//! Real-time Media Streaming Service
//! 
//! This service provides real-time video and audio streaming capabilities
//! using P2P CDN and WebRTC for peer-to-peer communication.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Stream session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSession {
    pub session_id: String,
    pub streamer_id: String,
    pub streamer_name: String,
    pub stream_type: String, // "video", "audio", "screen"
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub is_live: bool,
    pub viewer_count: u32,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
}

/// Stream viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamViewer {
    pub viewer_id: String,
    pub viewer_name: String,
    pub session_id: String,
    pub joined_at: u64,
    pub is_active: bool,
}

/// Stream quality settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQuality {
    pub quality_id: String,
    pub label: String, // "720p", "1080p", "4K"
    pub resolution: String,
    pub bitrate: u32,
    pub fps: u32,
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StreamStats {
    pub session_id: String,
    pub duration: u64,
    pub total_viewers: u32,
    pub peak_viewers: u32,
    pub average_bitrate: f32,
    pub total_bytes: u64,
}

/// Audio quality settings (Aster-style enhancements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQuality {
    pub quality_id: String,
    pub label: String, // "low", "medium", "high", "lossless"
    pub codec: String, // "opus", "aac", "flac"
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channels: u32,
}

/// Audio effects (Aster-style enhancements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffects {
    pub noise_reduction: bool,
    pub echo_cancellation: bool,
    pub auto_gain: bool,
    pub equalizer_enabled: bool,
    pub equalizer_bands: Vec<f32>, // 10-band equalizer values
    pub reverb_enabled: bool,
    pub reverb_level: f32,
}

/// Audio analysis data (Aster-style enhancements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub session_id: String,
    pub timestamp: u64,
    pub volume_level: f32,
    pub frequency_peaks: Vec<f32>,
    pub signal_to_noise_ratio: f32,
    pub clipping_detected: bool,
}

/// Audio mixer configuration (Aaster-style enhancements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMixerConfig {
    pub mixer_id: String,
    pub input_channels: Vec<String>,
    pub output_channels: Vec<String>,
    pub gain_levels: HashMap<String, f32>,
    pub panning: HashMap<String, f32>,
}

/// Configuration for Media Streaming Service
#[derive(Debug, Clone)]
pub struct MediaStreamingServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for MediaStreamingServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_media_streaming.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_media_streams");
        
        Self { socket_path, storage_dir }
    }
}

/// Media Streaming Service
pub struct MediaStreamingService {
    config: MediaStreamingServiceConfig,
    sessions: Arc<Mutex<HashMap<String, StreamSession>>>, // session_id -> session
    viewers: Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>, // session_id -> viewers
    user_sessions: Arc<Mutex<HashMap<String, Vec<String>>>>, // user_id -> session_ids
    qualities: Arc<Mutex<Vec<StreamQuality>>>,
    audio_qualities: Arc<Mutex<Vec<AudioQuality>>>, // Audio quality presets
    audio_effects: Arc<Mutex<HashMap<String, AudioEffects>>>, // session_id -> effects
    audio_analysis: Arc<Mutex<HashMap<String, Vec<AudioAnalysis>>>>, // session_id -> analysis history
    audio_mixers: Arc<Mutex<HashMap<String, AudioMixerConfig>>>, // mixer_id -> config
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl MediaStreamingService {
    pub fn new(config: MediaStreamingServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        let default_qualities = vec![
            StreamQuality {
                quality_id: "360p".to_string(),
                label: "360p".to_string(),
                resolution: "640x360".to_string(),
                bitrate: 800,
                fps: 30,
            },
            StreamQuality {
                quality_id: "720p".to_string(),
                label: "720p".to_string(),
                resolution: "1280x720".to_string(),
                bitrate: 2500,
                fps: 30,
            },
            StreamQuality {
                quality_id: "1080p".to_string(),
                label: "1080p".to_string(),
                resolution: "1920x1080".to_string(),
                bitrate: 5000,
                fps: 30,
            },
            StreamQuality {
                quality_id: "4K".to_string(),
                label: "4K".to_string(),
                resolution: "3840x2160".to_string(),
                bitrate: 15000,
                fps: 30,
            },
        ];
        
        let default_audio_qualities = vec![
            AudioQuality {
                quality_id: "low".to_string(),
                label: "Low Quality".to_string(),
                codec: "opus".to_string(),
                bitrate: 64,
                sample_rate: 16000,
                channels: 1,
            },
            AudioQuality {
                quality_id: "medium".to_string(),
                label: "Medium Quality".to_string(),
                codec: "opus".to_string(),
                bitrate: 128,
                sample_rate: 44100,
                channels: 2,
            },
            AudioQuality {
                quality_id: "high".to_string(),
                label: "High Quality".to_string(),
                codec: "opus".to_string(),
                bitrate: 256,
                sample_rate: 48000,
                channels: 2,
            },
            AudioQuality {
                quality_id: "lossless".to_string(),
                label: "Lossless".to_string(),
                codec: "flac".to_string(),
                bitrate: 1411,
                sample_rate: 48000,
                channels: 2,
            },
        ];
        
        Ok(Self {
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            viewers: Arc::new(Mutex::new(HashMap::new())),
            user_sessions: Arc::new(Mutex::new(HashMap::new())),
            qualities: Arc::new(Mutex::new(default_qualities)),
            audio_qualities: Arc::new(Mutex::new(default_audio_qualities)),
            audio_effects: Arc::new(Mutex::new(HashMap::new())),
            audio_analysis: Arc::new(Mutex::new(HashMap::new())),
            audio_mixers: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let socket_path = self.config.socket_path.clone();
        let sessions = Arc::clone(&self.sessions);
        let viewers = Arc::clone(&self.viewers);
        let user_sessions = Arc::clone(&self.user_sessions);
        let qualities = Arc::clone(&self.qualities);
        let audio_qualities = Arc::clone(&self.audio_qualities);
        let audio_effects = Arc::clone(&self.audio_effects);
        let audio_analysis = Arc::clone(&self.audio_analysis);
        let audio_mixers = Arc::clone(&self.audio_mixers);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let sessions = Arc::clone(&sessions);
                                let viewers = Arc::clone(&viewers);
                                let user_sessions = Arc::clone(&user_sessions);
                                let qualities = Arc::clone(&qualities);
                                let audio_qualities = Arc::clone(&audio_qualities);
                                let audio_effects = Arc::clone(&audio_effects);
                                let audio_analysis = Arc::clone(&audio_analysis);
                                let audio_mixers = Arc::clone(&audio_mixers);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, sessions, viewers, user_sessions, qualities, audio_qualities, audio_effects, audio_analysis, audio_mixers, node_id).await;
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

        println!("Media Streaming Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("Media Streaming Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Create a new stream session
    #[allow(dead_code)]
    pub fn create_session(&self, session: StreamSession) -> Result<(), String> {
        let session_id = session.session_id.clone();
        let streamer_id = session.streamer_id.clone();
        
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        sessions.insert(session_id.clone(), session.clone());
        drop(sessions);

        let mut user_sessions = self.user_sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        user_sessions.entry(streamer_id).or_insert_with(Vec::new).push(session_id.clone());

        Ok(())
    }

    /// Update stream session
    #[allow(dead_code)]
    pub fn update_session(&self, session: StreamSession) -> Result<(), String> {
        let session_id = session.session_id.clone();
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        sessions.insert(session_id, session);
        Ok(())
    }

    /// End stream session
    #[allow(dead_code)]
    pub fn end_session(&self, session_id: String) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions.get_mut(&session_id);
        if let Some(s) = session {
            s.is_live = false;
            s.ended_at = Some(current_timestamp());
        }
        drop(sessions);

        let mut viewers_guard = self.viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
        viewers_guard.remove(&session_id);

        Ok(())
    }

    /// Get stream session
    #[allow(dead_code)]
    pub fn get_session(&self, session_id: String) -> Option<StreamSession> {
        let sessions = self.sessions.lock().ok()?;
        sessions.get(&session_id).cloned()
    }

    /// List all active streams
    #[allow(dead_code)]
    pub fn list_active_streams(&self) -> Vec<StreamSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values()
            .filter(|s| s.is_live)
            .cloned()
            .collect()
    }

    /// List streams by user
    #[allow(dead_code)]
    pub fn list_user_streams(&self, user_id: String) -> Vec<StreamSession> {
        let user_sessions = self.user_sessions.lock().unwrap();
        let sessions = self.sessions.lock().unwrap();
        
        if let Some(session_ids) = user_sessions.get(&user_id) {
            session_ids.iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Join stream as viewer
    #[allow(dead_code)]
    pub fn join_stream(&self, viewer: StreamViewer) -> Result<(), String> {
        let session_id = viewer.session_id.clone();
        let _viewer_id = viewer.viewer_id.clone();
        
        let mut viewers = self.viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
        viewers.entry(session_id.clone()).or_insert_with(Vec::new).push(viewer.clone());
        drop(viewers);

        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.viewer_count += 1;
        }

        Ok(())
    }

    /// Leave stream
    #[allow(dead_code)]
    pub fn leave_stream(&self, session_id: String, viewer_id: String) -> Result<(), String> {
        let mut viewers = self.viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(viewer_list) = viewers.get_mut(&session_id) {
            viewer_list.retain(|v| v.viewer_id != viewer_id);
        }
        drop(viewers);

        let mut sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(session) = sessions.get_mut(&session_id) {
            if session.viewer_count > 0 {
                session.viewer_count -= 1;
            }
        }

        Ok(())
    }

    /// Get stream viewers
    #[allow(dead_code)]
    pub fn get_viewers(&self, session_id: String) -> Vec<StreamViewer> {
        let viewers = self.viewers.lock().unwrap();
        viewers.get(&session_id).cloned().unwrap_or_default()
    }

    /// Get available qualities
    #[allow(dead_code)]
    pub fn get_qualities(&self) -> Vec<StreamQuality> {
        let qualities = self.qualities.lock().unwrap();
        qualities.clone()
    }

    /// Search streams
    #[allow(dead_code)]
    pub fn search_streams(&self, query: String, limit: Option<usize>) -> Vec<StreamSession> {
        let sessions = self.sessions.lock().unwrap();
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<StreamSession> = sessions.values()
            .filter(|s| {
                s.is_live &&
                (s.title.to_lowercase().contains(&query_lower) ||
                 s.description.to_lowercase().contains(&query_lower) ||
                 s.streamer_name.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        
        results
    }

    /// Get trending streams
    #[allow(dead_code)]
    pub fn get_trending(&self, limit: Option<usize>) -> Vec<StreamSession> {
        let sessions = self.sessions.lock().unwrap();
        
        let mut trending: Vec<StreamSession> = sessions.values()
            .filter(|s| s.is_live && s.viewer_count > 0)
            .cloned()
            .collect();

        trending.sort_by(|a, b| b.viewer_count.cmp(&a.viewer_count));
        
        if let Some(limit) = limit {
            trending.truncate(limit);
        }

        trending
    }

    /// Get available audio qualities (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn get_audio_qualities(&self) -> Vec<AudioQuality> {
        let audio_qualities = self.audio_qualities.lock().unwrap();
        audio_qualities.clone()
    }

    /// Set audio effects for a session (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn set_audio_effects(&self, session_id: String, effects: AudioEffects) -> Result<(), String> {
        let mut audio_effects = self.audio_effects.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_effects.insert(session_id, effects);
        Ok(())
    }

    /// Get audio effects for a session (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn get_audio_effects(&self, session_id: String) -> Option<AudioEffects> {
        let audio_effects = self.audio_effects.lock().ok()?;
        audio_effects.get(&session_id).cloned()
    }

    /// Add audio analysis data (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn add_audio_analysis(&self, session_id: String, analysis: AudioAnalysis) -> Result<(), String> {
        let mut audio_analysis = self.audio_analysis.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_analysis.entry(session_id.clone()).or_insert_with(Vec::new).push(analysis);
        
        // Keep only last 100 analysis entries
        if let Some(analysis_history) = audio_analysis.get_mut(&session_id) {
            if analysis_history.len() > 100 {
                analysis_history.drain(0..analysis_history.len() - 100);
            }
        }
        
        Ok(())
    }

    /// Get audio analysis history (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn get_audio_analysis(&self, session_id: String, limit: Option<usize>) -> Vec<AudioAnalysis> {
        let audio_analysis = self.audio_analysis.lock().unwrap();
        let analysis_history = audio_analysis.get(&session_id).cloned().unwrap_or_default();
        
        if let Some(limit) = limit {
            analysis_history.into_iter().rev().take(limit).collect()
        } else {
            analysis_history.into_iter().rev().collect()
        }
    }

    /// Create audio mixer (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn create_audio_mixer(&self, config: AudioMixerConfig) -> Result<(), String> {
        let mixer_id = config.mixer_id.clone();
        let mut audio_mixers = self.audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_mixers.insert(mixer_id, config);
        Ok(())
    }

    /// Get audio mixer config (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn get_audio_mixer(&self, mixer_id: String) -> Option<AudioMixerConfig> {
        let audio_mixers = self.audio_mixers.lock().ok()?;
        audio_mixers.get(&mixer_id).cloned()
    }

    /// Update audio mixer gain (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn update_mixer_gain(&self, mixer_id: String, channel: String, gain: f32) -> Result<(), String> {
        let mut audio_mixers = self.audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(mixer) = audio_mixers.get_mut(&mixer_id) {
            mixer.gain_levels.insert(channel, gain);
        }
        Ok(())
    }

    /// Update audio mixer panning (Aster-style enhancement)
    #[allow(dead_code)]
    pub fn update_mixer_panning(&self, mixer_id: String, channel: String, panning: f32) -> Result<(), String> {
        let mut audio_mixers = self.audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(mixer) = audio_mixers.get_mut(&mixer_id) {
            mixer.panning.insert(channel, panning);
        }
        Ok(())
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    sessions: Arc<Mutex<HashMap<String, StreamSession>>>,
    viewers: Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>,
    user_sessions: Arc<Mutex<HashMap<String, Vec<String>>>>,
    qualities: Arc<Mutex<Vec<StreamQuality>>>,
    audio_qualities: Arc<Mutex<Vec<AudioQuality>>>,
    audio_effects: Arc<Mutex<HashMap<String, AudioEffects>>>,
    audio_analysis: Arc<Mutex<HashMap<String, Vec<AudioAnalysis>>>>,
    audio_mixers: Arc<Mutex<HashMap<String, AudioMixerConfig>>>,
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
            "create_session" => handle_create_session(&params, &sessions, &user_sessions).await,
            "update_session" => handle_update_session(&params, &sessions).await,
            "end_session" => handle_end_session(&params, &sessions, &viewers).await,
            "get_session" => handle_get_session(&params, &sessions).await,
            "list_active_streams" => handle_list_active_streams(&sessions).await,
            "list_user_streams" => handle_list_user_streams(&params, &user_sessions, &sessions).await,
            "join_stream" => handle_join_stream(&params, &viewers, &sessions).await,
            "leave_stream" => handle_leave_stream(&params, &viewers, &sessions).await,
            "get_viewers" => handle_get_viewers(&params, &viewers).await,
            "get_qualities" => handle_get_qualities(&qualities).await,
            "search_streams" => handle_search_streams(&params, &sessions).await,
            "get_trending_streams" => handle_get_trending_streams(&params, &sessions).await,
            "get_audio_qualities" => handle_get_audio_qualities(&audio_qualities).await,
            "set_audio_effects" => handle_set_audio_effects(&params, &audio_effects).await,
            "get_audio_effects" => handle_get_audio_effects(&params, &audio_effects).await,
            "add_audio_analysis" => handle_add_audio_analysis(&params, &audio_analysis).await,
            "get_audio_analysis" => handle_get_audio_analysis(&params, &audio_analysis).await,
            "create_audio_mixer" => handle_create_audio_mixer(&params, &audio_mixers).await,
            "get_audio_mixer" => handle_get_audio_mixer(&params, &audio_mixers).await,
            "update_mixer_gain" => handle_update_mixer_gain(&params, &audio_mixers).await,
            "update_mixer_panning" => handle_update_mixer_panning(&params, &audio_mixers).await,
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

async fn handle_create_session(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
    user_sessions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let session: StreamSession = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid session: {}", e))?;
    
    let session_id = session.session_id.clone();
    let streamer_id = session.streamer_id.clone();
    
    let mut sessions_guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    sessions_guard.insert(session_id.clone(), session.clone());
    drop(sessions_guard);

    let mut user_guard = user_sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    user_guard.entry(streamer_id).or_insert_with(Vec::new).push(session_id.clone());

    Ok(json!({
        "session_id": session_id
    }))
}

async fn handle_update_session(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let session: StreamSession = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid session: {}", e))?;
    
    let session_id = session.session_id.clone();
    let mut guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(session_id, session);

    Ok(json!({
        "updated": true
    }))
}

async fn handle_end_session(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
    viewers: &Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    
    let mut sessions_guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let session = sessions_guard.get_mut(session_id);
    if let Some(s) = session {
        s.is_live = false;
        s.ended_at = Some(current_timestamp());
    }
    drop(sessions_guard);

    let mut viewers_guard = viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
    viewers_guard.remove(session_id);

    Ok(json!({
        "ended": true
    }))
}

async fn handle_get_session(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    
    let guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(session_id)
        .map(|s| json!(s))
        .ok_or_else(|| "Session not found".to_string())
}

async fn handle_list_active_streams(
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let active: Vec<StreamSession> = guard.values()
        .filter(|s| s.is_live)
        .cloned()
        .collect();

    Ok(json!({
        "streams": active
    }))
}

async fn handle_list_user_streams(
    params: &serde_json::Value,
    user_sessions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let user_guard = user_sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let sessions_guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(session_ids) = user_guard.get(user_id) {
        let stream_list: Vec<StreamSession> = session_ids.iter()
            .filter_map(|id| sessions_guard.get(id).cloned())
            .collect();
        return Ok(json!({ "streams": stream_list }));
    }

    Ok(json!({
        "streams": Vec::<StreamSession>::new()
    }))
}

async fn handle_join_stream(
    params: &serde_json::Value,
    viewers: &Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let viewer: StreamViewer = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid viewer: {}", e))?;
    
    let session_id = viewer.session_id.clone();
    
    let mut viewers_guard = viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
    viewers_guard.entry(session_id.clone()).or_insert_with(Vec::new).push(viewer.clone());
    drop(viewers_guard);

    let mut sessions_guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(session) = sessions_guard.get_mut(&session_id) {
        session.viewer_count += 1;
    }

    Ok(json!({
        "joined": true
    }))
}

async fn handle_leave_stream(
    params: &serde_json::Value,
    viewers: &Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    let viewer_id = params.get("viewer_id").and_then(|v| v.as_str()).ok_or("Missing viewer_id")?;
    
    let mut viewers_guard = viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(viewer_list) = viewers_guard.get_mut(session_id) {
        viewer_list.retain(|v| v.viewer_id != viewer_id);
    }
    drop(viewers_guard);

    let mut sessions_guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(session) = sessions_guard.get_mut(session_id) {
        if session.viewer_count > 0 {
            session.viewer_count -= 1;
        }
    }

    Ok(json!({
        "left": true
    }))
}

async fn handle_get_viewers(
    params: &serde_json::Value,
    viewers: &Arc<Mutex<HashMap<String, Vec<StreamViewer>>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    
    let guard = viewers.lock().map_err(|e| format!("Lock error: {}", e))?;
    let viewer_list = guard.get(session_id).cloned().unwrap_or_default();

    Ok(json!({
        "viewers": viewer_list
    }))
}

async fn handle_get_qualities(
    qualities: &Arc<Mutex<Vec<StreamQuality>>>,
) -> Result<serde_json::Value, String> {
    let guard = qualities.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(json!({
        "qualities": guard.clone()
    }))
}

async fn handle_search_streams(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let query = params.get("query").and_then(|q| q.as_str()).ok_or("Missing query")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<StreamSession> = guard.values()
        .filter(|s| {
            s.is_live &&
            (s.title.to_lowercase().contains(&query_lower) ||
             s.description.to_lowercase().contains(&query_lower) ||
             s.streamer_name.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.viewer_count.cmp(&a.viewer_count));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "streams": results
    }))
}

async fn handle_get_trending_streams(
    params: &serde_json::Value,
    sessions: &Arc<Mutex<HashMap<String, StreamSession>>>,
) -> Result<serde_json::Value, String> {
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut trending: Vec<StreamSession> = guard.values()
        .filter(|s| s.is_live && s.viewer_count > 0)
        .cloned()
        .collect();

    trending.sort_by(|a, b| b.viewer_count.cmp(&a.viewer_count));
    
    if let Some(limit) = limit {
        trending.truncate(limit);
    }

    Ok(json!({
        "streams": trending
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

// Audio enhancement handlers (Aster-style)
async fn handle_get_audio_qualities(
    audio_qualities: &Arc<Mutex<Vec<AudioQuality>>>,
) -> Result<serde_json::Value, String> {
    let guard = audio_qualities.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(json!({
        "audio_qualities": guard.clone()
    }))
}

async fn handle_set_audio_effects(
    params: &serde_json::Value,
    audio_effects: &Arc<Mutex<HashMap<String, AudioEffects>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    let effects: AudioEffects = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid effects: {}", e))?;
    
    let mut guard = audio_effects.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(session_id.to_string(), effects);

    Ok(json!({
        "set": true
    }))
}

async fn handle_get_audio_effects(
    params: &serde_json::Value,
    audio_effects: &Arc<Mutex<HashMap<String, AudioEffects>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    
    let guard = audio_effects.lock().map_err(|e| format!("Lock error: {}", e))?;
    let effects = guard.get(session_id).ok_or("Audio effects not found")?;

    Ok(json!(effects))
}

async fn handle_add_audio_analysis(
    params: &serde_json::Value,
    audio_analysis: &Arc<Mutex<HashMap<String, Vec<AudioAnalysis>>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    let analysis: AudioAnalysis = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid analysis: {}", e))?;
    
    let mut guard = audio_analysis.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.entry(session_id.to_string()).or_insert_with(Vec::new).push(analysis);
    
    if let Some(analysis_history) = guard.get_mut(session_id) {
        if analysis_history.len() > 100 {
            analysis_history.drain(0..analysis_history.len() - 100);
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_get_audio_analysis(
    params: &serde_json::Value,
    audio_analysis: &Arc<Mutex<HashMap<String, Vec<AudioAnalysis>>>>,
) -> Result<serde_json::Value, String> {
    let session_id = params.get("session_id").and_then(|s| s.as_str()).ok_or("Missing session_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64());
    
    let guard = audio_analysis.lock().map_err(|e| format!("Lock error: {}", e))?;
    let analysis_history = guard.get(session_id).cloned().unwrap_or_default();
    
    let items = if let Some(limit) = limit {
        analysis_history.into_iter().rev().take(limit as usize).collect::<Vec<_>>()
    } else {
        analysis_history.into_iter().rev().collect()
    };

    Ok(json!({
        "analysis": items
    }))
}

async fn handle_create_audio_mixer(
    params: &serde_json::Value,
    audio_mixers: &Arc<Mutex<HashMap<String, AudioMixerConfig>>>,
) -> Result<serde_json::Value, String> {
    let config: AudioMixerConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid mixer config: {}", e))?;
    
    let mixer_id = config.mixer_id.clone();
    let mut guard = audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(mixer_id, config);

    Ok(json!({
        "created": true
    }))
}

async fn handle_get_audio_mixer(
    params: &serde_json::Value,
    audio_mixers: &Arc<Mutex<HashMap<String, AudioMixerConfig>>>,
) -> Result<serde_json::Value, String> {
    let mixer_id = params.get("mixer_id").and_then(|m| m.as_str()).ok_or("Missing mixer_id")?;
    
    let guard = audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mixer = guard.get(mixer_id).ok_or("Audio mixer not found")?;

    Ok(json!(mixer))
}

async fn handle_update_mixer_gain(
    params: &serde_json::Value,
    audio_mixers: &Arc<Mutex<HashMap<String, AudioMixerConfig>>>,
) -> Result<serde_json::Value, String> {
    let mixer_id = params.get("mixer_id").and_then(|m| m.as_str()).ok_or("Missing mixer_id")?;
    let channel = params.get("channel").and_then(|c| c.as_str()).ok_or("Missing channel")?;
    let gain = params.get("gain").and_then(|g| g.as_f64()).ok_or("Missing gain")? as f32;
    
    let mut guard = audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(mixer) = guard.get_mut(mixer_id) {
        mixer.gain_levels.insert(channel.to_string(), gain);
    }

    Ok(json!({
        "updated": true
    }))
}

async fn handle_update_mixer_panning(
    params: &serde_json::Value,
    audio_mixers: &Arc<Mutex<HashMap<String, AudioMixerConfig>>>,
) -> Result<serde_json::Value, String> {
    let mixer_id = params.get("mixer_id").and_then(|m| m.as_str()).ok_or("Missing mixer_id")?;
    let channel = params.get("channel").and_then(|c| c.as_str()).ok_or("Missing channel")?;
    let panning = params.get("panning").and_then(|p| p.as_f64()).ok_or("Missing panning")? as f32;
    
    let mut guard = audio_mixers.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(mixer) = guard.get_mut(mixer_id) {
        mixer.panning.insert(channel.to_string(), panning);
    }

    Ok(json!({
        "updated": true
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("media_streaming_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_session() {
        let config = MediaStreamingServiceConfig::default();
        let service = MediaStreamingService::new(config).unwrap();
        
        let session = StreamSession {
            session_id: "stream-1".to_string(),
            streamer_id: "user-1".to_string(),
            streamer_name: "User 1".to_string(),
            stream_type: "video".to_string(),
            title: "Test Stream".to_string(),
            description: "A test stream".to_string(),
            thumbnail_url: None,
            is_live: true,
            viewer_count: 0,
            created_at: current_timestamp(),
            started_at: Some(current_timestamp()),
            ended_at: None,
        };

        service.create_session(session).unwrap();
        let retrieved = service.get_session("stream-1".to_string()).unwrap();
        assert_eq!(retrieved.title, "Test Stream");
    }

    #[test]
    fn test_join_and_leave_stream() {
        let config = MediaStreamingServiceConfig::default();
        let service = MediaStreamingService::new(config).unwrap();
        
        let session = StreamSession {
            session_id: "stream-2".to_string(),
            streamer_id: "user-1".to_string(),
            streamer_name: "User 1".to_string(),
            stream_type: "video".to_string(),
            title: "Test Stream".to_string(),
            description: "A test stream".to_string(),
            thumbnail_url: None,
            is_live: true,
            viewer_count: 0,
            created_at: current_timestamp(),
            started_at: Some(current_timestamp()),
            ended_at: None,
        };

        service.create_session(session).unwrap();
        
        let viewer = StreamViewer {
            viewer_id: "viewer-1".to_string(),
            viewer_name: "Viewer 1".to_string(),
            session_id: "stream-2".to_string(),
            joined_at: current_timestamp(),
            is_active: true,
        };

        service.join_stream(viewer).unwrap();
        let session = service.get_session("stream-2".to_string()).unwrap();
        assert_eq!(session.viewer_count, 1);

        service.leave_stream("stream-2".to_string(), "viewer-1".to_string()).unwrap();
        let session = service.get_session("stream-2".to_string()).unwrap();
        assert_eq!(session.viewer_count, 0);
    }
}
