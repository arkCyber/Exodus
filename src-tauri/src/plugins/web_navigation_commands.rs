//! Exodus Browser — Chrome Web Navigation API Tauri commands
//!
//! Provides Tauri commands for chrome.webNavigation API

use tauri::{AppHandle, Manager, State};

use super::web_navigation::{
    fire_navigation_event, matches_filter, FrameDetails, NavigationDetails,
    TransitionQualifier, TransitionType, WebNavigationFilter, WebNavigationListener,
    WebNavigationRegistry,
};
use super::manager::ExtensionState;
use super::tabs::TabRegistry;

/// Register a web navigation event listener
#[tauri::command]
pub fn extension_web_navigation_add_listener(
    extension_id: String,
    event_name: String,
    filter: Option<WebNavigationFilter>,
    state: State<'_, WebNavigationRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    let listener = WebNavigationListener {
        extension_id: extension_id.clone(),
        event_name: event_name.clone(),
        filter,
    };
    registry.add_listener(listener)
}

/// Remove a web navigation event listener
#[tauri::command]
pub fn extension_web_navigation_remove_listener(
    extension_id: String,
    event_name: String,
    state: State<'_, WebNavigationRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.remove_listener(&extension_id, &event_name)
}

/// Remove all web navigation listeners for an extension
#[tauri::command]
pub fn extension_web_navigation_remove_all_listeners(
    extension_id: String,
    state: State<'_, WebNavigationRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.remove_all_listeners(&extension_id)
}

/// Get all frames for a tab
#[tauri::command]
pub fn extension_web_navigation_get_all_frames(
    tab_id: i32,
    ext_state: State<'_, ExtensionState>,
    tab_registry: State<'_, TabRegistry>,
) -> Result<Vec<FrameDetails>, String> {
    // Get tab information from tab registry
    let tab = tab_registry.find_by_chrome_id(tab_id as i64);
    
    if let Some(tab_info) = tab {
        // Return the main frame for the tab
        Ok(vec![FrameDetails {
            tab_id,
            frame_id: 0,
            parent_frame_id: -1,
            url: Some(tab_info.url),
            error_occurred: None,
        }])
    } else {
        // Tab not found, return empty
        Ok(Vec::new())
    }
}

/// Get a specific frame
#[tauri::command]
pub fn extension_web_navigation_get_frame(
    tab_id: i32,
    frame_id: i32,
    ext_state: State<'_, ExtensionState>,
    tab_registry: State<'_, TabRegistry>,
) -> Result<Option<FrameDetails>, String> {
    // Get tab information from tab registry
    let tab = tab_registry.find_by_chrome_id(tab_id as i64);
    
    if let Some(tab_info) = tab {
        // For now, we only support the main frame (frame_id: 0)
        if frame_id == 0 {
            Ok(Some(FrameDetails {
                tab_id,
                frame_id: 0,
                parent_frame_id: -1,
                url: Some(tab_info.url),
                error_occurred: None,
            }))
        } else {
            // Subframes not yet supported
            Ok(None)
        }
    } else {
        // Tab not found
        Ok(None)
    }
}

/// Fire a navigation event (called by browser navigation system)
#[tauri::command]
pub fn fire_web_navigation_event(
    app: AppHandle,
    event_name: String,
    details: NavigationDetails,
    registry: State<'_, WebNavigationRegistry>,
) -> Result<(), String> {
    fire_navigation_event(&app, registry.inner(), &event_name, details);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_details_serialization() {
        let details = NavigationDetails {
            tab_id: 1,
            url: "https://example.com".to_string(),
            process_id: 0,
            frame_id: 0,
            parent_frame_id: 0,
            transition_type: Some("link".to_string()),
            transition_qualifiers: None,
            server_redirect: None,
            timestamp: 1234567890.0,
        };
        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("tabId"));
    }

    #[test]
    fn test_navigation_details_all_fields() {
        let details = NavigationDetails {
            tab_id: 2,
            url: "https://github.com".to_string(),
            process_id: 1,
            frame_id: 1,
            parent_frame_id: 0,
            transition_type: Some("typed".to_string()),
            transition_qualifiers: Some(vec!["client_redirect".to_string()]),
            server_redirect: Some("https://redirect.com".to_string()),
            timestamp: 1234567891.0,
        };
        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("GitHub"));
        assert!(json.contains("typed"));
        assert!(json.contains("client_redirect"));
    }

    #[test]
    fn test_frame_details_serialization() {
        let frame = FrameDetails {
            tab_id: 1,
            frame_id: 0,
            parent_frame_id: 0,
            url: Some("https://example.com".to_string()),
            error_occurred: None,
        };
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("frameId"));
        assert!(json.contains("url"));
    }

    #[test]
    fn test_frame_details_with_error() {
        let frame = FrameDetails {
            tab_id: 1,
            frame_id: 1,
            parent_frame_id: 0,
            url: Some("https://example.com".to_string()),
            error_occurred: Some(true),
        };
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("errorOccurred"));
    }
}
