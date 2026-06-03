//! Tauri commands for News Aggregation Service
//! 
//! These commands allow the frontend to interact with the News Aggregation Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{NewsAggregationService, NewsAggregationServiceConfig, NewsArticle, NewsSource, NewsFeedConfig, NewsStats};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed News Aggregation Service instance
pub struct ManagedNewsAggregationService {
    service: Arc<NewsAggregationService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedNewsAggregationService {
    pub fn new(service: NewsAggregationService) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }
        
        self.service.start().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
}

/// Send JSON-RPC request to News Aggregation Service
async fn send_news_aggregation_request(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let socket_path_str = socket_path.to_string_lossy().to_string();
    let client = tokio::net::UnixStream::connect(&socket_path_str)
        .await
        .map_err(|e| format!("Failed to connect to News Aggregation Service: {}", e))?;

    let (mut reader, mut writer) = client.into_split();
    
    let request_str = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    
    writer.write_all(request_str.as_bytes()).await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut buf = [0u8; 8192];
    let n = reader.read(&mut buf).await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let response_str = String::from_utf8_lossy(&buf[..n]).to_string();
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(error) = response.get("error") {
        return Err(format!("News Aggregation Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the News Aggregation Service
#[tauri::command]
pub async fn news_aggregation_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = NewsAggregationServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = NewsAggregationService::new(config)
        .map_err(|e| format!("Failed to create News Aggregation Service: {}", e))?;
    
    let managed = ManagedNewsAggregationService::new(service);
    managed.start().await?;
    
    let _ = app.emit("news-aggregation-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the News Aggregation Service
#[tauri::command]
pub async fn news_aggregation_service_stop() -> Result<(), String> {
    let config = NewsAggregationServiceConfig::default();
    let service = NewsAggregationService::new(config)
        .map_err(|e| format!("Failed to create News Aggregation Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Add news article
#[tauri::command]
pub async fn news_add_article(article: NewsArticle) -> Result<String, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!(article);
    let result = send_news_aggregation_request(&config.socket_path, "add_article", params).await?;
    
    result.get("article_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid article_id response".to_string())
}

/// Add news source
#[tauri::command]
pub async fn news_add_source(source: NewsSource) -> Result<String, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!(source);
    let result = send_news_aggregation_request(&config.socket_path, "add_source", params).await?;
    
    result.get("source_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid source_id response".to_string())
}

/// Update news source
#[tauri::command]
pub async fn news_update_source(source: NewsSource) -> Result<(), String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!(source);
    send_news_aggregation_request(&config.socket_path, "update_source", params).await?;
    Ok(())
}

/// Remove news source
#[tauri::command]
pub async fn news_remove_source(source_id: String) -> Result<(), String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ "source_id": source_id });
    send_news_aggregation_request(&config.socket_path, "remove_source", params).await?;
    Ok(())
}

/// Get news article
#[tauri::command]
pub async fn news_get_article(article_id: String) -> Result<NewsArticle, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ "article_id": article_id });
    let result = send_news_aggregation_request(&config.socket_path, "get_article", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse article: {}", e))
}

/// Get articles by source
#[tauri::command]
pub async fn news_get_articles_by_source(source_id: String, limit: Option<usize>) -> Result<Vec<NewsArticle>, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ 
        "source_id": source_id,
        "limit": limit
    });
    let result = send_news_aggregation_request(&config.socket_path, "get_articles_by_source", params).await?;
    
    let articles = result.get("articles")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid articles response".to_string())?;
    
    articles.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get articles by category
#[tauri::command]
pub async fn news_get_articles_by_category(category: String, limit: Option<usize>) -> Result<Vec<NewsArticle>, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ 
        "category": category,
        "limit": limit
    });
    let result = send_news_aggregation_request(&config.socket_path, "get_articles_by_category", params).await?;
    
    let articles = result.get("articles")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid articles response".to_string())?;
    
    articles.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Search articles
#[tauri::command]
pub async fn news_search_articles(query: String, limit: Option<usize>) -> Result<Vec<NewsArticle>, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ 
        "query": query,
        "limit": limit
    });
    let result = send_news_aggregation_request(&config.socket_path, "search_articles", params).await?;
    
    let articles = result.get("articles")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid articles response".to_string())?;
    
    articles.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get latest articles
#[tauri::command]
pub async fn news_get_latest_articles(limit: Option<usize>) -> Result<Vec<NewsArticle>, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ "limit": limit });
    let result = send_news_aggregation_request(&config.socket_path, "get_latest_articles", params).await?;
    
    let articles = result.get("articles")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid articles response".to_string())?;
    
    articles.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get all sources
#[tauri::command]
pub async fn news_get_sources() -> Result<Vec<NewsSource>, String> {
    let config = NewsAggregationServiceConfig::default();
    let result = send_news_aggregation_request(&config.socket_path, "get_sources", json!(null)).await?;
    
    let sources = result.get("sources")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid sources response".to_string())?;
    
    sources.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Create news feed
#[tauri::command]
pub async fn news_create_feed(feed: NewsFeedConfig) -> Result<String, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!(feed);
    let result = send_news_aggregation_request(&config.socket_path, "create_feed", params).await?;
    
    result.get("feed_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid feed_id response".to_string())
}

/// Get feed articles
#[tauri::command]
pub async fn news_get_feed_articles(feed_id: String, limit: Option<usize>) -> Result<Vec<NewsArticle>, String> {
    let config = NewsAggregationServiceConfig::default();
    let params = json!({ 
        "feed_id": feed_id,
        "limit": limit
    });
    let result = send_news_aggregation_request(&config.socket_path, "get_feed_articles", params).await?;
    
    let articles = result.get("articles")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid articles response".to_string())?;
    
    articles.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get statistics
#[tauri::command]
pub async fn news_get_statistics() -> Result<NewsStats, String> {
    let config = NewsAggregationServiceConfig::default();
    let result = send_news_aggregation_request(&config.socket_path, "get_statistics", json!(null)).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse stats: {}", e))
}
