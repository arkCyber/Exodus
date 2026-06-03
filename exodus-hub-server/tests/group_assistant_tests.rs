//! Unit tests for group_assistant module

#[cfg(test)]
mod tests {
    use exodus_hub_server::group_assistant::{GroupAssistantService, GroupAssistantConfig, AssistantMessage};
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;
    use tempfile::TempDir;

    fn create_test_service() -> (GroupAssistantService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_group_assistant.db");
        let config = GroupAssistantConfig { db_path };
        let service = GroupAssistantService::new(config).unwrap();
        (service, temp_dir)
    }

    #[test]
    fn test_create_assistant() {
        let (mut service, _temp_dir) = create_test_service();
        let assistant = service.create_or_get_assistant("test_group_1", "Test Assistant").unwrap();

        assert_eq!(assistant.group_id, "test_group_1");
        assert_eq!(assistant.name, "Test Assistant");
        assert!(!assistant.assistant_id.is_empty());
    }

    #[test]
    fn test_get_existing_assistant() {
        let (mut service, _temp_dir) = create_test_service();
        let assistant1 = service.create_or_get_assistant("test_group_2", "Test Assistant").unwrap();
        let assistant2 = service.create_or_get_assistant("test_group_2", "Test Assistant").unwrap();

        assert_eq!(assistant1.assistant_id, assistant2.assistant_id);
        assert_eq!(assistant1.group_id, assistant2.group_id);
    }

    #[test]
    fn test_store_message() {
        let (mut service, _temp_dir) = create_test_service();
        service.create_or_get_assistant("test_group_3", "Test Assistant").unwrap();

        let message = AssistantMessage {
            message_id: Uuid::new_v4().to_string(),
            group_id: "test_group_3".to_string(),
            sender_id: "user_1".to_string(),
            content: "Hello, World!".to_string(),
            message_type: "text".to_string(),
            sequence: 1,
            timestamp: Utc::now(),
            integrity_hash: "hash123".to_string(),
        };

        service.store_message(message.clone()).unwrap();

        let messages = service.get_messages("test_group_3", 1, 1).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, World!");
    }

    #[test]
    fn test_track_sequence() {
        let (mut service, _temp_dir) = create_test_service();
        service.track_sequence("test_group_4", 5).unwrap();

        let seq = service.get_last_sequence("test_group_4").unwrap();
        assert_eq!(seq, Some(5));
    }

    #[test]
    fn test_record_receipt() {
        let (mut service, _temp_dir) = create_test_service();
        let receipt_id = Uuid::new_v4().to_string();
        let message_id = Uuid::new_v4().to_string();

        service.record_receipt(&receipt_id, &message_id, "user_2", 1).unwrap();

        let receipts = service.get_receipts(&message_id).unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].0, "user_2");
    }

    #[test]
    fn test_get_messages_range() {
        let (mut service, _temp_dir) = create_test_service();
        service.create_or_get_assistant("test_group_5", "Test Assistant").unwrap();

        for i in 1..=5 {
            let message = AssistantMessage {
                message_id: Uuid::new_v4().to_string(),
                group_id: "test_group_5".to_string(),
                sender_id: "user_1".to_string(),
                content: format!("Message {}", i),
                message_type: "text".to_string(),
                sequence: i,
                timestamp: Utc::now(),
                integrity_hash: format!("hash{}", i),
            };
            service.store_message(message).unwrap();
        }

        let messages = service.get_messages("test_group_5", 2, 4).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].sequence, 2);
        assert_eq!(messages[2].sequence, 4);
    }

    #[test]
    fn test_update_activity() {
        let (mut service, _temp_dir) = create_test_service();
        let assistant = service.create_or_get_assistant("test_group_6", "Test Assistant").unwrap();
        let original_last_active = assistant.last_active;

        std::thread::sleep(std::time::Duration::from_millis(10));
        service.update_activity("test_group_6").unwrap();

        let updated_assistant = service.create_or_get_assistant("test_group_6", "Test Assistant").unwrap();
        assert!(updated_assistant.last_active > original_last_active);
    }
}
