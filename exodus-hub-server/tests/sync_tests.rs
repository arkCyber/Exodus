//! Unit tests for message synchronization

#[cfg(test)]
mod tests {
    use exodus_hub_server::manager::ImManager;
    use tempfile::TempDir;

    fn create_test_manager() -> (ImManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_sync.db");
        let manager = ImManager::new(db_path).unwrap();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_get_messages_for_sync() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat"
        ).await.unwrap();

        // Create users
        let user1 = manager.create_user("user1", "User One").await.unwrap();
        let user2 = manager.create_user("user2", "User Two").await.unwrap();

        // Send some messages
        for i in 1..=5 {
            manager.send_message(
                &conv.id,
                &user1.id,
                Some(&user2.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Sync without after_sequence
        let sync_response = manager.get_messages_for_sync(&conv.id, None, 10).await.unwrap();
        assert_eq!(sync_response.messages.len(), 5);
        assert_eq!(sync_response.last_sequence, 5);
        assert!(!sync_response.has_more);
    }

    #[tokio::test]
    async fn test_get_messages_for_sync_with_pagination() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 2"
        ).await.unwrap();

        // Create users
        let user = manager.create_user("user3", "User Three").await.unwrap();
        let user_other = manager.create_user("user_other", "Other User").await.unwrap();

        // Send many messages
        for i in 1..=15 {
            manager.send_message(
                &conv.id,
                &user.id,
                Some(&user_other.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Sync with limit
        let sync_response = manager.get_messages_for_sync(&conv.id, None, 10).await.unwrap();
        assert_eq!(sync_response.messages.len(), 10);
        assert!(sync_response.has_more);
        assert_eq!(sync_response.last_sequence, 10);

        // Sync with after_sequence
        let sync_response2 = manager.get_messages_for_sync(&conv.id, Some(10), 10).await.unwrap();
        assert_eq!(sync_response2.messages.len(), 5);
        assert!(!sync_response2.has_more);
        assert_eq!(sync_response2.last_sequence, 15);
    }

    #[tokio::test]
    async fn test_compress_messages() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 3"
        ).await.unwrap();

        // Create users
        let user = manager.create_user("user4", "User Four").await.unwrap();
        let user_other = manager.create_user("user_other4", "Other User").await.unwrap();

        // Send messages
        for i in 1..=10 {
            manager.send_message(
                &conv.id,
                &user.id,
                Some(&user_other.id),
                &format!("Test message {} with some content", i),
                None,
            ).await.unwrap();
        }

        // Get messages
        let sync_response = manager.get_messages_for_sync(&conv.id, None, 100).await.unwrap();

        // Compress
        let compressed = ImManager::compress_messages(&sync_response.messages).unwrap();
        assert!(!compressed.is_empty());
        assert!(compressed.len() < bincode::serialize(&sync_response.messages).unwrap().len());
    }

    #[tokio::test]
    async fn test_decompress_messages() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 4"
        ).await.unwrap();

        // Create users
        let user = manager.create_user("user5", "User Five").await.unwrap();
        let user_other = manager.create_user("user_other5", "Other User").await.unwrap();

        // Send messages
        for i in 1..=5 {
            manager.send_message(
                &conv.id,
                &user.id,
                Some(&user_other.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Get and compress
        let sync_response = manager.get_messages_for_sync(&conv.id, None, 100).await.unwrap();
        let compressed = ImManager::compress_messages(&sync_response.messages).unwrap();

        // Decompress
        let decompressed = ImManager::decompress_messages(&compressed).unwrap();
        assert_eq!(decompressed.len(), sync_response.messages.len());

        // Verify content
        for (original, decompressed) in sync_response.messages.iter().zip(decompressed.iter()) {
            assert_eq!(original.id, decompressed.id);
            assert_eq!(original.content, decompressed.content);
            assert_eq!(original.sequence, decompressed.sequence);
        }
    }

    #[tokio::test]
    async fn test_get_compressed_messages_for_sync() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 5"
        ).await.unwrap();

        // Create users
        let user = manager.create_user("user6", "User Six").await.unwrap();
        let user_other = manager.create_user("user_other6", "Other User").await.unwrap();

        // Send messages
        for i in 1..=3 {
            manager.send_message(
                &conv.id,
                &user.id,
                Some(&user_other.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Get compressed
        let compressed = manager.get_compressed_messages_for_sync(&conv.id, None, 100).await.unwrap();
        assert!(!compressed.is_empty());

        // Decompress and verify
        let decompressed = ImManager::decompress_messages(&compressed).unwrap();
        assert_eq!(decompressed.len(), 3);
    }

    #[tokio::test]
    async fn test_sync_with_sequence_range() {
        let (manager, _temp_dir) = create_test_manager();

        // Create a conversation
        let conv = manager.create_conversation(
            exodus_hub_server::manager::ChatType::OneOnOne,
            "Test Chat 6"
        ).await.unwrap();

        // Create users
        let user = manager.create_user("user7", "User Seven").await.unwrap();
        let user_other = manager.create_user("user_other7", "Other User").await.unwrap();

        // Send messages
        for i in 1..=20 {
            manager.send_message(
                &conv.id,
                &user.id,
                Some(&user_other.id),
                &format!("Message {}", i),
                None,
            ).await.unwrap();
        }

        // Sync in batches
        let mut all_messages = Vec::new();
        let mut after_sequence = None;

        loop {
            let sync_response = manager.get_messages_for_sync(&conv.id, after_sequence, 5).await.unwrap();
            all_messages.extend(sync_response.messages.clone());
            
            if !sync_response.has_more {
                break;
            }
            
            after_sequence = Some(sync_response.last_sequence);
        }

        assert_eq!(all_messages.len(), 20);
    }
}
