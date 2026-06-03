//! Exodus Browser — Cross-Device Sync Backend
//!
//! Provides the foundation for cross-device synchronization including
//! authentication, data sync, and conflict resolution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub last_sync: Option<DateTime<Utc>>,
    pub device_count: u32,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: String,
    pub user_id: String,
    pub device_name: String,
    pub device_type: String, // desktop, mobile, tablet
    pub os: String,
    pub last_seen: DateTime<Utc>,
    pub is_active: bool,
}

/// Sync data item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDataItem {
    pub id: String,
    pub user_id: String,
    pub data_type: String, // bookmarks, history, settings
    pub data: serde_json::Value,
    pub version: u64,
    pub device_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

/// Sync conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub conflict_id: String,
    pub user_id: String,
    pub data_type: String,
    pub item_id: String,
    pub local_version: SyncDataItem,
    pub remote_version: SyncDataItem,
    pub conflict_type: String, // modify_modify, delete_modify
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
}

/// Sync backend state
pub struct SyncBackendState {
    db: Arc<Db>,
    users: Arc<Mutex<HashMap<String, UserAccount>>>,
    devices: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    sync_data: Arc<Mutex<HashMap<String, Vec<SyncDataItem>>>>,
    conflicts: Arc<Mutex<HashMap<String, Vec<SyncConflict>>>>,
}

impl SyncBackendState {
    /// Create sync backend state with sled database
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let db = sled::open(db_path).map_err(|e| format!("Failed to open sync backend DB: {}", e))?;
        
        let users = Self::load_users(&db)?;
        let devices = Self::load_devices(&db)?;
        let sync_data = Self::load_sync_data(&db)?;
        let conflicts = Self::load_conflicts(&db)?;
        
        Ok(Self {
            db: Arc::new(db),
            users: Arc::new(Mutex::new(users)),
            devices: Arc::new(Mutex::new(devices)),
            sync_data: Arc::new(Mutex::new(sync_data)),
            conflicts: Arc::new(Mutex::new(conflicts)),
        })
    }

    /// Load users from database
    fn load_users(db: &Db) -> Result<HashMap<String, UserAccount>, String> {
        let mut users = HashMap::new();
        if let Some(value) = db.get(b"users").map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(loaded) = bincode::deserialize::<Vec<UserAccount>>(&value) {
                for user in loaded {
                    users.insert(user.user_id.clone(), user);
                }
            }
        }
        Ok(users)
    }

    /// Load devices from database
    fn load_devices(db: &Db) -> Result<HashMap<String, DeviceInfo>, String> {
        let mut devices = HashMap::new();
        if let Some(value) = db.get(b"devices").map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(loaded) = bincode::deserialize::<Vec<DeviceInfo>>(&value) {
                for device in loaded {
                    devices.insert(device.device_id.clone(), device);
                }
            }
        }
        Ok(devices)
    }

    /// Load sync data from database
    fn load_sync_data(db: &Db) -> Result<HashMap<String, Vec<SyncDataItem>>, String> {
        let mut sync_data = HashMap::new();
        for item_result in db.iter() {
            let (key, value) = item_result.map_err(|e| format!("DB iteration error: {}", e))?;
            if key.starts_with(b"sync_data:") {
                if let Ok(item) = bincode::deserialize::<SyncDataItem>(&value) {
                    sync_data.entry(item.user_id.clone()).or_insert_with(Vec::new).push(item);
                }
            }
        }
        Ok(sync_data)
    }

    /// Load conflicts from database
    fn load_conflicts(db: &Db) -> Result<HashMap<String, Vec<SyncConflict>>, String> {
        let mut conflicts = HashMap::new();
        for item_result in db.iter() {
            let (key, value) = item_result.map_err(|e| format!("DB iteration error: {}", e))?;
            if key.starts_with(b"conflict:") {
                if let Ok(conflict) = bincode::deserialize::<SyncConflict>(&value) {
                    conflicts.entry(conflict.user_id.clone()).or_insert_with(Vec::new).push(conflict);
                }
            }
        }
        Ok(conflicts)
    }

    /// Create user account
    pub fn create_user(&self, email: String, display_name: String) -> Result<UserAccount, String> {
        let user_id = Uuid::new_v4().to_string();
        let user = UserAccount {
            user_id: user_id.clone(),
            email,
            display_name,
            created_at: Utc::now(),
            last_sync: None,
            device_count: 0,
        };

        let mut users = self.users.lock().map_err(|e| format!("Lock error: {}", e))?;
        users.insert(user_id.clone(), user.clone());
        
        let value = bincode::serialize(&users.values().cloned().collect::<Vec<_>>())
            .map_err(|e| format!("Serialization error: {}", e))?;
        self.db.insert(b"users", value)
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(user)
    }

    /// Register device for user
    pub fn register_device(
        &self,
        user_id: String,
        device_name: String,
        device_type: String,
        os: String,
    ) -> Result<DeviceInfo, String> {
        let device_id = Uuid::new_v4().to_string();
        let device = DeviceInfo {
            device_id: device_id.clone(),
            user_id: user_id.clone(),
            device_name,
            device_type,
            os,
            last_seen: Utc::now(),
            is_active: true,
        };

        let mut devices = self.devices.lock().map_err(|e| format!("Lock error: {}", e))?;
        devices.insert(device_id.clone(), device.clone());
        
        let value = bincode::serialize(&devices.values().cloned().collect::<Vec<_>>())
            .map_err(|e| format!("Serialization error: {}", e))?;
        self.db.insert(b"devices", value)
            .map_err(|e| format!("DB insert error: {}", e))?;

        // Update user device count
        let mut users = self.users.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(user) = users.get_mut(&user_id) {
            user.device_count += 1;
        }
        
        Ok(device)
    }

    /// Upload sync data
    pub fn upload_sync_data(&self, data: SyncDataItem) -> Result<(), String> {
        let key = format!("sync_data:{}", data.id);
        let value = bincode::serialize(&data).map_err(|e| format!("Serialization error: {}", e))?;
        
        self.db.insert(key.as_bytes(), value)
            .map_err(|e| format!("DB insert error: {}", e))?;

        let mut sync_data = self.sync_data.lock().map_err(|e| format!("Lock error: {}", e))?;
        sync_data.entry(data.user_id.clone()).or_insert_with(Vec::new).push(data);
        
        Ok(())
    }

    /// Download sync data for user
    pub fn download_sync_data(&self, user_id: &str, data_type: Option<String>) -> Result<Vec<SyncDataItem>, String> {
        let sync_data = self.sync_data.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(items) = sync_data.get(user_id) {
            if let Some(data_type) = data_type {
                Ok(items.iter().filter(|item| item.data_type == data_type).cloned().collect())
            } else {
                Ok(items.clone())
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Detect and create conflicts
    pub fn detect_conflicts(&self, user_id: &str) -> Result<Vec<SyncConflict>, String> {
        let sync_data = self.sync_data.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut conflicts = Vec::new();
        
        if let Some(items) = sync_data.get(user_id) {
            // Group by item_id to detect conflicts
            let mut item_groups: HashMap<String, Vec<&SyncDataItem>> = HashMap::new();
            for item in items {
                if !item.deleted {
                    item_groups.entry(item.id.clone()).or_insert_with(Vec::new).push(item);
                }
            }
            
            // Detect conflicts: multiple versions of same item
            for (item_id, versions) in item_groups {
                if versions.len() > 1 {
                    // Sort by version
                    let mut sorted_versions = versions.clone();
                    sorted_versions.sort_by(|a, b| b.version.cmp(&a.version));
                    
                    if sorted_versions.len() >= 2 {
                        let conflict = SyncConflict {
                            conflict_id: Uuid::new_v4().to_string(),
                            user_id: user_id.to_string(),
                            data_type: sorted_versions[0].data_type.clone(),
                            item_id: item_id.clone(),
                            local_version: sorted_versions[0].clone(),
                            remote_version: sorted_versions[1].clone(),
                            conflict_type: "modify_modify".to_string(),
                            created_at: Utc::now(),
                            resolved: false,
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }
        
        Ok(conflicts)
    }

    /// Resolve conflict
    pub fn resolve_conflict(&self, conflict_id: &str, keep_local: bool) -> Result<(), String> {
        let mut conflicts = self.conflicts.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(conflict) = conflicts.values_mut().flatten().find(|c| c.conflict_id == conflict_id) {
            conflict.resolved = true;
            
            // Remove the rejected version
            let version_to_remove = if keep_local {
                &conflict.remote_version
            } else {
                &conflict.local_version
            };
            
            let key = format!("sync_data:{}", version_to_remove.id);
            self.db.remove(key.as_bytes())
                .map_err(|e| format!("DB remove error: {}", e))?;
            
            let mut sync_data = self.sync_data.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(items) = sync_data.get_mut(&conflict.user_id) {
                items.retain(|item| item.id != version_to_remove.id);
            }
        }
        
        Ok(())
    }

    /// Get user account
    pub fn get_user(&self, user_id: &str) -> Option<UserAccount> {
        let users = self.users.lock().ok()?;
        users.get(user_id).cloned()
    }

    /// Get user by email
    pub fn get_user_by_email(&self, email: &str) -> Option<UserAccount> {
        let users = self.users.lock().ok()?;
        users.values().find(|u| u.email == email).cloned()
    }

    /// Update last sync time
    pub fn update_last_sync(&self, user_id: &str) -> Result<(), String> {
        let mut users = self.users.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(user) = users.get_mut(user_id) {
            user.last_sync = Some(Utc::now());
        }
        Ok(())
    }

    /// Get devices for user
    pub fn get_user_devices(&self, user_id: &str) -> Result<Vec<DeviceInfo>, String> {
        let devices = self.devices.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(devices.values().filter(|d| d.user_id == user_id).cloned().collect())
    }

    /// Get conflicts for user
    pub fn get_user_conflicts(&self, user_id: &str) -> Result<Vec<SyncConflict>, String> {
        let conflicts = self.conflicts.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(conflicts.get(user_id).cloned().unwrap_or_default())
    }
}
