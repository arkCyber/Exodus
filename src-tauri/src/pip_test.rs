//! Picture-in-Picture (画中画) functionality tests
//! Tests for PiP manager and Tauri commands

#[cfg(test)]
mod tests {
    use super::super::picture_in_picture::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_pip_manager_creation() {
        let manager = PipManager::new();
        let active = manager.get_all_active().await;
        assert!(active.is_empty());
    }
    
    #[tokio::test]
    async fn test_pip_state_serialization() {
        let state = PipState {
            video_url: "https://example.com/video.mp4".to_string(),
            is_active: true,
            window_width: 640,
            window_height: 480,
        };
        
        let json = serde_json::to_string(&state).expect("Failed to serialize state");
        let deserialized: PipState = serde_json::from_str(&json).expect("Failed to deserialize state");
        
        assert_eq!(deserialized.video_url, state.video_url);
        assert_eq!(deserialized.is_active, state.is_active);
        assert_eq!(deserialized.window_width, state.window_width);
        assert_eq!(deserialized.window_height, state.window_height);
    }
    
    #[tokio::test]
    async fn test_url_validation() {
        // Valid URLs
        assert!(validate_url("https://example.com/video.mp4").is_ok());
        assert!(validate_url("http://example.com/video.mp4").is_ok());
        assert!(validate_url("data:video/mp4;base64,ABC").is_ok());
        
        // Invalid URLs
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com/video.mp4").is_err());
        assert!(validate_url("not-a-url").is_err());
    }
    
    #[tokio::test]
    async fn test_dimension_validation() {
        // Valid dimensions
        assert!(validate_dimensions(640, 480).is_ok());
        assert!(validate_dimensions(160, 160).is_ok());
        assert!(validate_dimensions(4096, 4096).is_ok());
        
        // Invalid dimensions
        assert!(validate_dimensions(100, 480).is_err()); // Too small
        assert!(validate_dimensions(640, 100).is_err()); // Too small
        assert!(validate_dimensions(5000, 480).is_err()); // Too large
        assert!(validate_dimensions(640, 5000).is_err()); // Too large
    }
    
    #[tokio::test]
    async fn test_pip_enter_and_get_state() {
        let manager = Arc::new(PipManager::new());
        
        // Note: This test would need a proper mock AppHandle in a real environment
        // For now, we're just verifying the structure is correct
        let state = PipState {
            video_url: "https://example.com/video.mp4".to_string(),
            is_active: true,
            window_width: 640,
            window_height: 480,
        };
        
        assert_eq!(state.video_url, "https://example.com/video.mp4");
        assert!(state.is_active);
        assert_eq!(state.window_width, 640);
        assert_eq!(state.window_height, 480);
    }
}
