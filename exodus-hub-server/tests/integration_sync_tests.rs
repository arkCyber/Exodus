//! Integration tests for message synchronization
//!
//! These tests verify the end-to-end sync functionality including
//! API endpoints, compression, and pagination.

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use exodus_hub_server::{ImManager, create_im_router, ImServerState};
    use tempfile::TempDir;
    use tower::ServiceExt;

    async fn create_test_server() -> (axum::Router, std::sync::Arc<ImManager>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_integration.db");
        let manager = ImManager::new(db_path).unwrap();
        let manager_arc = std::sync::Arc::new(manager);
        let state = ImServerState::new(manager_arc.clone());
        let router = create_im_router(state);
        (router, manager_arc, temp_dir)
    }

    #[tokio::test]
    async fn test_sync_api_endpoint() {
        let (router, manager, _temp_dir) = create_test_server().await;

        // Create test data
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat"
        ).await.unwrap();
        let user1 = manager.create_user("user1", "User One").await.unwrap();
        let user2 = manager.create_user("user2", "User Two").await.unwrap();

        for i in 1..=5 {
            manager.send_message(
                &conv.id,
                &user1.id,
                Some(&user2.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Test sync endpoint
        let request = Request::builder()
            .uri(format!("/api/sync/messages?conversation_id={}&limit=10", conv.id))
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let sync_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(sync_response["messages"].as_array().unwrap().len(), 5);
        assert_eq!(sync_response["has_more"], false);
    }

    #[tokio::test]
    async fn test_sync_api_with_pagination() {
        let (router, manager, _temp_dir) = create_test_server().await;

        // Create test data
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 2"
        ).await.unwrap();
        let user1 = manager.create_user("user3", "User Three").await.unwrap();
        let user2 = manager.create_user("user4", "User Four").await.unwrap();

        for i in 1..=15 {
            manager.send_message(
                &conv.id,
                &user1.id,
                Some(&user2.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Test first page
        let request = Request::builder()
            .uri(format!("/api/sync/messages?conversation_id={}&limit=10", conv.id))
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let sync_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(sync_response["messages"].as_array().unwrap().len(), 10);
        assert_eq!(sync_response["has_more"], true);
        assert_eq!(sync_response["last_sequence"], 10);

        // Test second page
        let last_seq = sync_response["last_sequence"].as_u64().unwrap() as u32;
        let request = Request::builder()
            .uri(format!("/api/sync/messages?conversation_id={}&after_sequence={}&limit=10", conv.id, last_seq))
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let sync_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(sync_response["messages"].as_array().unwrap().len(), 5);
        assert_eq!(sync_response["has_more"], false);
    }

    #[tokio::test]
    async fn test_sync_compressed_endpoint() {
        let (router, manager, _temp_dir) = create_test_server().await;

        // Create test data
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 3"
        ).await.unwrap();
        let user1 = manager.create_user("user5", "User Five").await.unwrap();
        let user2 = manager.create_user("user6", "User Six").await.unwrap();

        for i in 1..=10 {
            manager.send_message(
                &conv.id,
                &user1.id,
                Some(&user2.id),
                &format!("Test message {} with content", i),
                None,
            ).await.unwrap();
        }

        // Test compressed endpoint
        let request = Request::builder()
            .uri(format!("/api/sync/compressed?conversation_id={}&limit=100", conv.id))
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "application/octet-stream");
        assert_eq!(headers.get("content-encoding").unwrap(), "zstd");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(!body.is_empty());

        // Decompress and verify
        let decompressed = zstd::decode_all(&body[..]).unwrap();
        let messages: Vec<exodus_hub_server::manager::Message> = bincode::deserialize(&decompressed).unwrap();
        assert_eq!(messages.len(), 10);
    }

    #[tokio::test]
    async fn test_sync_empty_conversation() {
        let (router, manager, _temp_dir) = create_test_server().await;

        // Create empty conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Empty Chat"
        ).await.unwrap();

        // Test sync endpoint
        let request = Request::builder()
            .uri(format!("/api/sync/messages?conversation_id={}", conv.id))
            .body(Body::empty())
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let sync_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(sync_response["messages"].as_array().unwrap().len(), 0);
        assert_eq!(sync_response["has_more"], false);
    }
}
