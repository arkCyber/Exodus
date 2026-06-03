//! Exodus Browser — chrome.alarms API implementation
//! 
//! Provides scheduling functionality for extensions with high reliability
//! and safety guarantees following aerospace-grade standards.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State};

const MAX_ALARM_NAME_LENGTH: usize = 100;
const MIN_PERIOD_MINUTES: f64 = 1.0;
const MAX_PERIOD_MINUTES: f64 = 525600.0; // 1 year max

/// Validate alarm name
pub fn validate_alarm_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Alarm name cannot be empty".to_string());
    }
    if name.len() > MAX_ALARM_NAME_LENGTH {
        return Err(format!("Alarm name too long (max {} characters)", MAX_ALARM_NAME_LENGTH));
    }
    Ok(())
}

/// Validate period in minutes
pub fn validate_period_minutes(period: f64) -> Result<(), String> {
    if period < MIN_PERIOD_MINUTES {
        return Err(format!("Period too short (min {} minutes)", MIN_PERIOD_MINUTES));
    }
    if period > MAX_PERIOD_MINUTES {
        return Err(format!("Period too long (max {} minutes)", MAX_PERIOD_MINUTES));
    }
    Ok(())
}

/// Alarm information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alarm {
    /// Alarm name
    pub name: String,
    /// Scheduled time (milliseconds since epoch)
    pub scheduled_time: u64,
    /// Period in minutes (if repeating)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_in_minutes: Option<f64>,
    /// Time when alarm was created
    pub created_at: u64,
}

/// Alarm creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmCreateInfo {
    /// When to fire the alarm (milliseconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<u64>,
    /// Delay in minutes before firing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_in_minutes: Option<f64>,
    /// Period in minutes for repeating alarms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_in_minutes: Option<f64>,
}

impl AlarmCreateInfo {
    /// Validate alarm creation options
    pub fn validate(&self) -> Result<(), String> {
        let count = [self.when.is_some(), self.delay_in_minutes.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if count > 1 {
            return Err("Cannot specify both 'when' and 'delayInMinutes'".to_string());
        }
        
        if count == 0 && self.period_in_minutes.is_none() {
            return Err("Must specify either 'when', 'delayInMinutes', or 'periodInMinutes'".to_string());
        }
        
        if let Some(period) = self.period_in_minutes {
            if period <= 0.0 || !period.is_finite() {
                return Err("periodInMinutes must be a positive finite number".to_string());
            }
        }
        
        if let Some(delay) = self.delay_in_minutes {
            if delay < 0.0 || !delay.is_finite() {
                return Err("delayInMinutes must be a non-negative finite number".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Calculate scheduled time from options
    pub fn calculate_scheduled_time(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        if let Some(when) = self.when {
            when
        } else if let Some(delay) = self.delay_in_minutes {
            now + (delay * 60_000.0) as u64
        } else {
            // Only period_in_minutes specified, fire immediately
            now
        }
    }
}

/// Alarms manager for extensions
pub struct AlarmsManager {
    alarms: Arc<Mutex<HashMap<String, HashMap<String, Alarm>>>>, // extension_id -> alarm_name -> alarm
    alarm_listeners: Arc<Mutex<HashMap<String, HashSet<String>>>>, // extension_id -> listener_ids
    storage_path: PathBuf,
}

impl AlarmsManager {
    /// Create a new alarms manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            alarms: Arc::new(Mutex::new(HashMap::new())),
            alarm_listeners: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Create an alarm for an extension
    pub fn create(
        &self,
        extension_id: &str,
        name: &str,
        info: AlarmCreateInfo,
    ) -> Result<Alarm, String> {
        info.validate()?;
        
        let scheduled_time = info.calculate_scheduled_time();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        let alarm = Alarm {
            name: name.to_string(),
            scheduled_time,
            period_in_minutes: info.period_in_minutes,
            created_at: now,
        };
        
        {
            let mut alarms = self.alarms.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            alarms
                .entry(extension_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(name.to_string(), alarm.clone());
        }
        self.save_to_disk()
            .map_err(|e| format!("Failed to save alarm: {}", e))?;
        Ok(alarm)
    }
    
    /// Get an alarm by name
    pub fn get(&self, extension_id: &str, name: &str) -> Option<Alarm> {
        let alarms = self.alarms.lock().ok()?;
        alarms.get(extension_id)?.get(name).cloned()
    }
    
    /// Get all alarms for an extension
    pub fn get_all(&self, extension_id: &str) -> Vec<Alarm> {
        let alarms = match self.alarms.lock() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Failed to acquire alarms lock: {}", e);
                return Vec::new();
            }
        };
        
        alarms.get(extension_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Clear all alarms for an extension
    pub fn clear_all(&self, extension_id: &str) -> Result<(), String> {
        {
            let mut alarms = self.alarms.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            alarms.remove(extension_id);
        }
        self.save_to_disk()
            .map_err(|e| format!("Failed to clear alarms: {}", e))?;
        Ok(())
    }
    
    /// Clear a specific alarm
    pub fn clear(&self, extension_id: &str, name: &str) -> Result<(), String> {
        {
            let mut alarms = self.alarms.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            if let Some(ext_alarms) = alarms.get_mut(extension_id) {
                ext_alarms.remove(name);
                if ext_alarms.is_empty() {
                    alarms.remove(extension_id);
                }
            }
        }
        self.save_to_disk()
            .map_err(|e| format!("Failed to clear alarm: {}", e))?;
        Ok(())
    }
    
    /// Check for due alarms and return them (for background polling)
    pub fn check_due_alarms(&self, extension_id: &str) -> Vec<Alarm> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        let due_alarms = {
            let mut alarms = match self.alarms.lock() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Failed to acquire alarms lock: {}", e);
                    return Vec::new();
                }
            };
            let mut due_alarms = Vec::new();
            if let Some(ext_alarms) = alarms.get_mut(extension_id) {
                let mut to_update = Vec::new();
                let mut to_remove = Vec::new();
                for (name, alarm) in ext_alarms.iter() {
                    if alarm.scheduled_time <= now {
                        due_alarms.push(alarm.clone());
                        if let Some(period) = alarm.period_in_minutes {
                            to_update.push((
                                name.clone(),
                                Alarm {
                                    name: name.clone(),
                                    scheduled_time: now + (period * 60_000.0) as u64,
                                    period_in_minutes: Some(period),
                                    created_at: alarm.created_at,
                                },
                            ));
                        } else {
                            to_remove.push(name.clone());
                        }
                    }
                }
                for (name, new_alarm) in to_update {
                    ext_alarms.insert(name, new_alarm);
                }
                for name in to_remove {
                    ext_alarms.remove(&name);
                }
                if ext_alarms.is_empty() {
                    alarms.remove(extension_id);
                }
            }
            due_alarms
        };
        let _ = self.save_to_disk();
        due_alarms
    }
    
    /// Register an alarm listener for an extension
    pub fn add_listener(&self, extension_id: &str, listener_id: String) {
        let mut listeners = match self.alarm_listeners.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire alarm_listeners lock: {}", e);
                return;
            }
        };
        
        listeners.entry(extension_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(listener_id);
    }
    
    /// Remove an alarm listener
    pub fn remove_listener(&self, extension_id: &str, listener_id: &str) {
        let mut listeners = match self.alarm_listeners.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire alarm_listeners lock: {}", e);
                return;
            }
        };
        
        if let Some(ext_listeners) = listeners.get_mut(extension_id) {
            ext_listeners.remove(listener_id);
            
            if ext_listeners.is_empty() {
                listeners.remove(extension_id);
            }
        }
    }
    
    /// Get listener count for an extension
    pub fn listener_count(&self, extension_id: &str) -> usize {
        let listeners = match self.alarm_listeners.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire alarm_listeners lock: {}", e);
                return 0;
            }
        };
        
        listeners.get(extension_id).map(|s| s.len()).unwrap_or(0)
    }
    
    /// Clean up alarms for an extension (called on uninstall)
    pub fn remove_extension(&self, extension_id: &str) -> Result<(), String> {
        {
            let mut alarms = self.alarms.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            alarms.remove(extension_id);
        }
        {
            let mut listeners = self.alarm_listeners.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            listeners.remove(extension_id);
        }
        self.save_to_disk()
            .map_err(|e| format!("Failed to cleanup extension: {}", e))?;
        Ok(())
    }
    
    /// Load alarms from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("alarms.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let data: HashMap<String, HashMap<String, Alarm>> = serde_json::from_str(&content)?;
            match self.alarms.lock() {
                Ok(mut alarms) => *alarms = data,
                Err(e) => {
                    eprintln!("Failed to acquire alarms lock: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Save alarms to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("alarms.json");
        
        let alarms = self.alarms.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*alarms)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Create an alarm
#[tauri::command]
pub fn alarm_create(
    extension_id: String,
    name: String,
    info: AlarmCreateInfo,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<Alarm, String> {
    manager.create(&extension_id, &name, info)
}

/// Get an alarm
#[tauri::command]
pub fn alarm_get(
    extension_id: String,
    name: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<Option<Alarm>, String> {
    Ok(manager.get(&extension_id, &name))
}

/// Get all alarms for an extension
#[tauri::command]
pub fn alarm_get_all(
    extension_id: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<Vec<Alarm>, String> {
    Ok(manager.get_all(&extension_id))
}

/// Clear all alarms for an extension
#[tauri::command]
pub fn alarm_clear_all(
    extension_id: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<(), String> {
    manager.clear_all(&extension_id)
}

/// Clear a specific alarm
#[tauri::command]
pub fn alarm_clear(
    extension_id: String,
    name: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<(), String> {
    manager.clear(&extension_id, &name)
}

/// Check for due alarms (called by background polling)
#[tauri::command]
pub fn alarm_check_due(
    extension_id: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<Vec<Alarm>, String> {
    Ok(manager.check_due_alarms(&extension_id))
}

/// Add alarm listener
#[tauri::command]
pub fn alarm_add_listener(
    extension_id: String,
    listener_id: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<(), String> {
    manager.add_listener(&extension_id, listener_id);
    Ok(())
}

/// Remove alarm listener
#[tauri::command]
pub fn alarm_remove_listener(
    extension_id: String,
    listener_id: String,
    manager: State<'_, Arc<AlarmsManager>>,
) -> Result<(), String> {
    manager.remove_listener(&extension_id, &listener_id);
    Ok(())
}

/// Interval between alarm poll ticks (seconds). 50s avoids macOS busy cursor vs 5s.
pub const ALARM_POLL_INTERVAL_SECS: u64 = 50;

/// Poll due alarms periodically and fire `chrome.alarms.onAlarm` in background hosts.
pub fn start_poll_scheduler(app: tauri::AppHandle, alarms: Arc<AlarmsManager>) {
    use super::background::background_webview_label;

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(ALARM_POLL_INTERVAL_SECS));
            let extensions: Vec<String> = app
                .try_state::<crate::plugins::ExtensionState>()
                .and_then(|state| {
                    state.lock().ok().map(|mgr| {
                        mgr.list()
                            .into_iter()
                            .filter(|e| e.enabled)
                            .map(|e| e.id)
                            .collect()
                    })
                })
                .unwrap_or_default();
            for ext_id in extensions {
                let due = alarms.check_due_alarms(&ext_id);
                for alarm in due {
                    let alarm_json = match serde_json::to_string(&alarm) {
                        Ok(json) => json,
                        Err(_) => continue,
                    };
                    let script = super::runtime::deliver_alarm_script(&alarm_json);
                    let label = background_webview_label(&ext_id);
                    if let Some(webview) = app.get_webview(&label) {
                        let _ = webview.eval(&script);
                    }
                    let _ = app.emit(
                        "exodus-extension-alarm",
                        serde_json::json!({
                            "extensionId": ext_id,
                            "name": alarm.name,
                            "scheduledTime": alarm.scheduled_time,
                        }),
                    );
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    fn temp_manager() -> AlarmsManager {
        let dir = temp_dir().join(format!("exodus_alarms_{}", uuid::Uuid::new_v4()));
        AlarmsManager::new(dir).expect("manager")
    }
    
    #[test]
    fn test_alarm_create_validation() {
        // Valid: when specified
        let info = AlarmCreateInfo {
            when: Some(1000),
            delay_in_minutes: None,
            period_in_minutes: None,
        };
        assert!(info.validate().is_ok());
        
        // Valid: delay specified
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_ok());
        
        // Valid: period only
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: None,
            period_in_minutes: Some(5.0),
        };
        assert!(info.validate().is_ok());
        
        // Invalid: both when and delay
        let info = AlarmCreateInfo {
            when: Some(1000),
            delay_in_minutes: Some(1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_err());
        
        // Invalid: negative period
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: None,
            period_in_minutes: Some(-1.0),
        };
        assert!(info.validate().is_err());
        
        // Invalid: negative delay
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(-1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_err());
    }
    
    #[test]
    fn test_alarm_lifecycle() {
        let manager = temp_manager();
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(0.001), // 0.06 seconds
            period_in_minutes: None,
        };
        
        let alarm = manager.create("ext1", "test-alarm", info).unwrap();
        assert_eq!(alarm.name, "test-alarm");
        
        let retrieved = manager.get("ext1", "test-alarm").unwrap();
        assert_eq!(retrieved.name, "test-alarm");
        
        let all = manager.get_all("ext1");
        assert_eq!(all.len(), 1);
        
        manager.clear("ext1", "test-alarm").unwrap();
        assert!(manager.get("ext1", "test-alarm").is_none());
    }
    
    #[test]
    fn test_repeating_alarm() {
        let manager = temp_manager();
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(0.001),
            period_in_minutes: Some(0.001),
        };
        
        manager.create("ext2", "repeating", info).unwrap();
        
        // Wait for alarm to fire
        std::thread::sleep(Duration::from_millis(100));
        
        let due = manager.check_due_alarms("ext2");
        assert!(!due.is_empty());
        
        // Alarm should still exist (repeating)
        let alarm = manager.get("ext2", "repeating");
        assert!(alarm.is_some());
    }
    
    #[test]
    fn test_clear_all() {
        let manager = temp_manager();
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(1.0),
            period_in_minutes: None,
        };
        
        manager.create("ext3", "alarm1", info.clone()).unwrap();
        manager.create("ext3", "alarm2", info).unwrap();
        
        assert_eq!(manager.get_all("ext3").len(), 2);
        
        manager.clear_all("ext3").unwrap();
        assert_eq!(manager.get_all("ext3").len(), 0);
    }
    
    #[test]
    fn test_persistence() {
        let dir = temp_dir().join(format!("exodus_alarms_persist_{}", uuid::Uuid::new_v4()));
        
        {
            let manager = AlarmsManager::new(dir.clone()).unwrap();
            let info = AlarmCreateInfo {
                when: Some(10000),
                delay_in_minutes: None,
                period_in_minutes: None,
            };
            manager.create("ext4", "persist-alarm", info).unwrap();
        }
        
        {
            let manager = AlarmsManager::new(dir).unwrap();
            let alarm = manager.get("ext4", "persist-alarm");
            assert!(alarm.is_some());
            assert_eq!(alarm.unwrap().scheduled_time, 10000);
        }
    }
}
