//! Local Pocket (本地文章保存) functionality tests
//! Tests for pocket manager and Tauri commands

#[cfg(test)]
mod tests {
    use super::super::local_pocket::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_pocket_manager_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("articles.json");
        let manager = LocalPocketManager::new(storage_path);
        
        let articles = manager.list_articles().await;
        assert!(articles.is_empty());
    }
    
    #[tokio::test]
    async fn test_article_serialization() {
        let article = SavedArticle {
            id: "test-id".to_string(),
            url: "https://example.com/article".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article content.".to_string(),
            excerpt: "This is a test article...".to_string(),
            author: Some("Test Author".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            saved_at: chrono::Utc::now(),
            read_at: None,
            is_favorite: false,
            is_archived: false,
            reading_time_minutes: 1,
            word_count: 100,
        };
        
        let json = serde_json::to_string(&article).expect("Failed to serialize article");
        let deserialized: SavedArticle = serde_json::from_str(&json).expect("Failed to deserialize article");
        
        assert_eq!(deserialized.id, article.id);
        assert_eq!(deserialized.title, article.title);
        assert_eq!(deserialized.url, article.url);
        assert_eq!(deserialized.tags, article.tags);
    }
    
    #[tokio::test]
    async fn test_request_serialization() {
        let request = SaveArticleRequest {
            url: "https://example.com/article".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article content.".to_string(),
            author: Some("Test Author".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
        };
        
        let json = serde_json::to_string(&request).expect("Failed to serialize request");
        let deserialized: SaveArticleRequest = serde_json::from_str(&json).expect("Failed to deserialize request");
        
        assert_eq!(deserialized.url, request.url);
        assert_eq!(deserialized.title, request.title);
        assert_eq!(deserialized.tags, request.tags);
    }
    
    #[tokio::test]
    async fn test_search_request_serialization() {
        let request = SearchArticlesRequest {
            query: "test".to_string(),
            limit: Some(10),
            offset: Some(0),
        };
        
        let json = serde_json::to_string(&request).expect("Failed to serialize request");
        let deserialized: SearchArticlesRequest = serde_json::from_str(&json).expect("Failed to deserialize request");
        
        assert_eq!(deserialized.query, request.query);
        assert_eq!(deserialized.limit, request.limit);
        assert_eq!(deserialized.offset, request.offset);
    }
    
    #[tokio::test]
    async fn test_pocket_stats_serialization() {
        let stats = PocketStats {
            total_articles: 10,
            unread_articles: 5,
            favorite_articles: 2,
            archived_articles: 1,
            total_word_count: 5000,
            total_reading_time_minutes: 25,
        };
        
        let json = serde_json::to_string(&stats).expect("Failed to serialize stats");
        let deserialized: PocketStats = serde_json::from_str(&json).expect("Failed to deserialize stats");
        
        assert_eq!(deserialized.total_articles, stats.total_articles);
        assert_eq!(deserialized.unread_articles, stats.unread_articles);
        assert_eq!(deserialized.favorite_articles, stats.favorite_articles);
    }
    
    #[tokio::test]
    async fn test_persist_and_load_articles() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("articles.json");
        let manager = Arc::new(LocalPocketManager::new(storage_path.clone()));
        
        // Create a test JSON file directly
        let article = SavedArticle {
            id: "test-id".to_string(),
            url: "https://example.com/article".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article content.".to_string(),
            excerpt: "This is a test article...".to_string(),
            author: Some("Test Author".to_string()),
            tags: vec!["test".to_string()],
            saved_at: chrono::Utc::now(),
            read_at: None,
            is_favorite: false,
            is_archived: false,
            reading_time_minutes: 1,
            word_count: 100,
        };
        
        let articles = vec![article.clone()];
        let json = serde_json::to_string(&articles).expect("Failed to serialize articles");
        tokio::fs::write(&storage_path, json).await.expect("Failed to write articles");
        
        // Create a manager instance to load the articles
        let manager2 = Arc::new(LocalPocketManager::new(storage_path));
        let result = manager2.load_articles().await;
        assert!(result.is_ok());
        
        // Verify the article was loaded
        let loaded_articles = manager2.list_articles().await;
        assert_eq!(loaded_articles.len(), 1);
        assert_eq!(loaded_articles[0].id, article.id);
    }
    
    #[tokio::test]
    async fn test_content_size_validation() {
        // Valid content
        assert!(validate_content_size(&"a".repeat(1000)).is_ok());
        
        // Content too large (10MB limit)
        assert!(validate_content_size(&"a".repeat(11 * 1024 * 1024)).is_err());
    }
    
    #[tokio::test]
    async fn test_title_validation() {
        // Valid titles
        assert!(validate_title("Test Title").is_ok());
        assert!(validate_title(&"A".repeat(500)).is_ok());
        
        // Invalid titles
        assert!(validate_title("").is_err()); // Empty
        assert!(validate_title(&"A".repeat(501)).is_err()); // Too long
    }
    
    #[tokio::test]
    async fn test_pocket_url_validation() {
        // Valid URLs
        assert!(validate_url("https://example.com/article").is_ok());
        assert!(validate_url("http://example.com/article").is_ok());
        
        // Invalid URLs
        assert!(validate_url("").is_err()); // Empty
        assert!(validate_url("not-a-url").is_err()); // Invalid format
        assert!(validate_url("ftp://example.com").is_err()); // Wrong protocol
    }
    
    #[tokio::test]
    async fn test_tags_validation() {
        // Valid tags
        assert!(validate_tags(&vec!["tag1".to_string(), "tag2".to_string()]).is_ok());
        assert!(validate_tags(&vec!["A".repeat(100).to_string()]).is_ok());
        
        // Invalid tags
        assert!(validate_tags(&vec!["".to_string()]).is_err()); // Empty tag
        assert!(validate_tags(&vec!["A".repeat(101).to_string()]).is_err()); // Tag too long
        assert!(validate_tags(&vec!["tag".to_string(); 51]).is_err()); // Too many tags
    }
}
