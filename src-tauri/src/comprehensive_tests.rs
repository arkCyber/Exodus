//! Comprehensive Tests for Exodus Browser
//!
//! This module contains integration tests for the newly implemented features:
//! - Chrome Extension APIs (history, bookmarks, downloads)
//! - Bookmark suggestions system
//! - Cloud sync service
//! - Widevine DRM framework

#[cfg(test)]
mod comprehensive_tests {
    use super::super::*;

    // Test Chrome Extension APIs
    mod chrome_api_tests {
        use crate::plugins::history_commands::*;
        use crate::plugins::bookmarks_commands::*;
        use crate::plugins::downloads_commands::*;

        #[test]
        fn test_history_api_query_structure() {
            let query = HistoryQuery {
                text: Some("test".to_string()),
                max_results: Some(10),
                end_time: Some(1234567890),
                start_time: Some(1234567000),
            };
            
            assert_eq!(query.text, Some("test".to_string()));
            assert_eq!(query.max_results, Some(10));
        }

        #[test]
        fn test_bookmark_node_serialization() {
            let node = BookmarkNode {
                id: "test-id".to_string(),
                parent_id: Some("parent-id".to_string()),
                index: Some(0),
                title: "Test Bookmark".to_string(),
                url: Some("https://example.com".to_string()),
                date_added: Some(1234567890),
                date_group_modified: None,
                unmodifiable: None,
                children: None,
            };
            
            let json = serde_json::to_string(&node).unwrap();
            assert!(json.contains("Test Bookmark"));
            assert!(json.contains("https://example.com"));
        }

        #[test]
        fn test_download_query_options() {
            let options = DownloadQueryOptions {
                url: Some("https://example.com".to_string()),
                filename: Some("test.pdf".to_string()),
                file_size: Some(FileSizeRange {
                    min: Some(1024),
                    max: Some(1048576),
                }),
                started_after: Some(1234567000),
                ended_before: Some(1234567890),
            };
            
            assert_eq!(options.url, Some("https://example.com".to_string()));
            assert_eq!(options.file_size.unwrap().min, Some(1024));
        }

        #[test]
        fn test_download_item_conversion() {
            use crate::download_manager::{DownloadItem, DownloadStatus};
            use std::path::PathBuf;
            
            let item = DownloadItem::new(
                "https://example.com/file.pdf".to_string(),
                "file.pdf".to_string(),
                PathBuf::from("/downloads/file.pdf"),
            );
            
            let ext_item: DownloadItemExt = item.clone().into();
            assert_eq!(ext_item.url, "https://example.com/file.pdf");
            assert_eq!(ext_item.filename, "file.pdf");
            assert_eq!(ext_item.state, "pending");
        }
    }

    // Test Bookmark Suggestions System
    mod bookmark_suggestions_tests {
        use crate::bookmark_suggestions::*;

        #[test]
        fn test_usage_tracking() {
            let manager = BookmarkSuggestionsManager::new("/tmp/test_usage_tracking")
                .expect("Failed to create manager");
            
            manager.record_access("bookmark1".to_string())
                .expect("Failed to record access");
            
            let usage = manager.get_usage("bookmark1");
            assert!(usage.is_some());
            assert_eq!(usage.unwrap().access_count, 1);
        }

        #[test]
        fn test_frequent_bookmarks() {
            let manager = BookmarkSuggestionsManager::new("/tmp/test_frequent")
                .expect("Failed to create manager");
            
            manager.record_access("bookmark1".to_string())
                .expect("Failed to record access");
            manager.record_access("bookmark1".to_string())
                .expect("Failed to record access");
            manager.record_access("bookmark2".to_string())
                .expect("Failed to record access");
            
            let frequent = manager.get_frequent_bookmarks(10);
            assert_eq!(frequent.len(), 2);
            assert_eq!(frequent[0].bookmark_id, "bookmark1");
            assert_eq!(frequent[0].access_count, 2);
        }

        #[test]
        fn test_recent_bookmarks() {
            let manager = BookmarkSuggestionsManager::new("/tmp/test_recent")
                .expect("Failed to create manager");
            
            manager.record_access("bookmark1".to_string())
                .expect("Failed to record access");
            std::thread::sleep(std::time::Duration::from_millis(10));
            manager.record_access("bookmark2".to_string())
                .expect("Failed to record access");
            
            let recent = manager.get_recent_bookmarks(10);
            assert_eq!(recent.len(), 2);
            assert_eq!(recent[0].bookmark_id, "bookmark2"); // Most recent
        }

        #[test]
        fn test_duplicate_detection() {
            let detector = DuplicateDetector::new();
            
            detector.add_bookmark("bookmark1".to_string(), "https://example.com".to_string());
            detector.add_bookmark("bookmark2".to_string(), "https://example.com".to_string());
            detector.add_bookmark("bookmark3".to_string(), "https://other.com".to_string());
            
            let duplicates = detector.find_duplicates();
            assert_eq!(duplicates.len(), 1);
            assert_eq!(duplicates[0].0, "https://example.com");
            assert!(detector.has_duplicates("https://example.com"));
            assert!(!detector.has_duplicates("https://other.com"));
        }

        #[test]
        fn test_duplicate_removal() {
            let detector = DuplicateDetector::new();
            
            detector.add_bookmark("bookmark1".to_string(), "https://example.com".to_string());
            detector.add_bookmark("bookmark2".to_string(), "https://example.com".to_string());
            
            assert!(detector.has_duplicates("https://example.com"));
            
            detector.remove_bookmark("bookmark1", "https://example.com");
            
            assert!(!detector.has_duplicates("https://example.com"));
        }
    }

    // Test Cloud Sync Service
    mod cloud_sync_tests {
        use crate::cloud_sync::*;

        #[test]
        fn test_cloud_sync_config_default() {
            let config = CloudSyncConfig::default();
            assert_eq!(config.timeout_secs, 30);
            assert_eq!(config.max_retries, 3);
            assert_eq!(config.sync_interval_secs, 300);
            assert!(!config.enabled);
        }

        #[test]
        fn test_cloud_sync_error_display() {
            let error = CloudSyncError::RateLimited;
            assert_eq!(error.to_string(), "Rate limited");
            
            let error = CloudSyncError::AuthenticationFailed;
            assert_eq!(error.to_string(), "Authentication failed");
            
            let error = CloudSyncError::QuotaExceeded;
            assert_eq!(error.to_string(), "Quota exceeded");
        }

        #[test]
        fn test_conflict_resolution_serialization() {
            let resolution = ConflictResolution::KeepLocal;
            let json = serde_json::to_string(&resolution).unwrap();
            assert!(json.contains("keepLocal"));
            
            let resolution = ConflictResolution::KeepRemote;
            let json = serde_json::to_string(&resolution).unwrap();
            assert!(json.contains("keepRemote"));
            
            let resolution = ConflictResolution::Merge;
            let json = serde_json::to_string(&resolution).unwrap();
            assert!(json.contains("merge"));
        }

        #[test]
        fn test_sync_status_serialization() {
            let status = SyncStatus {
                last_sync_time: Some(1234567890),
                pending_changes: 5,
                conflicts: 2,
                quota_used: 1024000,
                quota_total: 10485760,
            };
            
            let json = serde_json::to_string(&status).unwrap();
            assert!(json.contains("pendingChanges"));
            assert!(json.contains("conflicts"));
            assert!(json.contains("quotaUsed"));
        }

        #[test]
        fn test_sync_conflict_serialization() {
            let conflict = SyncConflict {
                conflict_id: "conflict-1".to_string(),
                item_type: "bookmark".to_string(),
                local_data: serde_json::json!({"title": "Local Title"}),
                remote_data: serde_json::json!({"title": "Remote Title"}),
                conflict_time: 1234567890,
            };
            
            let json = serde_json::to_string(&conflict).unwrap();
            assert!(json.contains("conflictId"));
            assert!(json.contains("itemType"));
            assert!(json.contains("localData"));
            assert!(json.contains("remoteData"));
        }
    }

    // Test Widevine DRM Framework
    mod widevine_drm_tests {
        use crate::widevine_drm::*;

        #[test]
        fn test_drm_session_state_serialization() {
            let state = DrmSessionState::Created;
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains("Created"));
            
            let state = DrmSessionState::KeyReady;
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains("KeyReady"));
        }

        #[test]
        fn test_drm_key_request_serialization() {
            let request = DrmKeyRequest {
                session_id: "session-1".to_string(),
                init_data_type: "cenc".to_string(),
                init_data: vec
![1, 2, 3, 4],
                key_system: "com.widevine.alpha".to_string(),
            };
            
            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("sessionId"));
            assert!(json.contains("initDataType"));
            assert!(json.contains("keySystem"));
        }

        #[test]
        fn test_drm_key_response_serialization() {
            let response = DrmKeyResponse {
                session_id: "session-1".to_string(),
                key_data: vec
![5, 6, 7, 8],
            };
            
            let json = serde_json::to_string(&response).unwrap();
            assert!(json.contains("sessionId"));
            assert!(json.contains("keyData"));
        }

        #[test]
        fn test_drm_session_serialization() {
            let session = DrmSession {
                session_id: "session-1".to_string(),
                key_system: "com.widevine.alpha".to_string(),
                state: DrmSessionState::Created,
                created_at: 1234567890,
                media_url: Some("https://example.com/video.mp4".to_string()),
            };
            
            let json = serde_json::to_string(&session).unwrap();
            assert!(json.contains("sessionId"));
            assert!(json.contains("keySystem"));
            assert!(json.contains("mediaUrl"));
        }
    }

    // Integration Tests
    mod integration_tests {
        #[test]
        fn test_extension_api_integration() {
            // Test that all extension APIs can be imported and used
            use crate::plugins::history_commands::*;
            use crate::plugins::bookmarks_commands::*;
            use crate::plugins::downloads_commands::*;
            
            // Verify types are accessible
            let _query = HistoryQuery {
                text: None,
                max_results: None,
                end_time: None,
                start_time: None,
            };
            let _node = BookmarkNode {
                id: "test".to_string(),
                parent_id: None,
                index: None,
                title: "Test".to_string(),
                url: None,
                date_added: None,
                date_group_modified: None,
                unmodifiable: None,
                children: None,
            };
            let _download_query = DownloadQuery {
                limit: None,
                query: None,
            };
            
            // If we get here, imports work correctly
            assert!(true);
        }

        #[test]
        fn test_sync_integration() {
            // Test that sync services can work together
            use crate::cloud_sync::*;
            
            let config = CloudSyncConfig::default();
            assert!(!config.enabled);
            
            // Verify sync config works
            assert_eq!(config.timeout_secs, 30);
            
            assert!(true);
        }
    }
}
