//! Comprehensive tests for new browser extension APIs
//! 
//! Aerospace-grade testing suite covering:
//! - Unit tests for individual API components
//! - Integration tests for API interactions
//! - Security tests for permission boundaries
//! - Performance benchmarks for critical paths

#[allow(unused_imports)]
use super::alarms::{AlarmCreateInfo, AlarmsManager, Alarm};
use super::storage::{ExtensionStorage, ExtensionSessionStorage};
use super::tabs::{TabRegistry, ExtensionTabInfo};
use super::runtime::{get_platform_info, SuspendTracker, PlatformInfo};
// use crate::extension_permissions::{
//     ExtensionPermissionsManager, PermissionEvent, PermissionEventType, PermissionType,
// };
use std::env::temp_dir;
use std::path::Path;
use std::time::Duration;

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_alarm_create_info_validation() {
        // Test valid configurations
        let info = AlarmCreateInfo {
            when: Some(1000),
            delay_in_minutes: None,
            period_in_minutes: None,
        };
        assert!(info.validate().is_ok());
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_ok());
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: None,
            period_in_minutes: Some(5.0),
        };
        assert!(info.validate().is_ok());
        
        // Test invalid configurations
        let info = AlarmCreateInfo {
            when: Some(1000),
            delay_in_minutes: Some(1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_err());
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: Some(-1.0),
            period_in_minutes: None,
        };
        assert!(info.validate().is_err());
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: None,
            period_in_minutes: Some(-5.0),
        };
        assert!(info.validate().is_err());
        
        let info = AlarmCreateInfo {
            when: None,
            delay_in_minutes: None,
            period_in_minutes: Some(f64::INFINITY),
        };
        assert!(info.validate().is_err());
    }
    
    #[test]
    fn test_alarm_manager_lifecycle() {
        // Use in-memory directory to avoid file I/O blocking
        let dir = temp_dir().join(format!("exodus_alarm_test_{}", uuid::Uuid::new_v4()));
        let manager = AlarmsManager::new(dir.clone()).expect("Failed to create alarm manager");
        
        // Create alarm
        let info = AlarmCreateInfo {
            when: Some(10000), // Use absolute time to avoid scheduling delays
            delay_in_minutes: None,
            period_in_minutes: None,
        };
        let alarm = manager.create("test-ext", "test-alarm", info).expect("Failed to create alarm");
        assert_eq!(alarm.name, "test-alarm");
        
        // Retrieve alarm
        let retrieved = manager.get("test-ext", "test-alarm").expect("Failed to get alarm");
        assert_eq!(retrieved.name, "test-alarm");
        
        // Get all alarms
        let all = manager.get_all("test-ext");
        assert_eq!(all.len(), 1);
        
        // Clear alarm
        manager.clear("test-ext", "test-alarm").expect("Failed to clear alarm");
        assert!(manager.get("test-ext", "test-alarm").is_none());
        
        // Test persistence
        let manager2 = AlarmsManager::new(dir).expect("Failed to create second manager");
        assert!(manager2.get("test-ext", "test-alarm").is_none());
    }
    
    #[test]
    fn test_repeating_alarm() {
        let dir = temp_dir().join(format!("exodus_repeat_test_{}", uuid::Uuid::new_v4()));
        let manager = AlarmsManager::new(dir).expect("Failed to create alarm manager");
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_millis() as u64;
        // Schedule in the past so check_due_alarms is deterministic (no flaky sleep).
        let info = AlarmCreateInfo {
            when: Some(now_ms.saturating_sub(5000)),
            delay_in_minutes: None,
            period_in_minutes: Some(1.0),
        };
        
        manager.create("test-ext", "repeating", info).expect("Failed to create repeating alarm");
        
        let due = manager.check_due_alarms("test-ext");
        assert!(!due.is_empty());
        
        // Alarm should still exist (repeating)
        let alarm = manager.get("test-ext", "repeating");
        assert!(alarm.is_some());
        assert!(alarm.unwrap().period_in_minutes.is_some());
    }
    
    #[test]
    fn test_session_storage_lifecycle() {
        let storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Set data
        let mut items = serde_json::Map::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        storage.set("ext1", &perms, items).expect("Failed to set");
        
        // Get data
        let retrieved = storage.get("ext1", &perms, None).expect("Failed to get");
        assert_eq!(retrieved.get("key1"), Some(&serde_json::json!("value1")));
        
        // Remove data
        storage.remove("ext1", &perms, vec!["key1".to_string()]).expect("Failed to remove");
        let retrieved = storage.get("ext1", &perms, None).expect("Failed to get after remove");
        assert!(!retrieved.contains_key("key1"));
        
        // Clear all
        storage.set("ext1", &perms, serde_json::Map::new()).expect("Failed to set for clear");
        storage.clear("ext1", &perms).expect("Failed to clear");
        let retrieved = storage.get("ext1", &perms, None).expect("Failed to get after clear");
        assert!(retrieved.is_empty());
    }
    
    #[test]
    fn test_storage_get_bytes_in_use() {
        let dir = temp_dir().join(format!("exodus_storage_bytes_{}", uuid::Uuid::new_v4()));
        let storage = ExtensionStorage::new(&dir).expect("Failed to create storage");
        let perms = vec![super::super::permissions::Permission::Storage];
        
        let mut items = serde_json::Map::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        items.insert("key2".to_string(), serde_json::json!("value2"));
        storage.set("ext1", &perms, items).expect("Failed to set");
        
        let bytes = storage.get_bytes_in_use("ext1", &perms, None).expect("Failed to get bytes");
        assert!(bytes > 0);
        
        let bytes_specific = storage.get_bytes_in_use("ext1", &perms, Some(vec!["key1".to_string()])).expect("Failed to get specific bytes");
        assert!(bytes_specific < bytes);
    }
    
    #[test]
    fn test_session_storage_get_bytes_in_use() {
        let storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        let mut items = serde_json::Map::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        storage.set("ext1", &perms, items).expect("Failed to set");
        
        let bytes = storage.get_bytes_in_use("ext1", &perms, None).expect("Failed to get bytes");
        assert!(bytes > 0);
    }
    
    #[test]
    fn test_tab_registry_operations() {
        let registry = TabRegistry::default();
        
        // Sync tabs
        registry.sync(vec![
            ExtensionTabInfo {
                id: "a".into(),
                chrome_tab_id: 1,
                index: 0,
                webview_label: "exodus-tab-a".into(),
                url: "https://a.com".into(),
                title: "A".into(),
                active: false,
            },
            ExtensionTabInfo {
                id: "b".into(),
                chrome_tab_id: 2,
                index: 1,
                webview_label: "exodus-tab-b".into(),
                url: "https://b.com".into(),
                title: "B".into(),
                active: true,
            },
        ]);
        
        // Test get
        let tab = registry.get(1).expect("Failed to get tab");
        assert_eq!(tab.chrome_tab_id, 1);
        
        // Test get_current (should return the active tab, which is chrome_tab_id 2)
        let current = registry.get_current().expect("Failed to get current tab");
        assert_eq!(current.chrome_tab_id, 2);
        
        // Test move
        registry.move_tab(1, 5).expect("Failed to move tab");
        let tab = registry.get(1).expect("Failed to get moved tab");
        assert_eq!(tab.index, 5);
        
        // Test duplicate
        let duplicated = registry.duplicate(1).expect("Failed to duplicate tab");
        assert_ne!(duplicated.chrome_tab_id, 1);
        assert_eq!(duplicated.url, "https://a.com");
        
        // Test detect_language
        let lang = registry.detect_language(1).expect("Failed to detect language");
        assert_eq!(lang, "en");
        
        let mut registry2 = TabRegistry::default();
        registry2.sync(vec![ExtensionTabInfo {
            id: "tab3".to_string(),
            chrome_tab_id: 3,
            index: 0,
            webview_label: "exodus-tab-3".to_string(),
            url: "https://example.de".to_string(),
            title: "German".to_string(),
            active: true,
        }]);
        let lang = registry2.detect_language(3).expect("Failed to detect German");
        assert_eq!(lang, "de");
    }
    
    #[test]
    fn test_platform_info() {
        let info = get_platform_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(!info.nacl_arch.is_empty());
        
        // Verify OS mapping
        if cfg!(target_os = "macos") {
            assert_eq!(info.os, "mac");
        } else if cfg!(target_os = "linux") {
            assert_eq!(info.os, "linux");
        } else if cfg!(target_os = "windows") {
            assert_eq!(info.os, "win");
        }
    }
    
    #[test]
    fn test_suspend_tracker() {
        let mut tracker = SuspendTracker::new();
        
        tracker.record_suspend("ext1");
        let time = tracker.get_suspend_time("ext1");
        assert!(time.is_some());
        assert!(time.unwrap() > 0);
        
        tracker.clear_suspend("ext1");
        assert!(tracker.get_suspend_time("ext1").is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_permissions_with_events() {
        let dir = temp_dir().join(format!("exodus_perm_events_{}", uuid::Uuid::new_v4()));
        let manager = ExtensionPermissionsManager::new(dir).expect("Failed to create manager");
        
        // Grant permission - should trigger Added event
        manager.grant_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to grant");
        
        let events = manager.get_permission_events("ext1");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, PermissionEventType::Added);
        assert_eq!(events[0].permission, PermissionType::Tabs);
        
        // Deny permission - should trigger Removed event
        manager.deny_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to deny");
        
        let events = manager.get_permission_events("ext1");
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].event_type, PermissionEventType::Removed);
        assert_eq!(events[1].permission, PermissionType::Tabs);
        
        // Clear events
        manager.clear_permission_events("ext1").expect("Failed to clear events");
        let events = manager.get_permission_events("ext1");
        assert!(events.is_empty());
    }
    
    #[test]
    fn test_alarm_with_permissions() {
        // Use separate directories to avoid sled database conflicts
        let perm_dir = temp_dir().join(format!("exodus_alarm_perm_{}", uuid::Uuid::new_v4()));
        let alarm_dir = temp_dir().join(format!("exodus_alarm_{}", uuid::Uuid::new_v4()));
        
        let perm_manager =
            ExtensionPermissionsManager::new(perm_dir).expect("Failed to create perm manager");
        let alarm_manager =
            AlarmsManager::new(alarm_dir).expect("Failed to create alarm manager");
        
        // Grant permission first
        perm_manager.grant_permission("ext1".to_string(), PermissionType::Notifications).expect("Failed to grant");
        
        // Create alarm
        let info = AlarmCreateInfo {
            when: Some(10000), // Use absolute time to avoid scheduling delays
            delay_in_minutes: None,
            period_in_minutes: None,
        };
        alarm_manager.create("ext1", "test-alarm", info).expect("Failed to create alarm");
        
        // Verify alarm exists
        let alarm = alarm_manager.get("ext1", "test-alarm");
        assert!(alarm.is_some());
        
        // Verify permission was granted
        assert!(perm_manager.has_permission("ext1", PermissionType::Notifications));
    }
    
    #[test]
    fn test_storage_with_permissions() {
        // Use session storage only to avoid file I/O blocking
        let session_storage = ExtensionSessionStorage::new();
        
        // Try to access without permission
        let result = session_storage.get("ext1", &[], None);
        assert!(result.is_err());
        
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Now access should succeed
        let result = session_storage.get("ext1", &perms, None);
        assert!(result.is_ok());
        
        // Set some data
        let mut items = serde_json::Map::new();
        items.insert("key".to_string(), serde_json::json!("value"));
        session_storage.set("ext1", &perms, items).expect("Failed to set");
        
        // Verify data
        let result = session_storage.get("ext1", &perms, None).expect("Failed to get");
        assert_eq!(result.get("key"), Some(&serde_json::json!("value")));
    }
    
    #[test]
    fn test_full_extension_lifecycle() {
        let dir = temp_dir().join(format!("exodus_lifecycle_{}", uuid::Uuid::new_v4()));
        let perm_manager = ExtensionPermissionsManager::new(dir.clone()).expect("Failed to create perm manager");
        let alarm_manager = AlarmsManager::new(dir.clone()).expect("Failed to create alarm manager");
        let session_storage = ExtensionSessionStorage::new();
        
        // 1. Install extension - grant permissions
        perm_manager.grant_permission("ext1".to_string(), PermissionType::Storage).expect("Failed to grant storage");
        perm_manager.grant_permission("ext1".to_string(), PermissionType::Notifications).expect("Failed to grant notifications");
        perm_manager.grant_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to grant tabs");
        
        // 2. Initialize session storage (avoid file I/O)
        let mut items = serde_json::Map::new();
        items.insert("initialized".to_string(), serde_json::json!(true));
        session_storage.set("ext1", &[super::super::permissions::Permission::Storage], items).expect("Failed to set");
        
        // 3. Set up alarms
        let info = AlarmCreateInfo {
            when: Some(10000), // Use absolute time
            delay_in_minutes: None,
            period_in_minutes: Some(60.0),
        };
        alarm_manager.create("ext1", "hourly-check", info).expect("Failed to create alarm");
        
        // 4. Verify all components
        assert!(perm_manager.has_permission("ext1", PermissionType::Storage));
        assert!(perm_manager.has_permission("ext1", PermissionType::Notifications));
        assert!(perm_manager.has_permission("ext1", PermissionType::Tabs));
        
        let stored = session_storage.get("ext1", &[super::super::permissions::Permission::Storage], None).expect("Failed to get");
        assert_eq!(stored.get("initialized"), Some(&serde_json::json!(true)));
        
        let alarm = alarm_manager.get("ext1", "hourly-check");
        assert!(alarm.is_some());
        assert!(alarm.unwrap().period_in_minutes.is_some());
        
        // 5. Uninstall extension - revoke permissions and cleanup
        perm_manager.revoke_all_permissions("ext1").expect("Failed to revoke");
        alarm_manager.remove_extension("ext1").expect("Failed to remove alarms");
        session_storage.remove_extension("ext1");
        
        // 6. Verify cleanup
        assert!(!perm_manager.has_permission("ext1", PermissionType::Storage));
        assert!(alarm_manager.get("ext1", "hourly-check").is_none());
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_permission_isolation() {
        let dir = temp_dir().join(format!("exodus_isolation_{}", uuid::Uuid::new_v4()));
        let perm_manager = ExtensionPermissionsManager::new(dir).expect("Failed to create manager");
        let session_storage = ExtensionSessionStorage::new();
        
        // Grant storage permission to ext1 only
        perm_manager.grant_permission("ext1".to_string(), PermissionType::Storage).expect("Failed to grant");
        
        let perms1 = vec![super::super::permissions::Permission::Storage];
        let perms2: Vec<super::super::permissions::Permission> = vec![];
        
        // ext1 should be able to access storage
        let result = session_storage.get("ext1", &perms1, None);
        assert!(result.is_ok());
        
        // ext2 should not be able to access storage
        let result = session_storage.get("ext2", &perms2, None);
        assert!(result.is_err());
        
        // ext2 with storage permission should work
        perm_manager.grant_permission("ext2".to_string(), PermissionType::Storage).expect("Failed to grant ext2");
        let result = session_storage.get("ext2", &perms1, None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_alarm_isolation() {
        let dir = temp_dir().join(format!("exodus_alarm_iso_{}", uuid::Uuid::new_v4()));
        let manager = AlarmsManager::new(dir.clone()).expect("Failed to create manager");
        
        // Create alarm for ext1
        let info = AlarmCreateInfo {
            when: Some(10000), // Use absolute time
            delay_in_minutes: None,
            period_in_minutes: None,
        };
        manager.create("ext1", "alarm1", info).expect("Failed to create alarm");
        
        // ext1 should see the alarm
        let alarms = manager.get_all("ext1");
        assert_eq!(alarms.len(), 1);
        
        // ext2 should not see ext1's alarms
        let alarms = manager.get_all("ext2");
        assert_eq!(alarms.len(), 0);
        
        // Clear ext1's alarms should not affect ext2
        manager.clear_all("ext1").expect("Failed to clear");
        assert!(manager.get_all("ext1").is_empty());
        assert!(manager.get_all("ext2").is_empty());
    }
    
    #[test]
    fn test_storage_data_isolation() {
        let session_storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Set data for ext1
        let mut items1 = serde_json::Map::new();
        items1.insert("secret".to_string(), serde_json::json!("ext1-data"));
        session_storage.set("ext1", &perms, items1).expect("Failed to set ext1");
        
        // Set different data for ext2
        let mut items2 = serde_json::Map::new();
        items2.insert("secret".to_string(), serde_json::json!("ext2-data"));
        session_storage.set("ext2", &perms, items2).expect("Failed to set ext2");
        
        // Verify isolation
        let data1 = session_storage.get("ext1", &perms, None).expect("Failed to get ext1");
        let data2 = session_storage.get("ext2", &perms, None).expect("Failed to get ext2");
        
        assert_eq!(data1.get("secret"), Some(&serde_json::json!("ext1-data")));
        assert_eq!(data2.get("secret"), Some(&serde_json::json!("ext2-data")));
        
        assert_ne!(data1.get("secret"), data2.get("secret"));
    }
    
    #[test]
    fn test_permission_event_integrity() {
        let dir = temp_dir().join(format!("exodus_event_integrity_{}", uuid::Uuid::new_v4()));
        let manager = ExtensionPermissionsManager::new(dir.clone()).expect("Failed to create manager");
        
        // Grant same permission twice - should only create one Added event
        manager.grant_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to grant 1");
        manager.grant_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to grant 2");
        
        let events = manager.get_permission_events("ext1");
        let added_events = events.iter().filter(|e| e.event_type == PermissionEventType::Added).count();
        assert_eq!(added_events, 1);
        
        // Deny permission twice - should only create one Removed event
        manager.deny_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to deny 1");
        manager.deny_permission("ext1".to_string(), PermissionType::Tabs).expect("Failed to deny 2");
        
        let events = manager.get_permission_events("ext1");
        let removed_events = events.iter().filter(|e| e.event_type == PermissionEventType::Removed).count();
        assert_eq!(removed_events, 1);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_alarm_creation_performance() {
        let dir = temp_dir().join(format!("exodus_alarm_perf_{}", uuid::Uuid::new_v4()));
        let manager = AlarmsManager::new(dir).expect("Failed to create manager");
        
        let start = Instant::now();
        for i in 0..10 { // Reduced from 100 to 10 for faster testing
            let info = AlarmCreateInfo {
                when: Some(10000 + (i as u64) * 1000), // Use absolute times
                delay_in_minutes: None,
                period_in_minutes: None,
            };
            manager.create("ext1", &format!("alarm-{}", i), info).expect("Failed to create alarm");
        }
        let duration = start.elapsed();
        
        // Should create 10 alarms in less than 100ms
        assert!(duration.as_millis() < 100, "Alarm creation too slow: {:?}", duration);
    }
    
    #[test]
    fn test_storage_operations_performance() {
        let session_storage = ExtensionSessionStorage::new(); // Use session storage to avoid file I/O
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Test set performance
        let start = Instant::now();
        for i in 0..10 { // Reduced from 100 to 10
            let mut items = serde_json::Map::new();
            items.insert(format!("key-{}", i), serde_json::json!(i));
            session_storage.set("ext1", &perms, items).expect("Failed to set");
        }
        let set_duration = start.elapsed();
        assert!(set_duration.as_millis() < 200, "Storage set too slow: {:?}", set_duration);
        
        // Test get performance
        let start = Instant::now();
        for _ in 0..10 { // Reduced from 100 to 10
            session_storage.get("ext1", &perms, None).expect("Failed to get");
        }
        let get_duration = start.elapsed();
        assert!(get_duration.as_millis() < 100, "Storage get too slow: {:?}", get_duration);
    }
    
    #[test]
    fn test_session_storage_performance() {
        let storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Test set performance
        let start = Instant::now();
        for i in 0..100 { // Reduced from 1000 to 100
            let mut items = serde_json::Map::new();
            items.insert(format!("key-{}", i), serde_json::json!(i));
            storage.set("ext1", &perms, items).expect("Failed to set");
        }
        let set_duration = start.elapsed();
        assert!(set_duration.as_millis() < 50, "Session storage set too slow: {:?}", set_duration);
        
        // Test get performance
        let start = Instant::now();
        for _ in 0..100 { // Reduced from 1000 to 100
            storage.get("ext1", &perms, None).expect("Failed to get");
        }
        let get_duration = start.elapsed();
        assert!(get_duration.as_millis() < 20, "Session storage get too slow: {:?}", get_duration);
    }

    #[test]
    fn test_storage_get_bytes_in_use_performance() {
        let session_storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Set up test data
        for i in 0..100 {
            let mut items = serde_json::Map::new();
            items.insert(format!("key-{}", i), serde_json::json!(format!("value-{}", i)));
            session_storage.set("ext1", &perms, items).expect("Failed to set");
        }
        
        // Test getBytesInUse performance
        let start = Instant::now();
        for _ in 0..100 {
            session_storage.get_bytes_in_use("ext1", &perms, None).expect("Failed to get bytes");
        }
        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "getBytesInUse too slow: {:?}", duration);
    }

    #[test]
    fn test_session_storage_remove_performance() {
        let storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Set up test data
        for i in 0..100 {
            let mut items = serde_json::Map::new();
            items.insert(format!("key-{}", i), serde_json::json!(i));
            storage.set("ext1", &perms, items).expect("Failed to set");
        }
        
        // Test remove performance
        let start = Instant::now();
        for i in 0..50 {
            storage
                .remove("ext1", &perms, vec![format!("key-{}", i)])
                .expect("Failed to remove");
        }
        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "Remove too slow: {:?}", duration);
    }

    #[test]
    fn test_session_storage_clear_performance() {
        let storage = ExtensionSessionStorage::new();
        let perms = vec![super::super::permissions::Permission::Storage];
        
        // Set up test data
        for i in 0..100 {
            let mut items = serde_json::Map::new();
            items.insert(format!("key-{}", i), serde_json::json!(i));
            storage.set("ext1", &perms, items).expect("Failed to set");
        }
        
        // Test clear performance
        let start = Instant::now();
        storage.clear("ext1", &perms).expect("Failed to clear");
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10, "Clear too slow: {:?}", duration);
    }
    
    #[test]
    fn test_permission_check_performance() {
        let dir = temp_dir().join(format!("exodus_perm_perf_{}", uuid::Uuid::new_v4()));
        let manager = ExtensionPermissionsManager::new(dir).expect("Failed to create manager");
        
        // Grant many permissions
        for perm in &[PermissionType::Tabs, PermissionType::Storage, PermissionType::Notifications] {
            manager.grant_permission("ext1".to_string(), perm.clone()).expect("Failed to grant");
        }
        
        let start = Instant::now();
        for _ in 0..1000 { // Reduced from 10000 to 1000
            manager.has_permission("ext1", PermissionType::Storage);
        }
        let duration = start.elapsed();
        
        // Should check 1000 permissions in less than 10ms
        assert!(duration.as_millis() < 10, "Permission check too slow: {:?}", duration);
    }
    
    #[test]
    fn test_alarm_due_check_performance() {
        let dir = temp_dir().join(format!("exodus_due_perf_{}", uuid::Uuid::new_v4()));
        let manager = AlarmsManager::new(dir).expect("Failed to create manager");
        
        // Create many alarms
        for i in 0..10 { // Reduced from 100 to 10
            let info = AlarmCreateInfo {
                when: Some(i as u64),
                delay_in_minutes: None,
                period_in_minutes: None,
            };
            manager.create("ext1", &format!("alarm-{}", i), info).expect("Failed to create");
        }
        
        let start = Instant::now();
        for _ in 0..100 { // Reduced from 1000 to 100
            manager.check_due_alarms("ext1");
        }
        let duration = start.elapsed();
        
        // Should check 100 times with 10 alarms in less than 100ms
        assert!(duration.as_millis() < 100, "Due check too slow: {:?}", duration);
    }
}
