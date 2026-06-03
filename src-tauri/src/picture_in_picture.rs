//! Picture-in-Picture (画中画) functionality for Exodus Browser
//! Supports HTML5 video elements with Picture-in-Picture API

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const MIN_WINDOW_SIZE: u32 = 160;
const MAX_WINDOW_SIZE: u32 = 4096;
const MAX_URL_LENGTH: usize = 2048;

/// Validate URL format
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }
    
    if url.len() > MAX_URL_LENGTH {
        return Err(format!("URL too long (max {} characters)", MAX_URL_LENGTH));
    }
    
    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("data:") {
        return Err("URL must start with http://, https://, or data:".to_string());
    }
    
    Ok(())
}

/// Validate window dimensions
pub fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width < MIN_WINDOW_SIZE || width > MAX_WINDOW_SIZE {
        return Err(format!("Width must be between {} and {}", MIN_WINDOW_SIZE, MAX_WINDOW_SIZE));
    }
    
    if height < MIN_WINDOW_SIZE || height > MAX_WINDOW_SIZE {
        return Err(format!("Height must be between {} and {}", MIN_WINDOW_SIZE, MAX_WINDOW_SIZE));
    }
    
    Ok(())
}

/// Picture-in-Picture state for a specific video
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipState {
    pub video_url: String,
    pub is_active: bool,
    pub window_width: u32,
    pub window_height: u32,
}

/// Picture-in-Picture manager
#[derive(Clone)]
pub struct PipManager {
    states: Arc<Mutex<Vec<PipState>>>,
}

impl PipManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Enter Picture-in-Picture mode for a video
    pub async fn enter_pip(&self, video_url: String, width: u32, height: u32, app: AppHandle) -> Result<(), String> {
        // Validate inputs
        validate_url(&video_url)?;
        validate_dimensions(width, height)?;
        
        let mut states = self.states.lock().await;
        
        // Check if this video is already in PiP
        if let Some(existing) = states.iter().find(|s| s.video_url == video_url) {
            if existing.is_active {
                return Ok(()); // Already active
            }
        }
        
        // Create new PiP state
        let state = PipState {
            video_url: video_url.clone(),
            is_active: true,
            window_width: width,
            window_height: height,
        };
        
        // Remove old entry for this video if exists
        states.retain(|s| s.video_url != video_url);
        states.push(state.clone());
        
        // Emit event
        let _ = app.emit("exodus-pip-entered", &state);
        
        Ok(())
    }

    /// Exit Picture-in-Picture mode for a video
    pub async fn exit_pip(&self, video_url: String, app: AppHandle) -> Result<(), String> {
        let mut states = self.states.lock().await;
        
        if let Some(index) = states.iter().position(|s| s.video_url == video_url) {
            let mut state = states[index].clone();
            state.is_active = false;
            states[index] = state.clone();
            
            let _ = app.emit("exodus-pip-exited", &state);
        }
        
        Ok(())
    }

    /// Resize Picture-in-Picture window
    pub async fn resize_pip(&self, video_url: String, width: u32, height: u32, app: AppHandle) -> Result<(), String> {
        // Validate inputs
        validate_url(&video_url)?;
        validate_dimensions(width, height)?;
        
        let mut states = self.states.lock().await;
        
        if let Some(state) = states.iter_mut().find(|s| s.video_url == video_url) {
            state.window_width = width;
            state.window_height = height;
            
            let _ = app.emit("exodus-pip-resized", state.clone());
        } else {
            return Err("Picture-in-Picture window not found for this video".to_string());
        }
        
        Ok(())
    }

    /// Get Picture-in-Picture state for a specific video
    pub async fn get_pip_state(&self, video_url: String) -> Option<PipState> {
        let states = self.states.lock().await;
        states.iter().find(|s| s.video_url == video_url).cloned()
    }
    
    /// Get all active Picture-in-Picture states
    pub async fn get_all_active(&self) -> Vec<PipState> {
        let states = self.states.lock().await;
        states.iter().filter(|s| s.is_active).cloned().collect()
    }
}

impl Default for PipManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands for Picture-in-Picture

/// Enter Picture-in-Picture mode
#[tauri::command]
pub async fn pip_enter(
    video_url: String,
    width: u32,
    height: u32,
    pip_manager: tauri::State<'_, Arc<PipManager>>,
    app: AppHandle,
) -> Result<(), String> {
    pip_manager.enter_pip(video_url, width, height, app).await
}

/// Exit Picture-in-Picture mode
#[tauri::command]
pub async fn pip_exit(
    video_url: String,
    pip_manager: tauri::State<'_, Arc<PipManager>>,
    app: AppHandle,
) -> Result<(), String> {
    pip_manager.exit_pip(video_url, app).await
}

/// Resize Picture-in-Picture window
#[tauri::command]
pub async fn pip_resize(
    video_url: String,
    width: u32,
    height: u32,
    pip_manager: tauri::State<'_, Arc<PipManager>>,
    app: AppHandle,
) -> Result<(), String> {
    pip_manager.resize_pip(video_url, width, height, app).await
}

/// Get Picture-in-Picture state for a video
#[tauri::command]
pub async fn pip_get_state(
    video_url: String,
    pip_manager: tauri::State<'_, Arc<PipManager>>,
) -> Result<Option<PipState>, String> {
    Ok(pip_manager.get_pip_state(video_url).await)
}

/// Get all active Picture-in-Picture states
#[tauri::command]
pub async fn pip_get_all_active(
    pip_manager: tauri::State<'_, Arc<PipManager>>,
) -> Result<Vec<PipState>, String> {
    Ok(pip_manager.get_all_active().await)
}
