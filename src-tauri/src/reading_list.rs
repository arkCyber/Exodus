//! Exodus Browser — Reading List functionality
//!
//! Provides a reading list for saving pages to read later with offline support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Reading list item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingListItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub favicon_url: Option<String>,
    pub added_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub archived: bool,
    pub tags: Vec<String>,
}

/// Reading list state
pub struct ReadingListState {
    db: Arc<Db>,
    items: Arc<Mutex<Vec<ReadingListItem>>>,
}

impl ReadingListState {
    /// Create reading list state with sled database
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let db = sled::open(db_path).map_err(|e| format!("Failed to open reading list DB: {}", e))?;
        let items = Self::load_items(&db)?;
        
        Ok(Self {
            db: Arc::new(db),
            items: Arc::new(Mutex::new(items)),
        })
    }

    /// Load items from database
    fn load_items(db: &Db) -> Result<Vec<ReadingListItem>, String> {
        let mut items = Vec::new();
        for item_result in db.iter() {
            let (key, value) = item_result.map_err(|e| format!("DB iteration error: {}", e))?;
            if key.starts_with(b"item:") {
                if let Ok(item) = bincode::deserialize::<ReadingListItem>(&value) {
                    items.push(item);
                }
            }
        }
        items.sort_by(|a, b| b.added_at.cmp(&a.added_at));
        Ok(items)
    }

    /// Add item to reading list
    pub fn add_item(&self, url: String, title: String, excerpt: Option<String>) -> Result<ReadingListItem, String> {
        let id = Uuid::new_v4().to_string();
        let item = ReadingListItem {
            id: id.clone(),
            url,
            title,
            excerpt,
            favicon_url: None,
            added_at: Utc::now(),
            read_at: None,
            archived: false,
            tags: Vec::new(),
        };

        let key = format!("item:{}", id);
        let value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
        
        self.db.insert(key.as_bytes(), value)
            .map_err(|e| format!("DB insert error: {}", e))?;

        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        items.insert(0, item.clone());
        items.sort_by(|a, b| b.added_at.cmp(&a.added_at));
        
        Ok(item)
    }

    /// Remove item from reading list
    pub fn remove_item(&self, id: &str) -> Result<(), String> {
        let key = format!("item:{}", id);
        self.db.remove(key.as_bytes())
            .map_err(|e| format!("DB remove error: {}", e))?;

        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        items.retain(|item| item.id != id);
        
        Ok(())
    }

    /// Mark item as read
    pub fn mark_as_read(&self, id: &str) -> Result<(), String> {
        let key = format!("item:{}", id);
        if let Some(value) = self.db.get(key.as_bytes()).map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(mut item) = bincode::deserialize::<ReadingListItem>(&value) {
                item.read_at = Some(Utc::now());
                let new_value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
                self.db.insert(key.as_bytes(), new_value)
                    .map_err(|e| format!("DB update error: {}", e))?;

                let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
                if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                    existing.read_at = Some(Utc::now());
                }
            }
        }
        Ok(())
    }

    /// Archive item
    pub fn archive_item(&self, id: &str) -> Result<(), String> {
        let key = format!("item:{}", id);
        if let Some(value) = self.db.get(key.as_bytes()).map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(mut item) = bincode::deserialize::<ReadingListItem>(&value) {
                item.archived = true;
                let new_value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
                self.db.insert(key.as_bytes(), new_value)
                    .map_err(|e| format!("DB update error: {}", e))?;

                let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
                if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                    existing.archived = true;
                }
            }
        }
        Ok(())
    }

    /// Unarchive item
    pub fn unarchive_item(&self, id: &str) -> Result<(), String> {
        let key = format!("item:{}", id);
        if let Some(value) = self.db.get(key.as_bytes()).map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(mut item) = bincode::deserialize::<ReadingListItem>(&value) {
                item.archived = false;
                let new_value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
                self.db.insert(key.as_bytes(), new_value)
                    .map_err(|e| format!("DB update error: {}", e))?;

                let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
                if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                    existing.archived = false;
                }
            }
        }
        Ok(())
    }

    /// Add tag to item
    pub fn add_tag(&self, id: &str, tag: String) -> Result<(), String> {
        let key = format!("item:{}", id);
        if let Some(value) = self.db.get(key.as_bytes()).map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(mut item) = bincode::deserialize::<ReadingListItem>(&value) {
                if !item.tags.contains(&tag) {
                    item.tags.push(tag.clone());
                    let new_value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
                    self.db.insert(key.as_bytes(), new_value)
                        .map_err(|e| format!("DB update error: {}", e))?;

                    let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
                    if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                        if !existing.tags.contains(&tag) {
                            existing.tags.push(tag);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Remove tag from item
    pub fn remove_tag(&self, id: &str, tag: &str) -> Result<(), String> {
        let key = format!("item:{}", id);
        if let Some(value) = self.db.get(key.as_bytes()).map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(mut item) = bincode::deserialize::<ReadingListItem>(&value) {
                item.tags.retain(|t| t != tag);
                let new_value = bincode::serialize(&item).map_err(|e| format!("Serialization error: {}", e))?;
                self.db.insert(key.as_bytes(), new_value)
                    .map_err(|e| format!("DB update error: {}", e))?;

                let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
                if let Some(existing) = items.iter_mut().find(|i| i.id == id) {
                    existing.tags.retain(|t| t != tag);
                }
            }
        }
        Ok(())
    }

    /// Get all items
    pub fn get_all(&self) -> Result<Vec<ReadingListItem>, String> {
        let items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(items.clone())
    }

    /// Get unread items
    pub fn get_unread(&self) -> Result<Vec<ReadingListItem>, String> {
        let items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(items.iter().filter(|item| item.read_at.is_none() && !item.archived).cloned().collect())
    }

    /// Get archived items
    pub fn get_archived(&self) -> Result<Vec<ReadingListItem>, String> {
        let items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(items.iter().filter(|item| item.archived).cloned().collect())
    }

    /// Get items by tag
    pub fn get_by_tag(&self, tag: &str) -> Result<Vec<ReadingListItem>, String> {
        let items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(items.iter().filter(|item| item.tags.iter().any(|t| t == tag)).cloned().collect())
    }

    /// Search items
    pub fn search(&self, query: &str) -> Result<Vec<ReadingListItem>, String> {
        let items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        let query_lower = query.to_lowercase();
        Ok(items.iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower) ||
                item.url.to_lowercase().contains(&query_lower) ||
                item.excerpt.as_ref().map_or(false, |e| e.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect())
    }
}
