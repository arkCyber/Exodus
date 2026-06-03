//! News Aggregation Service
//! 
//! This service provides news aggregation capabilities from various sources,
//! supporting RSS feeds, custom sources, and personalized news feeds.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// News article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub article_id: String,
    pub source_id: String,
    pub source_name: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub author: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub published_at: u64,
    pub category: String,
    pub tags: Vec<String>,
    pub language: String,
}

/// News source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    pub source_id: String,
    pub name: String,
    pub url: String,
    pub source_type: String, // "rss", "api", "custom"
    pub category: String,
    pub language: String,
    pub update_interval: u64, // seconds
    pub last_updated: u64,
    pub is_active: bool,
    pub icon_url: Option<String>,
}

/// News feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFeedConfig {
    pub feed_id: String,
    pub name: String,
    pub source_ids: Vec<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub language: String,
    pub max_articles: usize,
}

/// News statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsStats {
    pub total_articles: u32,
    pub total_sources: u32,
    pub articles_last_24h: u32,
    pub most_active_source: String,
}

/// Configuration for News Aggregation Service
#[derive(Debug, Clone)]
pub struct NewsAggregationServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for NewsAggregationServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_news_aggregation.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_news");
        
        Self { socket_path, storage_dir }
    }
}

/// News Aggregation Service
pub struct NewsAggregationService {
    config: NewsAggregationServiceConfig,
    articles: Arc<Mutex<HashMap<String, NewsArticle>>>, // article_id -> article
    sources: Arc<Mutex<HashMap<String, NewsSource>>>, // source_id -> source
    feeds: Arc<Mutex<HashMap<String, NewsFeedConfig>>>, // feed_id -> feed
    article_index: Arc<Mutex<HashMap<String, Vec<String>>>>, // source_id -> article_ids
    category_index: Arc<Mutex<HashMap<String, Vec<String>>>>, // category -> article_ids
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl NewsAggregationService {
    pub fn new(config: NewsAggregationServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            articles: Arc::new(Mutex::new(HashMap::new())),
            sources: Arc::new(Mutex::new(HashMap::new())),
            feeds: Arc::new(Mutex::new(HashMap::new())),
            article_index: Arc::new(Mutex::new(HashMap::new())),
            category_index: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }

        let socket_path = self.config.socket_path.clone();
        let articles = Arc::clone(&self.articles);
        let sources = Arc::clone(&self.sources);
        let feeds = Arc::clone(&self.feeds);
        let article_index = Arc::clone(&self.article_index);
        let category_index = Arc::clone(&self.category_index);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let articles = Arc::clone(&articles);
                                let sources = Arc::clone(&sources);
                                let feeds = Arc::clone(&feeds);
                                let article_index = Arc::clone(&article_index);
                                let category_index = Arc::clone(&category_index);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, articles, sources, feeds, article_index, category_index, node_id).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        println!("News Aggregation Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }

        if let Some(tx) = self.shutdown_tx.lock().ok().and_then(|mut tx| tx.take()) {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("News Aggregation Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Add news article
    #[allow(dead_code)]
    pub fn add_article(&self, article: NewsArticle) -> Result<(), String> {
        let article_id = article.article_id.clone();
        let source_id = article.source_id.clone();
        let category = article.category.clone();
        
        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        articles.insert(article_id.clone(), article.clone());
        drop(articles);

        let mut article_index = self.article_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        article_index.entry(source_id).or_insert_with(Vec::new).push(article_id.clone());
        drop(article_index);

        let mut category_index = self.category_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        category_index.entry(category).or_insert_with(Vec::new).push(article_id.clone());

        Ok(())
    }

    /// Add news source
    #[allow(dead_code)]
    pub fn add_source(&self, source: NewsSource) -> Result<(), String> {
        let source_id = source.source_id.clone();
        let mut sources = self.sources.lock().map_err(|e| format!("Lock error: {}", e))?;
        sources.insert(source_id, source);
        Ok(())
    }

    /// Update source
    #[allow(dead_code)]
    pub fn update_source(&self, source: NewsSource) -> Result<(), String> {
        let source_id = source.source_id.clone();
        let mut sources = self.sources.lock().map_err(|e| format!("Lock error: {}", e))?;
        sources.insert(source_id, source);
        Ok(())
    }

    /// Remove source
    #[allow(dead_code)]
    pub fn remove_source(&self, source_id: String) -> Result<(), String> {
        let mut sources = self.sources.lock().map_err(|e| format!("Lock error: {}", e))?;
        sources.remove(&source_id);
        Ok(())
    }

    /// Get article
    #[allow(dead_code)]
    pub fn get_article(&self, article_id: String) -> Option<NewsArticle> {
        let articles = self.articles.lock().ok()?;
        articles.get(&article_id).cloned()
    }

    /// Get articles by source
    #[allow(dead_code)]
    pub fn get_articles_by_source(&self, source_id: String, limit: Option<usize>) -> Vec<NewsArticle> {
        let article_index = self.article_index.lock();
        let articles = self.articles.lock();
        
        if let (Ok(article_index), Ok(articles)) = (article_index, articles) {
            if let Some(article_ids) = article_index.get(&source_id) {
                let mut article_list: Vec<NewsArticle> = article_ids.iter()
                    .filter_map(|id| articles.get(id).cloned())
                    .collect();
                article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
                
                if let Some(limit) = limit {
                    article_list.truncate(limit);
                }
                
                article_list
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get articles by category
    #[allow(dead_code)]
    pub fn get_articles_by_category(&self, category: String, limit: Option<usize>) -> Vec<NewsArticle> {
        let category_index = self.category_index.lock();
        let articles = self.articles.lock();
        
        if let (Ok(category_index), Ok(articles)) = (category_index, articles) {
            if let Some(article_ids) = category_index.get(&category) {
                let mut article_list: Vec<NewsArticle> = article_ids.iter()
                    .filter_map(|id| articles.get(id).cloned())
                    .collect();
                article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
                
                if let Some(limit) = limit {
                    article_list.truncate(limit);
                }
                
                article_list
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Search articles
    #[allow(dead_code)]
    pub fn search_articles(&self, query: String, limit: Option<usize>) -> Vec<NewsArticle> {
        let articles = self.articles.lock();
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<NewsArticle> = articles.as_ref()
            .ok()
            .map(|articles| articles.values()
            .filter(|a| {
                a.title.to_lowercase().contains(&query_lower) ||
                a.summary.to_lowercase().contains(&query_lower) ||
                a.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect())
            .unwrap_or_default();

        results.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }

    /// Get latest articles
    #[allow(dead_code)]
    pub fn get_latest_articles(&self, limit: Option<usize>) -> Vec<NewsArticle> {
        let articles = self.articles.lock();
        
        let mut latest: Vec<NewsArticle> = articles.as_ref()
            .ok()
            .map(|articles| articles.values().cloned().collect())
            .unwrap_or_default();
        latest.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        if let Some(limit) = limit {
            latest.truncate(limit);
        }

        latest
    }

    /// Get all sources
    #[allow(dead_code)]
    pub fn get_sources(&self) -> Vec<NewsSource> {
        self.sources.lock()
            .map(|sources| sources.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Create feed
    #[allow(dead_code)]
    pub fn create_feed(&self, feed: NewsFeedConfig) -> Result<(), String> {
        let feed_id = feed.feed_id.clone();
        let mut feeds = self.feeds.lock().map_err(|e| format!("Lock error: {}", e))?;
        feeds.insert(feed_id, feed);
        Ok(())
    }

    /// Get feed articles
    #[allow(dead_code)]
    pub fn get_feed_articles(&self, feed_id: String, limit: Option<usize>) -> Vec<NewsArticle> {
        let feeds = self.feeds.lock();
        let articles = self.articles.lock();
        
        if let (Ok(feeds), Ok(articles)) = (feeds, articles) {
            if let Some(feed) = feeds.get(&feed_id) {
                let mut article_ids: Vec<String> = Vec::new();
                
                for source_id in &feed.source_ids {
                    let article_index = self.article_index.lock();
                    if let Ok(article_index) = article_index {
                        if let Some(ids) = article_index.get(source_id) {
                            article_ids.extend(ids.clone());
                        }
                    }
                }
            
            let mut article_list: Vec<NewsArticle> = article_ids.iter()
                .filter_map(|id| articles.get(id).cloned())
                .collect();
            
            // Filter by categories
            if !feed.categories.is_empty() {
                article_list.retain(|a| feed.categories.contains(&a.category));
            }
            
            // Filter by keywords
            if !feed.keywords.is_empty() {
                article_list.retain(|a| {
                    let title_lower = a.title.to_lowercase();
                    feed.keywords.iter().any(|k| title_lower.contains(&k.to_lowercase()))
                });
            }
            
            article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
            
            let max_articles = feed.max_articles.min(limit.unwrap_or(feed.max_articles));
            article_list.truncate(max_articles);
            
            article_list
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

    /// Get statistics
    #[allow(dead_code)]
    pub fn get_statistics(&self) -> NewsStats {
        let articles = self.articles.lock();
        let sources = self.sources.lock();
        
        let total_articles = articles.as_ref().ok().map(|a| a.len() as u32).unwrap_or(0);
        let total_sources = sources.as_ref().ok().map(|s| s.len() as u32).unwrap_or(0);
        
        let now = current_timestamp();
        let articles_last_24h = articles.as_ref()
            .ok()
            .map(|articles| articles.values()
            .filter(|a| now - a.published_at < 86400)
            .count() as u32)
            .unwrap_or(0);
        
        let most_active_source = self.article_index.lock()
            .ok()
            .and_then(|article_index| article_index.iter()
            .max_by_key(|(_, ids)| ids.len())
            .map(|(source_id, _)| sources.as_ref().ok().and_then(|s| s.get(source_id)).map(|s| s.name.clone()).unwrap_or_default()))
            .unwrap_or_default();

        NewsStats {
            total_articles,
            total_sources,
            articles_last_24h,
            most_active_source,
        }
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    articles: Arc<Mutex<HashMap<String, NewsArticle>>>,
    sources: Arc<Mutex<HashMap<String, NewsSource>>>,
    feeds: Arc<Mutex<HashMap<String, NewsFeedConfig>>>,
    article_index: Arc<Mutex<HashMap<String, Vec<String>>>>,
    category_index: Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

        let result = match method {
            "add_article" => handle_add_article(&params, &articles, &article_index, &category_index).await,
            "add_source" => handle_add_source(&params, &sources).await,
            "update_source" => handle_update_source(&params, &sources).await,
            "remove_source" => handle_remove_source(&params, &sources).await,
            "get_article" => handle_get_article(&params, &articles).await,
            "get_articles_by_source" => handle_get_articles_by_source(&params, &article_index, &articles).await,
            "get_articles_by_category" => handle_get_articles_by_category(&params, &category_index, &articles).await,
            "search_articles" => handle_search_articles(&params, &articles).await,
            "get_latest_articles" => handle_get_latest_articles(&params, &articles).await,
            "get_sources" => handle_get_sources(&sources).await,
            "create_feed" => handle_create_feed(&params, &feeds).await,
            "get_feed_articles" => handle_get_feed_articles(&params, &feeds, &article_index, &articles).await,
            "get_statistics" => handle_get_statistics(&articles, &sources, &article_index).await,
            "node_info" => handle_node_info(&node_id).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": result.unwrap_err()},
                "id": id
            })
        };

        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

async fn handle_add_article(
    params: &serde_json::Value,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
    article_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    category_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let article: NewsArticle = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid article: {}", e))?;
    
    let article_id = article.article_id.clone();
    let source_id = article.source_id.clone();
    let category = article.category.clone();
    
    let mut articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    articles_guard.insert(article_id.clone(), article.clone());
    drop(articles_guard);

    let mut article_index_guard = article_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    article_index_guard.entry(source_id).or_insert_with(Vec::new).push(article_id.clone());
    drop(article_index_guard);

    let mut category_index_guard = category_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    category_index_guard.entry(category).or_insert_with(Vec::new).push(article_id.clone());

    Ok(json!({
        "article_id": article_id
    }))
}

async fn handle_add_source(
    params: &serde_json::Value,
    sources: &Arc<Mutex<HashMap<String, NewsSource>>>,
) -> Result<serde_json::Value, String> {
    let source: NewsSource = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid source: {}", e))?;
    
    let source_id = source.source_id.clone();
    let mut guard = sources.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(source_id.clone(), source);

    Ok(json!({
        "source_id": source_id
    }))
}

async fn handle_update_source(
    params: &serde_json::Value,
    sources: &Arc<Mutex<HashMap<String, NewsSource>>>,
) -> Result<serde_json::Value, String> {
    let source: NewsSource = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid source: {}", e))?;
    
    let source_id = source.source_id.clone();
    let mut guard = sources.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(source_id, source);

    Ok(json!({
        "updated": true
    }))
}

async fn handle_remove_source(
    params: &serde_json::Value,
    sources: &Arc<Mutex<HashMap<String, NewsSource>>>,
) -> Result<serde_json::Value, String> {
    let source_id = params.get("source_id").and_then(|s| s.as_str()).ok_or("Missing source_id")?;
    
    let mut guard = sources.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.remove(source_id);

    Ok(json!({
        "removed": true
    }))
}

async fn handle_get_article(
    params: &serde_json::Value,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params.get("article_id").and_then(|a| a.as_str()).ok_or("Missing article_id")?;
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(article_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Article not found".to_string())
}

async fn handle_get_articles_by_source(
    params: &serde_json::Value,
    article_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let source_id = params.get("source_id").and_then(|s| s.as_str()).ok_or("Missing source_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let index_guard = article_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(article_ids) = index_guard.get(source_id) {
        let mut article_list: Vec<NewsArticle> = article_ids.iter()
            .filter_map(|id| articles_guard.get(id).cloned())
            .collect();
        article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        if let Some(limit) = limit {
            article_list.truncate(limit);
        }
        
        return Ok(json!({ "articles": article_list }));
    }

    Ok(json!({
        "articles": Vec::<NewsArticle>::new()
    }))
}

async fn handle_get_articles_by_category(
    params: &serde_json::Value,
    category_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let category = params.get("category").and_then(|c| c.as_str()).ok_or("Missing category")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let index_guard = category_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(article_ids) = index_guard.get(category) {
        let mut article_list: Vec<NewsArticle> = article_ids.iter()
            .filter_map(|id| articles_guard.get(id).cloned())
            .collect();
        article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        if let Some(limit) = limit {
            article_list.truncate(limit);
        }
        
        return Ok(json!({ "articles": article_list }));
    }

    Ok(json!({
        "articles": Vec::<NewsArticle>::new()
    }))
}

async fn handle_search_articles(
    params: &serde_json::Value,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let query = params.get("query").and_then(|q| q.as_str()).ok_or("Missing query")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<NewsArticle> = guard.values()
        .filter(|a| {
            a.title.to_lowercase().contains(&query_lower) ||
            a.summary.to_lowercase().contains(&query_lower) ||
            a.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "articles": results
    }))
}

async fn handle_get_latest_articles(
    params: &serde_json::Value,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut latest: Vec<NewsArticle> = guard.values().cloned().collect();
    latest.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    
    if let Some(limit) = limit {
        latest.truncate(limit);
    }

    Ok(json!({
        "articles": latest
    }))
}

async fn handle_get_sources(
    sources: &Arc<Mutex<HashMap<String, NewsSource>>>,
) -> Result<serde_json::Value, String> {
    let guard = sources.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(json!({
        "sources": guard.values().cloned().collect::<Vec<NewsSource>>()
    }))
}

async fn handle_create_feed(
    params: &serde_json::Value,
    feeds: &Arc<Mutex<HashMap<String, NewsFeedConfig>>>,
) -> Result<serde_json::Value, String> {
    let feed: NewsFeedConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid feed: {}", e))?;
    
    let feed_id = feed.feed_id.clone();
    let mut guard = feeds.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(feed_id.clone(), feed);

    Ok(json!({
        "feed_id": feed_id
    }))
}

async fn handle_get_feed_articles(
    params: &serde_json::Value,
    feeds: &Arc<Mutex<HashMap<String, NewsFeedConfig>>>,
    article_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
) -> Result<serde_json::Value, String> {
    let feed_id = params.get("feed_id").and_then(|f| f.as_str()).ok_or("Missing feed_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let feeds_guard = feeds.lock().map_err(|e| format!("Lock error: {}", e))?;
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(feed) = feeds_guard.get(feed_id) {
        let mut article_ids: Vec<String> = Vec::new();
        
        for source_id in &feed.source_ids {
            let index_guard = article_index.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(ids) = index_guard.get(source_id) {
                article_ids.extend(ids.clone());
            }
        }
        
        let mut article_list: Vec<NewsArticle> = article_ids.iter()
            .filter_map(|id| articles_guard.get(id).cloned())
            .collect();
        
        if !feed.categories.is_empty() {
            article_list.retain(|a| feed.categories.contains(&a.category));
        }
        
        if !feed.keywords.is_empty() {
            article_list.retain(|a| {
                let title_lower = a.title.to_lowercase();
                feed.keywords.iter().any(|k| title_lower.contains(&k.to_lowercase()))
            });
        }
        
        article_list.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        let max_articles = feed.max_articles.min(limit.unwrap_or(feed.max_articles));
        article_list.truncate(max_articles);
        
        return Ok(json!({ "articles": article_list }));
    }

    Ok(json!({
        "articles": Vec::<NewsArticle>::new()
    }))
}

async fn handle_get_statistics(
    articles: &Arc<Mutex<HashMap<String, NewsArticle>>>,
    sources: &Arc<Mutex<HashMap<String, NewsSource>>>,
    article_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    let sources_guard = sources.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let total_articles = articles_guard.len() as u32;
    let total_sources = sources_guard.len() as u32;
    
    let now = current_timestamp();
    let articles_last_24h = articles_guard.values()
        .filter(|a| now - a.published_at < 86400)
        .count() as u32;
    
    let index_guard = article_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let most_active_source = index_guard
        .iter()
        .max_by_key(|(_, ids)| ids.len())
        .map(|(source_id, _)| sources_guard.get(source_id).map(|s| s.name.clone()).unwrap_or_default())
        .unwrap_or_default();

    Ok(json!({
        "total_articles": total_articles,
        "total_sources": total_sources,
        "articles_last_24h": articles_last_24h,
        "most_active_source": most_active_source
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("news_aggregation_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_article() {
        let config = NewsAggregationServiceConfig::default();
        let service = NewsAggregationService::new(config).expect("Failed to create service");
        
        let article = NewsArticle {
            article_id: "article-1".to_string(),
            source_id: "source-1".to_string(),
            source_name: "Test Source".to_string(),
            title: "Test Article".to_string(),
            summary: "A test article".to_string(),
            content: "Test content".to_string(),
            author: Some("Author".to_string()),
            url: "https://example.com".to_string(),
            image_url: None,
            published_at: current_timestamp(),
            category: "technology".to_string(),
            tags: vec!["tech".to_string()],
            language: "en".to_string(),
        };

        service.add_article(article).expect("Failed to add article");
        let retrieved = service.get_article("article-1".to_string()).expect("Failed to get article");
        assert_eq!(retrieved.title, "Test Article");
    }

    #[test]
    fn test_add_and_get_source() {
        let config = NewsAggregationServiceConfig::default();
        let service = NewsAggregationService::new(config).expect("Failed to create service");
        
        let source = NewsSource {
            source_id: "source-1".to_string(),
            name: "Test Source".to_string(),
            url: "https://example.com".to_string(),
            source_type: "rss".to_string(),
            category: "technology".to_string(),
            language: "en".to_string(),
            update_interval: 3600,
            last_updated: current_timestamp(),
            is_active: true,
            icon_url: None,
        };

        service.add_source(source).expect("Failed to add source");
        let sources = service.get_sources();
        assert_eq!(sources.len(), 1);
    }
}
