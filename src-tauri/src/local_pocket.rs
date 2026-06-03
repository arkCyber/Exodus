//! Local Pocket (本地文章保存) functionality for Exodus Browser
//! Save articles locally for offline reading, similar to Pocket but with local storage

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tauri::{AppHandle, Emitter};

const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_TITLE_LENGTH: usize = 500;
const MAX_URL_LENGTH: usize = 2048;
const MAX_TAGS_COUNT: usize = 50;
const MAX_TAG_LENGTH: usize = 100;

/// Validate article content size
pub fn validate_content_size(content: &str) -> Result<(), String> {
    if content.len() > MAX_CONTENT_SIZE {
        return Err(format!("Content too large (max {} bytes)", MAX_CONTENT_SIZE));
    }
    Ok(())
}

/// Validate article title
pub fn validate_title(title: &str) -> Result<(), String> {
    if title.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title.len() > MAX_TITLE_LENGTH {
        return Err(format!("Title too long (max {} characters)", MAX_TITLE_LENGTH));
    }
    Ok(())
}

/// Validate article URL
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }
    if url.len() > MAX_URL_LENGTH {
        return Err(format!("URL too long (max {} characters)", MAX_URL_LENGTH));
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }
    Ok(())
}

/// Validate tags
pub fn validate_tags(tags: &[String]) -> Result<(), String> {
    if tags.len() > MAX_TAGS_COUNT {
        return Err(format!("Too many tags (max {})", MAX_TAGS_COUNT));
    }
    for tag in tags {
        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }
        if tag.len() > MAX_TAG_LENGTH {
            return Err(format!("Tag too long (max {} characters)", MAX_TAG_LENGTH));
        }
    }
    Ok(())
}

/// Saved article data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedArticle {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub saved_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub reading_time_minutes: u32,
    pub word_count: u32,
}

/// Request to save an article
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveArticleRequest {
    pub url: String,
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
}

/// Request to update an article
#[derive(Debug, Deserialize)]
pub struct UpdateArticleRequest {
    pub id: String,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
}

/// Request to search articles
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchArticlesRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Local Pocket manager
pub struct LocalPocketManager {
    articles: Arc<tokio::sync::Mutex<Vec<SavedArticle>>>,
    storage_path: PathBuf,
    dirty: Arc<AtomicBool>,
    search_index: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Vec<String>>>>, // term -> article IDs
}

impl LocalPocketManager {
    pub fn new(storage_path: PathBuf) -> Self {
        let manager = Self {
            articles: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            storage_path,
            dirty: Arc::new(AtomicBool::new(false)),
            search_index: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };
        manager
    }

    /// Save an article to local storage
    pub async fn save_article(&self, request: SaveArticleRequest, app: AppHandle) -> Result<SavedArticle, String> {
        // Validate inputs
        validate_url(&request.url)?;
        validate_title(&request.title)?;
        validate_content_size(&request.content)?;
        validate_tags(&request.tags)?;
        
        let word_count = request.content.split_whitespace().count() as u32;
        let reading_time = (word_count / 200).max(1); // Average reading speed: 200 words/min
        
        let excerpt = if request.content.len() > 300 {
            request.content.chars().take(300).collect::<String>() + "..."
        } else {
            request.content.clone()
        };

        let article = SavedArticle {
            id: uuid::Uuid::new_v4().to_string(),
            url: request.url,
            title: request.title,
            content: request.content,
            excerpt,
            author: request.author,
            tags: request.tags,
            saved_at: Utc::now(),
            read_at: None,
            is_favorite: false,
            is_archived: false,
            reading_time_minutes: reading_time,
            word_count,
        };

        let mut articles = self.articles.lock().await;
        articles.push(article.clone());
        self.dirty.store(true, Ordering::SeqCst);
        drop(articles);

        // Update search index
        self.update_search_index(&article).await;

        // Emit event to frontend
        app.emit("exodus-pocket-article-saved", &article)
            .map_err(|e| format!("Failed to emit pocket event: {}", e))?;

        // Persist to disk
        self.persist_articles().await?;

        Ok(article)
    }

    /// Get all saved articles
    pub async fn list_articles(&self) -> Vec<SavedArticle> {
        self.articles.lock().await.clone()
    }

    /// Get a specific article by ID
    pub async fn get_article(&self, id: String) -> Option<SavedArticle> {
        let articles = self.articles.lock().await;
        articles.iter().find(|a| a.id == id).cloned()
    }

    /// Update an article
    pub async fn update_article(&self, request: UpdateArticleRequest, app: AppHandle) -> Result<SavedArticle, String> {
        // Validate inputs if provided
        if let Some(title) = &request.title {
            validate_title(title)?;
        }
        if let Some(tags) = &request.tags {
            validate_tags(tags)?;
        }
        
        let mut articles = self.articles.lock().await;
        
        if let Some(article) = articles.iter_mut().find(|a| a.id == request.id) {
            if let Some(title) = request.title {
                article.title = title;
            }
            if let Some(tags) = request.tags {
                article.tags = tags;
            }
            if let Some(is_favorite) = request.is_favorite {
                article.is_favorite = is_favorite;
            }
            if let Some(is_archived) = request.is_archived {
                article.is_archived = is_archived;
            }
            
            let updated = article.clone();
            self.dirty.store(true, Ordering::SeqCst);
            drop(articles);

            // Emit event to frontend
            app.emit("exodus-pocket-article-updated", &updated)
                .map_err(|e| format!("Failed to emit pocket event: {}", e))?;

            // Persist to disk
            self.persist_articles().await?;

            Ok(updated)
        } else {
            Err("Article not found".to_string())
        }
    }

    /// Mark article as read
    pub async fn mark_as_read(&self, id: String, app: AppHandle) -> Result<(), String> {
        let mut articles = self.articles.lock().await;
        
        if let Some(article) = articles.iter_mut().find(|a| a.id == id) {
            article.read_at = Some(Utc::now());
            self.dirty.store(true, Ordering::SeqCst);
            drop(articles);

            // Emit event to frontend
            app.emit("exodus-pocket-article-read", &id)
                .map_err(|e| format!("Failed to emit pocket event: {}", e))?;

            // Persist to disk
            self.persist_articles().await?;

            Ok(())
        } else {
            Err("Article not found".to_string())
        }
    }

    /// Delete an article
    pub async fn delete_article(&self, id: String, app: AppHandle) -> Result<(), String> {
        let mut articles = self.articles.lock().await;
        let original_len = articles.len();
        articles.retain(|a| a.id != id);
        
        if articles.len() < original_len {
            self.dirty.store(true, Ordering::SeqCst);
            drop(articles);

            // Emit event to frontend
            app.emit("exodus-pocket-article-deleted", &id)
                .map_err(|e| format!("Failed to emit pocket event: {}", e))?;

            // Persist to disk
            self.persist_articles().await?;

            Ok(())
        } else {
            Err("Article not found".to_string())
        }
    }

    /// Search articles by query (using search index for better performance)
    pub async fn search_articles(&self, request: SearchArticlesRequest) -> Vec<SavedArticle> {
        let articles = self.articles.lock().await;
        let index = self.search_index.lock().await;
        let query_lower = request.query.to_lowercase();
        
        // Use search index if available
        let mut matching_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for word in query_lower.split_whitespace() {
            if let Some(ids) = index.get(word) {
                for id in ids {
                    matching_ids.insert(id.clone());
                }
            }
        }
        
        let results: Vec<SavedArticle> = if matching_ids.is_empty() {
            // Fallback to linear search if index doesn't have results
            articles
                .iter()
                .filter(|a| {
                    a.title.to_lowercase().contains(&query_lower) ||
                    a.excerpt.to_lowercase().contains(&query_lower) ||
                    a.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                })
                .cloned()
                .collect()
        } else {
            // Use index results
            articles
                .iter()
                .filter(|a| matching_ids.contains(&a.id))
                .cloned()
                .collect()
        };

        let offset = request.offset.unwrap_or(0);
        let limit = request.limit.unwrap_or(results.len());
        
        results.into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }

    /// Get articles by tag
    pub async fn get_articles_by_tag(&self, tag: String) -> Vec<SavedArticle> {
        let articles = self.articles.lock().await;
        articles
            .iter()
            .filter(|a| a.tags.iter().any(|t| t == &tag))
            .cloned()
            .collect()
    }

    /// Get all tags
    pub async fn get_all_tags(&self) -> Vec<String> {
        let articles = self.articles.lock().await;
        let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for article in articles.iter() {
            for tag in &article.tags {
                tags.insert(tag.clone());
            }
        }
        
        tags.into_iter().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> PocketStats {
        let articles = self.articles.lock().await;
        let total = articles.len();
        let unread = articles.iter().filter(|a| a.read_at.is_none()).count();
        let favorites = articles.iter().filter(|a| a.is_favorite).count();
        let archived = articles.iter().filter(|a| a.is_archived).count();
        let total_words: u32 = articles.iter().map(|a| a.word_count).sum();
        
        PocketStats {
            total_articles: total,
            unread_articles: unread,
            favorite_articles: favorites,
            archived_articles: archived,
            total_word_count: total_words,
            total_reading_time_minutes: total_words / 200,
        }
    }

    /// Persist articles to disk
    async fn persist_articles(&self) -> Result<(), String> {
        let articles = self.articles.lock().await;
        
        // Create storage directory if it doesn't exist
        if let Some(parent) = self.storage_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        let json = serde_json::to_string(&*articles)
            .map_err(|e| format!("Failed to serialize articles: {}", e))?;

        tokio::fs::write(&self.storage_path, json)
            .await
            .map_err(|e| format!("Failed to write articles to disk: {}", e))?;

        Ok(())
    }

    /// Load articles from disk
    pub async fn load_articles(&self) -> Result<(), String> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let json = tokio::fs::read_to_string(&self.storage_path)
            .await
            .map_err(|e| format!("Failed to read articles from disk: {}", e))?;

        let articles: Vec<SavedArticle> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize articles: {}", e))?;

        *self.articles.lock().await = articles;
        self.dirty.store(false, Ordering::SeqCst);

        // Rebuild search index after loading
        self.rebuild_search_index().await;

        Ok(())
    }

    /// Force persist articles to disk (immediate write)
    pub async fn force_persist(&self) -> Result<(), String> {
        self.persist_articles().await
    }

    /// Update search index for an article
    async fn update_search_index(&self, article: &SavedArticle) {
        let mut index = self.search_index.lock().await;
        
        // Index title words
        for word in article.title.split_whitespace() {
            let term = word.to_lowercase();
            index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
        }
        
        // Index excerpt words
        for word in article.excerpt.split_whitespace() {
            let term = word.to_lowercase();
            index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
        }
        
        // Index tags
        for tag in &article.tags {
            let term = tag.to_lowercase();
            index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
        }
    }

    /// Rebuild search index from all articles
    async fn rebuild_search_index(&self) {
        let articles = self.articles.lock().await;
        let mut index = self.search_index.lock().await;
        index.clear();
        
        for article in articles.iter() {
            // Index title words
            for word in article.title.split_whitespace() {
                let term = word.to_lowercase();
                index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
            }
            
            // Index excerpt words
            for word in article.excerpt.split_whitespace() {
                let term = word.to_lowercase();
                index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
            }
            
            // Index tags
            for tag in &article.tags {
                let term = tag.to_lowercase();
                index.entry(term).or_insert_with(Vec::new).push(article.id.clone());
            }
        }
    }
}

/// Pocket statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocketStats {
    pub total_articles: usize,
    pub unread_articles: usize,
    pub favorite_articles: usize,
    pub archived_articles: usize,
    pub total_word_count: u32,
    pub total_reading_time_minutes: u32,
}

// Tauri commands for Local Pocket

/// Save an article to local pocket
#[tauri::command]
pub async fn pocket_save_article(
    request: SaveArticleRequest,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
    app: AppHandle,
) -> Result<SavedArticle, String> {
    pocket_manager.save_article(request, app).await
}

/// List all saved articles
#[tauri::command]
pub async fn pocket_list_articles(
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<Vec<SavedArticle>, String> {
    Ok(pocket_manager.list_articles().await)
}

/// Get a specific article
#[tauri::command]
pub async fn pocket_get_article(
    id: String,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<Option<SavedArticle>, String> {
    Ok(pocket_manager.get_article(id).await)
}

/// Update an article
#[tauri::command]
pub async fn pocket_update_article(
    request: UpdateArticleRequest,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
    app: AppHandle,
) -> Result<SavedArticle, String> {
    pocket_manager.update_article(request, app).await
}

/// Mark article as read
#[tauri::command]
pub async fn pocket_mark_as_read(
    id: String,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
    app: AppHandle,
) -> Result<(), String> {
    pocket_manager.mark_as_read(id, app).await
}

/// Delete an article
#[tauri::command]
pub async fn pocket_delete_article(
    id: String,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
    app: AppHandle,
) -> Result<(), String> {
    pocket_manager.delete_article(id, app).await
}

/// Search articles
#[tauri::command]
pub async fn pocket_search_articles(
    request: SearchArticlesRequest,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<Vec<SavedArticle>, String> {
    Ok(pocket_manager.search_articles(request).await)
}

/// Get articles by tag
#[tauri::command]
pub async fn pocket_get_articles_by_tag(
    tag: String,
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<Vec<SavedArticle>, String> {
    Ok(pocket_manager.get_articles_by_tag(tag).await)
}

/// Get all tags
#[tauri::command]
pub async fn pocket_get_all_tags(
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<Vec<String>, String> {
    Ok(pocket_manager.get_all_tags().await)
}

/// Get pocket statistics
#[tauri::command]
pub async fn pocket_get_stats(
    pocket_manager: tauri::State<'_, Arc<LocalPocketManager>>,
) -> Result<PocketStats, String> {
    Ok(pocket_manager.get_stats().await)
}
