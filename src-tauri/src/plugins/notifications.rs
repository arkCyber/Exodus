//! Exodus Browser — Web Extension notifications API (chrome.notifications).

use serde::{Deserialize, Serialize};

/// Notification type.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Basic,
    Image,
    List,
    Progress,
}

/// Notification creation options.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<NotificationType>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_icon_mask_url: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_message: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<NotificationButton>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<NotificationItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationButton {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub title: String,
    pub message: String,
}

/// Notification information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInfo {
    pub notification_id: String,
    pub options: NotificationOptions,
}

/// Simple in-memory notification store for MVP.
pub struct NotificationStore {
    notifications: std::sync::Mutex<std::collections::HashMap<String, NotificationInfo>>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
    
    pub fn create(&self, notification_id: String, options: NotificationOptions) -> Result<(), String> {
        let mut store = self.notifications.lock()
            .map_err(|e| format!("Notification store lock error: {e}"))?;
        
        store.insert(notification_id.clone(), NotificationInfo {
            notification_id,
            options,
        });
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn update(&self, notification_id: &str, options: NotificationOptions) -> Result<bool, String> {
        let mut store = self.notifications.lock()
            .map_err(|e| format!("Notification store lock error: {e}"))?;
        
        if let Some(info) = store.get_mut(notification_id) {
            info.options = options;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    #[allow(dead_code)]
    pub fn clear(&self, notification_id: &str) -> Result<bool, String> {
        let mut store = self.notifications.lock()
            .map_err(|e| format!("Notification store lock error: {e}"))?;
        
        Ok(store.remove(notification_id).is_some())
    }
    
    #[allow(dead_code)]
    pub fn get_all(&self) -> Vec<NotificationInfo> {
        let store = self.notifications.lock();
        match store {
            Ok(guard) => guard.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl Default for NotificationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_store() {
        let store = NotificationStore::new();
        
        let options = NotificationOptions {
            r#type: Some(NotificationType::Basic),
            title: Some("Test".to_string()),
            message: Some("Test message".to_string()),
            ..Default::default()
        };
        
        assert!(store.create("test-1".to_string(), options.clone()).is_ok());
        assert!(store.update("test-1", options.clone()).is_ok());
        assert!(store.clear("test-1").unwrap());
        assert!(!store.clear("test-1").unwrap());
    }
}
