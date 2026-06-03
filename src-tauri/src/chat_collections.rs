//! Chat Collections — WeChat-style saved messages (收藏) for WebChat sidebar.
//!
//! Persists message snapshots under `{app_data}/webchat/collections.json`.
//! Separate from contact starring (`Contact.is_favorite`).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

const MAX_CONTENT_LEN: usize = 50_000;
const MAX_ITEMS: usize = 10_000;

/// Attachment snapshot stored with a saved chat item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachmentSnapshot {
    pub attachment_id: String,
    pub file_type: String,
    pub blob_hash: String,
    pub file_name: String,
    pub file_size: i64,
    pub thumbnail_hash: Option<String>,
}

/// A message saved to Collections (收藏).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedChatItem {
    pub id: String,
    pub user_id: String,
    pub source_message_id: String,
    pub conversation_id: String,
    pub conversation_type: String,
    pub conversation_title: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content_type: String,
    pub content: String,
    pub message_type: String,
    pub attachments: Vec<MessageAttachmentSnapshot>,
    pub original_timestamp: i64,
    pub saved_at: i64,
}

/// Request payload to save a chat message to Collections.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveChatItemRequest {
    pub user_id: String,
    pub source_message_id: String,
    pub conversation_id: String,
    pub conversation_type: String,
    pub conversation_title: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub attachments: Vec<MessageAttachmentSnapshot>,
    pub original_timestamp: i64,
}

/// In-memory + JSON persistence manager for saved chat items.
pub struct ChatCollectionManager {
    items: Arc<Mutex<Vec<SavedChatItem>>>,
    storage_path: PathBuf,
    loaded: Arc<Mutex<bool>>,
}

impl ChatCollectionManager {
    /// Create a manager backed by the given JSON file path.
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            storage_path,
            loaded: Arc::new(Mutex::new(false)),
        }
    }

    fn ensure_loaded(&self) -> Result<(), String> {
        let mut loaded = self.loaded.lock().map_err(|e| e.to_string())?;
        if *loaded {
            return Ok(());
        }
        if self.storage_path.exists() {
            let data = std::fs::read_to_string(&self.storage_path).map_err(|e| e.to_string())?;
            if !data.trim().is_empty() {
                let parsed: Vec<SavedChatItem> =
                    serde_json::from_str(&data).map_err(|e| format!("Invalid collections JSON: {e}"))?;
                *self.items.lock().map_err(|e| e.to_string())? = parsed;
            }
        }
        *loaded = true;
        Ok(())
    }

    fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let items = self.items.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*items).map_err(|e| e.to_string())?;
        std::fs::write(&self.storage_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Save a message snapshot; rejects duplicates per user + source message id.
    pub fn save(&self, request: SaveChatItemRequest) -> Result<SavedChatItem, String> {
        self.ensure_loaded()?;
        if request.content.len() > MAX_CONTENT_LEN {
            return Err(format!("Content too long (max {MAX_CONTENT_LEN} chars)"));
        }
        let mut items = self.items.lock().map_err(|e| e.to_string())?;
        if items.len() >= MAX_ITEMS {
            return Err("Collection limit reached".to_string());
        }
        if items.iter().any(|i| {
            i.user_id == request.user_id && i.source_message_id == request.source_message_id
        }) {
            return Err("Message already saved to Collections".to_string());
        }

        let content_type = derive_content_type(&request.content, &request.message_type, &request.attachments);
        let item = SavedChatItem {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: request.user_id,
            source_message_id: request.source_message_id,
            conversation_id: request.conversation_id,
            conversation_type: request.conversation_type,
            conversation_title: request.conversation_title,
            sender_id: request.sender_id,
            sender_name: request.sender_name,
            content_type,
            content: request.content,
            message_type: request.message_type,
            attachments: request.attachments,
            original_timestamp: request.original_timestamp,
            saved_at: Utc::now().timestamp_millis(),
        };
        items.push(item.clone());
        drop(items);
        self.persist()?;
        Ok(item)
    }

    /// List saved items for a user, newest first.
    pub fn list(&self, user_id: &str) -> Result<Vec<SavedChatItem>, String> {
        self.ensure_loaded()?;
        let items = self.items.lock().map_err(|e| e.to_string())?;
        let mut user_items: Vec<SavedChatItem> = items
            .iter()
            .filter(|i| i.user_id == user_id)
            .cloned()
            .collect();
        user_items.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
        Ok(user_items)
    }

    /// Search saved items by content, sender, or conversation title.
    pub fn search(&self, user_id: &str, query: &str) -> Result<Vec<SavedChatItem>, String> {
        let q = query.trim().to_lowercase();
        let all = self.list(user_id)?;
        if q.is_empty() {
            return Ok(all);
        }
        Ok(all
            .into_iter()
            .filter(|i| {
                i.content.to_lowercase().contains(&q)
                    || i.sender_name.to_lowercase().contains(&q)
                    || i.conversation_title.to_lowercase().contains(&q)
            })
            .collect())
    }

    /// Delete a saved item; returns true when removed.
    pub fn delete(&self, id: &str, user_id: &str) -> Result<bool, String> {
        self.ensure_loaded()?;
        let mut items = self.items.lock().map_err(|e| e.to_string())?;
        let before = items.len();
        items.retain(|i| !(i.id == id && i.user_id == user_id));
        let deleted = items.len() < before;
        drop(items);
        if deleted {
            self.persist()?;
        }
        Ok(deleted)
    }

    /// Check whether a source message is already saved.
    pub fn is_saved(&self, user_id: &str, source_message_id: &str) -> Result<bool, String> {
        self.ensure_loaded()?;
        let items = self.items.lock().map_err(|e| e.to_string())?;
        Ok(items
            .iter()
            .any(|i| i.user_id == user_id && i.source_message_id == source_message_id))
    }
}

fn derive_content_type(
    content: &str,
    message_type: &str,
    attachments: &[MessageAttachmentSnapshot],
) -> String {
    if !attachments.is_empty() {
        if attachments.iter().any(|a| a.file_type.starts_with("image")) {
            return "image".to_string();
        }
        if attachments.len() > 1 {
            return "mixed".to_string();
        }
        return "file".to_string();
    }
    if message_type == "image" {
        return "image".to_string();
    }
    if message_type == "file" {
        return "file".to_string();
    }
    if content.contains("http://") || content.contains("https://") {
        return "link".to_string();
    }
    "text".to_string()
}

fn get_manager(state: State<'_, Arc<ChatCollectionManager>>) -> Arc<ChatCollectionManager> {
    state.inner().clone()
}

#[tauri::command]
pub async fn chat_collection_save(
    manager: State<'_, Arc<ChatCollectionManager>>,
    request: SaveChatItemRequest,
) -> Result<SavedChatItem, String> {
    get_manager(manager).save(request)
}

#[tauri::command]
pub async fn chat_collection_list(
    manager: State<'_, Arc<ChatCollectionManager>>,
    user_id: String,
) -> Result<Vec<SavedChatItem>, String> {
    get_manager(manager).list(&user_id)
}

#[tauri::command]
pub async fn chat_collection_search(
    manager: State<'_, Arc<ChatCollectionManager>>,
    user_id: String,
    query: String,
) -> Result<Vec<SavedChatItem>, String> {
    get_manager(manager).search(&user_id, &query)
}

#[tauri::command]
pub async fn chat_collection_delete(
    manager: State<'_, Arc<ChatCollectionManager>>,
    id: String,
    user_id: String,
) -> Result<bool, String> {
    get_manager(manager).delete(&id, &user_id)
}

#[tauri::command]
pub async fn chat_collection_is_saved(
    manager: State<'_, Arc<ChatCollectionManager>>,
    user_id: String,
    source_message_id: String,
) -> Result<bool, String> {
    get_manager(manager).is_saved(&user_id, &source_message_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saved_chat_item_creation() {
        let item = SavedChatItem {
            id: "test-id".to_string(),
            user_id: "user-1".to_string(),
            source_message_id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            conversation_type: "group".to_string(),
            conversation_title: "Test Group".to_string(),
            sender_id: "sender-1".to_string(),
            sender_name: "Alice".to_string(),
            content_type: "text".to_string(),
            content: "Hello world".to_string(),
            message_type: "message".to_string(),
            attachments: vec![],
            original_timestamp: 1234567890,
            saved_at: 1234567890,
        };

        assert_eq!(item.id, "test-id");
        assert_eq!(item.content, "Hello world");
        assert!(item.attachments.is_empty());
    }

    #[test]
    fn test_message_attachment_snapshot_creation() {
        let attachment = MessageAttachmentSnapshot {
            attachment_id: "att-1".to_string(),
            file_type: "image".to_string(),
            blob_hash: "hash123".to_string(),
            file_name: "photo.jpg".to_string(),
            file_size: 1024,
            thumbnail_hash: Some("thumb123".to_string()),
        };

        assert_eq!(attachment.attachment_id, "att-1");
        assert_eq!(attachment.file_size, 1024);
        assert!(attachment.thumbnail_hash.is_some());
    }

    #[test]
    fn test_save_chat_item_request_creation() {
        let request = SaveChatItemRequest {
            user_id: "user-1".to_string(),
            source_message_id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            conversation_type: "group".to_string(),
            conversation_title: "Test".to_string(),
            sender_id: "sender-1".to_string(),
            sender_name: "Alice".to_string(),
            content: "Test content".to_string(),
            message_type: "message".to_string(),
            attachments: vec![],
            original_timestamp: 1234567890,
        };

        assert_eq!(request.user_id, "user-1");
        assert!(request.attachments.is_empty());
    }

    #[test]
    fn test_saved_chat_item_serialization() {
        let item = SavedChatItem {
            id: "test-id".to_string(),
            user_id: "user-1".to_string(),
            source_message_id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            conversation_type: "group".to_string(),
            conversation_title: "Test Group".to_string(),
            sender_id: "sender-1".to_string(),
            sender_name: "Alice".to_string(),
            content_type: "text".to_string(),
            content: "Hello world".to_string(),
            message_type: "message".to_string(),
            attachments: vec![],
            original_timestamp: 1234567890,
            saved_at: 1234567890,
        };

        let json = serde_json::to_string(&item).expect("serialize");
        assert!(json.contains("test-id"));
        assert!(json.contains("Hello world"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CONTENT_LEN, 50_000);
        assert_eq!(MAX_ITEMS, 10_000);
    }

    #[test]
    fn test_saved_chat_item_with_attachments() {
        let attachment = MessageAttachmentSnapshot {
            attachment_id: "att-1".to_string(),
            file_type: "image".to_string(),
            blob_hash: "hash123".to_string(),
            file_name: "photo.jpg".to_string(),
            file_size: 1024,
            thumbnail_hash: Some("thumb123".to_string()),
        };

        let item = SavedChatItem {
            id: "test-id".to_string(),
            user_id: "user-1".to_string(),
            source_message_id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            conversation_type: "group".to_string(),
            conversation_title: "Test Group".to_string(),
            sender_id: "sender-1".to_string(),
            sender_name: "Alice".to_string(),
            content_type: "text".to_string(),
            content: "Hello world".to_string(),
            message_type: "message".to_string(),
            attachments: vec![attachment],
            original_timestamp: 1234567890,
            saved_at: 1234567890,
        };

        assert_eq!(item.attachments.len(), 1);
    }
}
