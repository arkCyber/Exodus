//! Exodus Browser — Chrome WebRequest API Tauri commands
//!
//! Provides Tauri commands for chrome.webRequest API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Request details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestDetails {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub tab_id: Option<i32>,
    pub parent_frame_id: Option<i32>,
    pub frame_id: Option<i32>,
    pub request_body: Option<RequestBody>,
    pub initiator: Option<String>,
    pub type_: String,
    pub timestamp: f64,
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub error: Option<String>,
    #[serde(rename = "formData")]
    pub form_data: Option<HashMap<String, Vec<String>>>,
    pub raw: Option<Vec<u8>>,
}

/// Response headers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseHeaders {
    pub response_headers: Option<HashMap<String, Vec<String>>>,
    pub status_line: Option<String>,
}

/// Blocking response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingResponse {
    pub cancel: Option<bool>,
    pub redirect_url: Option<String>,
    pub request_headers: Option<HashMap<String, Vec<String>>>,
    pub response_headers: Option<HashMap<String, Vec<String>>>,
    pub auth_credentials: Option<AuthCredentials>,
}

/// Auth credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

/// Filter for web requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilter {
    pub urls: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub tab_id: Option<i32>,
    pub window_id: Option<i32>,
}

/// Extra info spec
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraInfoSpec {
    pub request_headers: Option<bool>,
    pub response_headers: Option<bool>,
    pub blocking: Option<bool>,
}

/// Web request event listener
#[derive(Debug, Clone)]
pub struct WebRequestListener {
    pub extension_id: String,
    pub filter: RequestFilter,
    pub extra_info_spec: ExtraInfoSpec,
}

/// Web request manager
pub struct WebRequestManager {
    listeners: Arc<Mutex<Vec<WebRequestListener>>>,
}

impl WebRequestManager {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_listener(&self, listener: WebRequestListener) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.push(listener);
    }

    pub fn remove_listener(&self, extension_id: &str) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.retain(|l| l.extension_id != extension_id);
    }

    pub fn get_listeners_for_url(&self, url: &str, request_type: &str) -> Vec<WebRequestListener> {
        let listeners = self.listeners.lock().unwrap();
        listeners
            .iter()
            .filter(|l| {
                if let Some(ref urls) = l.filter.urls {
                    if !urls.iter().any(|pattern| self.url_matches(url, pattern)) {
                        return false;
                    }
                }
                if let Some(ref types) = l.filter.types {
                    if !types.contains(&request_type.to_string()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    fn url_matches(&self, url: &str, pattern: &str) -> bool {
        // Simple pattern matching
        if pattern == "<all_urls>" {
            return true;
        }
        if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return url.starts_with(parts[0]) && url.ends_with(parts[1]);
            }
        }
        url == pattern
    }
}

impl Default for WebRequestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Add listener for onBeforeRequest
#[tauri::command]
pub fn webrequest_on_before_request(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onBeforeSendHeaders
#[tauri::command]
pub fn webrequest_on_before_send_headers(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onSendHeaders
#[tauri::command]
pub fn webrequest_on_send_headers(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onHeadersReceived
#[tauri::command]
pub fn webrequest_on_headers_received(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onResponseStarted
#[tauri::command]
pub fn webrequest_on_response_started(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onBeforeRedirect
#[tauri::command]
pub fn webrequest_on_before_redirect(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onCompleted
#[tauri::command]
pub fn webrequest_on_completed(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Add listener for onErrorOccurred
#[tauri::command]
pub fn webrequest_on_error_occurred(
    extension_id: String,
    filter: RequestFilter,
    extra_info_spec: ExtraInfoSpec,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    let listener = WebRequestListener {
        extension_id: extension_id.clone(),
        filter,
        extra_info_spec,
    };
    manager.add_listener(listener);
    Ok(())
}

/// Remove listener
#[tauri::command]
pub fn webrequest_remove_listener(
    extension_id: String,
    event_name: String,
    manager: State<'_, WebRequestManager>,
) -> Result<(), String> {
    manager.remove_listener(&extension_id);
    Ok(())
}

/// Get handler behavior changed
#[tauri::command]
pub fn webrequest_get_handler_behavior_changed(
    extension_id: String,
) -> Result<bool, String> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_filter_serialization() {
        let filter = RequestFilter {
            urls: Some(vec!["<all_urls>".to_string()]),
            types: Some(vec!["main_frame".to_string()]),
            tab_id: None,
            window_id: None,
        };
        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("urls"));
        assert!(json.contains("types"));
    }

    #[test]
    fn test_blocking_response_serialization() {
        let response = BlockingResponse {
            cancel: Some(false),
            redirect_url: None,
            request_headers: None,
            response_headers: None,
            auth_credentials: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("cancel"));
    }

    #[test]
    fn test_url_matching() {
        let manager = WebRequestManager::new();
        assert!(manager.url_matches("https://example.com", "<all_urls>"));
        assert!(manager.url_matches("https://example.com", "https://*.com"));
        assert!(!manager.url_matches("https://example.com", "http://example.com"));
    }

    #[test]
    fn test_web_request_manager_add_listener() {
        let manager = WebRequestManager::new();
        let listener = WebRequestListener {
            extension_id: "test-ext".to_string(),
            filter: RequestFilter {
                urls: Some(vec!["<all_urls>".to_string()]),
                types: None,
                tab_id: None,
                window_id: None,
            },
            extra_info_spec: ExtraInfoSpec {
                request_headers: None,
                response_headers: None,
                blocking: None,
            },
        };
        manager.add_listener(listener);
        let listeners = manager.get_listeners_for_url("https://example.com", "main_frame");
        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].extension_id, "test-ext");
    }

    #[test]
    fn test_web_request_manager_remove_listener() {
        let manager = WebRequestManager::new();
        let listener = WebRequestListener {
            extension_id: "test-ext".to_string(),
            filter: RequestFilter {
                urls: Some(vec!["<all_urls>".to_string()]),
                types: None,
                tab_id: None,
                window_id: None,
            },
            extra_info_spec: ExtraInfoSpec {
                request_headers: None,
                response_headers: None,
                blocking: None,
            },
        };
        manager.add_listener(listener);
        manager.remove_listener("test-ext");
        let listeners = manager.get_listeners_for_url("https://example.com", "main_frame");
        assert_eq!(listeners.len(), 0);
    }

    #[test]
    fn test_web_request_manager_filter_by_type() {
        let manager = WebRequestManager::new();
        let listener = WebRequestListener {
            extension_id: "test-ext".to_string(),
            filter: RequestFilter {
                urls: Some(vec!["<all_urls>".to_string()]),
                types: Some(vec!["main_frame".to_string()]),
                tab_id: None,
                window_id: None,
            },
            extra_info_spec: ExtraInfoSpec {
                request_headers: None,
                response_headers: None,
                blocking: None,
            },
        };
        manager.add_listener(listener);
        let listeners = manager.get_listeners_for_url("https://example.com", "main_frame");
        assert_eq!(listeners.len(), 1);
        let listeners = manager.get_listeners_for_url("https://example.com", "sub_frame");
        assert_eq!(listeners.len(), 0);
    }

    #[test]
    fn test_request_details_serialization() {
        let details = RequestDetails {
            request_id: "123".to_string(),
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            tab_id: Some(1),
            parent_frame_id: Some(0),
            frame_id: Some(1),
            request_body: None,
            initiator: None,
            type_: "main_frame".to_string(),
            timestamp: 1234567890.0,
        };
        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("requestId"));
        assert!(json.contains("url"));
        assert!(json.contains("method"));
    }

    #[test]
    fn test_extra_info_spec_serialization() {
        let spec = ExtraInfoSpec {
            request_headers: Some(true),
            response_headers: Some(false),
            blocking: Some(true),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("requestHeaders"));
        assert!(json.contains("responseHeaders"));
        assert!(json.contains("blocking"));
    }
}
