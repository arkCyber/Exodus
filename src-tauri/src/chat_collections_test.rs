//! Unit tests for chat collections (WeChat-style 收藏).

#[cfg(test)]
mod tests {
    use super::super::chat_collections::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn sample_request(user_id: &str, message_id: &str) -> SaveChatItemRequest {
        SaveChatItemRequest {
            user_id: user_id.to_string(),
            source_message_id: message_id.to_string(),
            conversation_id: "room-1".to_string(),
            conversation_type: "dm".to_string(),
            conversation_title: "Alice".to_string(),
            sender_id: "sender-1".to_string(),
            sender_name: "Alice".to_string(),
            content: "Hello https://example.com".to_string(),
            message_type: "text".to_string(),
            attachments: vec![],
            original_timestamp: 1_700_000_000_000,
        }
    }

    #[test]
    fn test_save_list_and_search() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("collections.json");
        let manager = ChatCollectionManager::new(path);

        let saved = manager
            .save(sample_request("user-1", "msg-1"))
            .expect("save");
        assert_eq!(saved.content_type, "link");

        let list = manager.list("user-1").expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].source_message_id, "msg-1");

        let hits = manager.search("user-1", "example").expect("search");
        assert_eq!(hits.len(), 1);

        let dup = manager.save(sample_request("user-1", "msg-1"));
        assert!(dup.is_err());
    }

    #[test]
    fn test_delete_and_is_saved() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("collections.json");
        let manager = ChatCollectionManager::new(path);

        let saved = manager
            .save(sample_request("user-1", "msg-2"))
            .expect("save");
        assert!(manager.is_saved("user-1", "msg-2").expect("is_saved"));

        let deleted = manager.delete(&saved.id, "user-1").expect("delete");
        assert!(deleted);
        assert!(!manager.is_saved("user-1", "msg-2").expect("is_saved"));
    }

    #[test]
    fn test_persistence_reload() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("collections.json");

        {
            let manager = ChatCollectionManager::new(path.clone());
            manager
                .save(sample_request("user-1", "msg-3"))
                .expect("save");
        }

        let reloaded = ChatCollectionManager::new(path);
        let list = reloaded.list("user-1").expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].source_message_id, "msg-3");
    }

    #[test]
    fn test_image_content_type() {
        let temp = TempDir::new().expect("temp dir");
        let manager = ChatCollectionManager::new(temp.path().join("collections.json"));
        let mut req = sample_request("user-1", "img-1");
        req.message_type = "image".to_string();
        req.content = String::new();
        req.attachments.push(MessageAttachmentSnapshot {
            attachment_id: "a1".to_string(),
            file_type: "image/png".to_string(),
            blob_hash: "hash".to_string(),
            file_name: "photo.png".to_string(),
            file_size: 1024,
            thumbnail_hash: None,
        });
        let saved = manager.save(req).expect("save");
        assert_eq!(saved.content_type, "image");
    }
}
