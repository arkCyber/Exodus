//! Public Account Service - Self-media content distribution
//! 
//! This service provides public account management, follower subscriptions,
//! article publishing, and analytics for self-media content creators.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

use std::time::Duration;
/// Public account profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAccount {
    pub account_id: String,
    pub owner_id: String, // Contact ID of the owner
    pub name: String,
    pub description: String,
    pub avatar: Option<String>, // URL to avatar image
    pub cover_image: Option<String>, // URL to cover image
    pub category: String, // "technology", "entertainment", "education", etc.
    pub tags: Vec<String>,
    pub is_verified: bool,
    pub follower_count: u32,
    pub article_count: u32,
    pub total_views: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Article published by public account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub article_id: String,
    pub account_id: String,
    pub title: String,
    pub content: String, // Rich text content
    pub summary: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
    pub category: String, // Article category for browsing
    pub status: String, // "draft", "published", "archived", "scheduled"
    pub view_count: u64,
    pub like_count: u32,
    pub comment_count: u32,
    pub share_count: u32,
    pub is_pinned: bool,
    pub is_featured: bool,
    pub published_at: Option<u64>,
    pub scheduled_publish_time: Option<u64>, // Scheduled publish timestamp
    pub created_at: u64,
    pub updated_at: u64,
}

/// Follower subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follower {
    pub follower_id: String, // Contact ID of the follower
    pub account_id: String,
    pub subscribed_at: u64,
    pub last_read_article_id: Option<String>, // Last article read by follower
    pub notification_enabled: bool,
}

/// Media item for the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub media_id: String,
    pub account_id: String,
    pub media_type: String, // "image", "video", "audio"
    pub filename: String,
    pub url: String, // CDN URL or local path
    pub size_bytes: u64,
    pub width: Option<u32>, // For images/videos
    pub height: Option<u32>, // For images/videos
    pub duration: Option<u32>, // For videos/audio in seconds
    pub mime_type: String,
    pub tags: Vec<String>,
    pub created_at: u64,
}

/// Custom menu item for public account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMenuItem {
    pub menu_id: String,
    pub account_id: String,
    pub name: String,
    pub action_type: String, // "link", "article", "media", "external"
    pub action_value: String, // URL or article_id or media_id
    pub icon: Option<String>,
    pub order: u32,
    pub is_enabled: bool,
    pub created_at: u64,
}

/// Push notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub notification_id: String,
    pub account_id: String,
    pub recipient_id: String, // Contact ID of the recipient
    pub notification_type: String, // "article", "promotion", "system"
    pub title: String,
    pub content: String,
    pub link: Option<String>, // Optional link to article or content
    pub is_read: bool,
    pub created_at: u64,
}

/// Article analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleAnalytics {
    pub article_id: String,
    pub account_id: String,
    pub view_count: u64,
    pub unique_views: u64,
    pub like_count: u32,
    pub comment_count: u32,
    pub share_count: u32,
    pub read_time_avg: u32, // Average read time in seconds
    pub completion_rate: f32, // Percentage of readers who finished the article
    pub daily_views: Vec<(u64, u64)>, // (timestamp, view_count)
}

/// Account analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAnalytics {
    pub account_id: String,
    pub follower_count: u32,
    pub article_count: u32,
    pub total_views: u64,
    pub total_likes: u32,
    pub total_comments: u32,
    pub total_shares: u32,
    pub engagement_rate: f32, // (likes + comments + shares) / views
    pub daily_followers: Vec<(u64, u32)>, // (timestamp, follower_count)
    pub daily_views: Vec<(u64, u64)>, // (timestamp, view_count)
}

/// Configuration for Public Account Service
#[derive(Debug, Clone)]
pub struct PublicAccountServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for PublicAccountServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_public_account.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_public_accounts");
        
        Self { socket_path, storage_dir }
    }
}

/// Public Account Service
pub struct PublicAccountService {
    config: PublicAccountServiceConfig,
    accounts: Arc<Mutex<HashMap<String, PublicAccount>>>, // account_id -> account
    articles: Arc<Mutex<HashMap<String, Article>>>, // article_id -> article
    followers: Arc<Mutex<HashMap<String, Vec<Follower>>>>, // account_id -> followers
    user_subscriptions: Arc<Mutex<HashMap<String, Vec<String>>>>, // user_id -> account_ids
    article_analytics: Arc<Mutex<HashMap<String, ArticleAnalytics>>>, // article_id -> analytics
    account_analytics: Arc<Mutex<HashMap<String, AccountAnalytics>>>, // account_id -> analytics
    media_library: Arc<Mutex<HashMap<String, MediaItem>>>, // media_id -> media item
    account_media: Arc<Mutex<HashMap<String, Vec<String>>>>, // account_id -> media_ids
    custom_menu: Arc<Mutex<HashMap<String, CustomMenuItem>>>, // menu_id -> menu item
    account_menu: Arc<Mutex<HashMap<String, Vec<String>>>>, // account_id -> menu_ids
    push_notifications: Arc<Mutex<HashMap<String, PushNotification>>>, // notification_id -> notification
    recipient_notifications: Arc<Mutex<HashMap<String, Vec<String>>>>, // recipient_id -> notification_ids
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl PublicAccountService {
    pub fn new(config: PublicAccountServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            accounts: Arc::new(Mutex::new(HashMap::new())),
            articles: Arc::new(Mutex::new(HashMap::new())),
            followers: Arc::new(Mutex::new(HashMap::new())),
            user_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            article_analytics: Arc::new(Mutex::new(HashMap::new())),
            account_analytics: Arc::new(Mutex::new(HashMap::new())),
            media_library: Arc::new(Mutex::new(HashMap::new())),
            account_media: Arc::new(Mutex::new(HashMap::new())),
            custom_menu: Arc::new(Mutex::new(HashMap::new())),
            account_menu: Arc::new(Mutex::new(HashMap::new())),
            push_notifications: Arc::new(Mutex::new(HashMap::new())),
            recipient_notifications: Arc::new(Mutex::new(HashMap::new())),
            node_id: String::new(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
    }

    #[allow(dead_code)]
    pub fn with_node_id(config: PublicAccountServiceConfig, node_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            accounts: Arc::new(Mutex::new(HashMap::new())),
            articles: Arc::new(Mutex::new(HashMap::new())),
            followers: Arc::new(Mutex::new(HashMap::new())),
            user_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            article_analytics: Arc::new(Mutex::new(HashMap::new())),
            account_analytics: Arc::new(Mutex::new(HashMap::new())),
            media_library: Arc::new(Mutex::new(HashMap::new())),
            account_media: Arc::new(Mutex::new(HashMap::new())),
            custom_menu: Arc::new(Mutex::new(HashMap::new())),
            account_menu: Arc::new(Mutex::new(HashMap::new())),
            push_notifications: Arc::new(Mutex::new(HashMap::new())),
            recipient_notifications: Arc::new(Mutex::new(HashMap::new())),
            node_id,
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
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        let accounts = Arc::clone(&self.accounts);
        let articles = Arc::clone(&self.articles);
        let followers = Arc::clone(&self.followers);
        let user_subscriptions = Arc::clone(&self.user_subscriptions);
        let article_analytics = Arc::clone(&self.article_analytics);
        let account_analytics = Arc::clone(&self.account_analytics);
        let media_library = Arc::clone(&self.media_library);
        let account_media = Arc::clone(&self.account_media);
        let custom_menu = Arc::clone(&self.custom_menu);
        let account_menu = Arc::clone(&self.account_menu);
        let push_notifications = Arc::clone(&self.push_notifications);
        let recipient_notifications = Arc::clone(&self.recipient_notifications);
        let node_id = self.node_id.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let accounts = Arc::clone(&accounts);
                                let articles = Arc::clone(&articles);
                                let followers = Arc::clone(&followers);
                                let user_subscriptions = Arc::clone(&user_subscriptions);
                                let article_analytics = Arc::clone(&article_analytics);
                                let account_analytics = Arc::clone(&account_analytics);
                                let media_library = Arc::clone(&media_library);
                                let account_media = Arc::clone(&account_media);
                                let custom_menu = Arc::clone(&custom_menu);
                                let account_menu = Arc::clone(&account_menu);
                                let push_notifications = Arc::clone(&push_notifications);
                                let recipient_notifications = Arc::clone(&recipient_notifications);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, accounts, articles, followers, user_subscriptions, article_analytics, account_analytics, media_library, account_media, custom_menu, account_menu, push_notifications, recipient_notifications, node_id).await;
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

        println!("Public Account Service started on {:?}", socket_path);
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

        let socket_path = self.config.socket_path.clone();
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        println!("Public Account Service stopped");
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

    /// Create a public account
    #[allow(dead_code)]
    pub fn create_account(&self, account: PublicAccount) -> Result<(), String> {
        let account_id = account.account_id.clone();
        let _owner_id = account.owner_id.clone();
        
        let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        accounts.insert(account_id.clone(), account);
        drop(accounts);

        // Initialize account analytics
        let mut analytics = self.account_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        analytics.insert(account_id.clone(), AccountAnalytics {
            account_id,
            follower_count: 0,
            article_count: 0,
            total_views: 0,
            total_likes: 0,
            total_comments: 0,
            total_shares: 0,
            engagement_rate: 0.0,
            daily_followers: vec![],
            daily_views: vec![],
        });

        Ok(())
    }

    /// Get public account
    #[allow(dead_code)]
    pub fn get_account(&self, account_id: String) -> Option<PublicAccount> {
        let accounts = self.accounts.lock().ok()?;
        accounts.get(&account_id).cloned()
    }

    /// Update public account
    #[allow(dead_code)]
    pub fn update_account(&self, account: PublicAccount) -> Result<(), String> {
        let account_id = account.account_id.clone();
        let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        accounts.insert(account_id, account);
        Ok(())
    }

    /// List all accounts
    #[allow(dead_code)]
    pub fn list_accounts(&self) -> Vec<PublicAccount> {
        self.accounts.lock()
            .map(|accounts| accounts.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get accounts by owner
    #[allow(dead_code)]
    pub fn get_accounts_by_owner(&self, owner_id: String) -> Vec<PublicAccount> {
        self.accounts.lock()
            .map(|accounts| accounts.values()
            .filter(|a| a.owner_id == owner_id)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Publish an article
    #[allow(dead_code)]
    pub fn publish_article(&self, mut article: Article) -> Result<(), String> {
        let article_id = article.article_id.clone();
        let account_id = article.account_id.clone();
        
        article.published_at = Some(current_timestamp());
        article.status = "published".to_string();

        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        articles.insert(article_id.clone(), article.clone());
        drop(articles);

        // Update account article count
        let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.article_count += 1;
        }
        drop(accounts);

        // Initialize article analytics
        let mut analytics = self.article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        analytics.insert(article_id.clone(), ArticleAnalytics {
            article_id,
            account_id: account_id.clone(),
            view_count: 0,
            unique_views: 0,
            like_count: 0,
            comment_count: 0,
            share_count: 0,
            read_time_avg: 0,
            completion_rate: 0.0,
            daily_views: vec![],
        });

        Ok(())
    }

    /// Schedule an article for publishing
    #[allow(dead_code)]
    pub fn schedule_article(&self, article: Article, scheduled_time: u64) -> Result<(), String> {
        let article_id = article.article_id.clone();
        let _account_id = article.account_id.clone();
        
        let mut article = article;
        article.status = "scheduled".to_string();
        article.scheduled_publish_time = Some(scheduled_time);

        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        articles.insert(article_id.clone(), article);

        Ok(())
    }

    /// Process scheduled articles (publish those whose time has come)
    #[allow(dead_code)]
    pub fn process_scheduled_articles(&self) -> Result<Vec<String>, String> {
        let current = current_timestamp();
        let articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut to_publish: Vec<Article> = vec![];

        // Find articles that should be published
        for (_article_id, article) in articles.iter() {
            if article.status == "scheduled" {
                if let Some(scheduled_time) = article.scheduled_publish_time {
                    if scheduled_time <= current {
                        to_publish.push(article.clone());
                    }
                }
            }
        }

        drop(articles);

        let mut published_ids = vec![];

        // Publish the articles
        for mut article in to_publish {
            article.published_at = Some(current_timestamp());
            article.status = "published".to_string();
            article.scheduled_publish_time = None;
            
            let account_id = article.account_id.clone();
            let article_id = article.article_id.clone();

            let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
            articles.insert(article_id.clone(), article.clone());
            drop(articles);

            // Update account article count
            let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(account) = accounts.get_mut(&account_id) {
                account.article_count += 1;
            }
            drop(accounts);

            // Initialize article analytics
            let mut analytics = self.article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
            analytics.insert(article_id.clone(), ArticleAnalytics {
                article_id: article_id.clone(),
                account_id: account_id.clone(),
                view_count: 0,
                unique_views: 0,
                like_count: 0,
                comment_count: 0,
                share_count: 0,
                read_time_avg: 0,
                completion_rate: 0.0,
                daily_views: vec![],
            });

            published_ids.push(article_id);
        }

        Ok(published_ids)
    }

    /// Get scheduled articles
    #[allow(dead_code)]
    pub fn get_scheduled_articles(&self, account_id: String) -> Vec<Article> {
        self.articles.lock()
            .map(|articles| articles.values()
            .filter(|a| a.account_id == account_id && a.status == "scheduled")
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get article
    #[allow(dead_code)]
    pub fn get_article(&self, article_id: String) -> Option<Article> {
        let articles = self.articles.lock().ok()?;
        articles.get(&article_id).cloned()
    }

    /// List articles by account
    #[allow(dead_code)]
    pub fn list_articles_by_account(&self, account_id: String) -> Vec<Article> {
        self.articles.lock()
            .map(|articles| articles.values()
            .filter(|a| a.account_id == account_id && a.status == "published")
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Subscribe to a public account
    #[allow(dead_code)]
    pub fn subscribe(&self, follower_id: String, account_id: String) -> Result<(), String> {
        let follower = Follower {
            follower_id: follower_id.clone(),
            account_id: account_id.clone(),
            subscribed_at: current_timestamp(),
            last_read_article_id: None,
            notification_enabled: true,
        };

        // Add to followers list
        let mut followers = self.followers.lock().map_err(|e| format!("Lock error: {}", e))?;
        followers.entry(account_id.clone()).or_insert_with(Vec::new).push(follower.clone());
        drop(followers);

        // Add to user subscriptions
        let mut user_subscriptions = self.user_subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
        user_subscriptions.entry(follower_id).or_insert_with(Vec::new).push(account_id.clone());
        drop(user_subscriptions);

        // Update account follower count
        let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.follower_count += 1;
        }

        Ok(())
    }

    /// Unsubscribe from a public account
    #[allow(dead_code)]
    pub fn unsubscribe(&self, follower_id: String, account_id: String) -> Result<(), String> {
        // Remove from followers list
        let mut followers = self.followers.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(follower_list) = followers.get_mut(&account_id) {
            follower_list.retain(|f| f.follower_id != follower_id);
        }
        drop(followers);

        // Remove from user subscriptions
        let mut user_subscriptions = self.user_subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(sub_list) = user_subscriptions.get_mut(&follower_id) {
            sub_list.retain(|id| id != &account_id);
        }
        drop(user_subscriptions);

        // Update account follower count
        let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.follower_count = account.follower_count.saturating_sub(1);
        }

        Ok(())
    }

    /// Get followers of an account
    #[allow(dead_code)]
    pub fn get_followers(&self, account_id: String) -> Vec<Follower> {
        self.followers.lock()
            .map(|followers| followers.get(&account_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Get user's subscriptions
    #[allow(dead_code)]
    pub fn get_user_subscriptions(&self, user_id: String) -> Vec<String> {
        self.user_subscriptions.lock()
            .map(|user_subscriptions| user_subscriptions.get(&user_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Record article view
    #[allow(dead_code)]
    pub fn record_view(&self, article_id: String, _user_id: String) -> Result<(), String> {
        let mut analytics = self.article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(article_analytics) = analytics.get_mut(&article_id) {
            article_analytics.view_count += 1;
            article_analytics.unique_views += 1;
        }

        // Update article view count
        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(article) = articles.get_mut(&article_id) {
            article.view_count += 1;
        }

        // Update account total views
        let account_id = {
            let articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
            articles.get(&article_id).map(|a| a.account_id.clone())
        };

        if let Some(account_id) = account_id {
            let mut accounts = self.accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(account) = accounts.get_mut(&account_id) {
                account.total_views += 1;
            }
        }

        Ok(())
    }

    /// Like an article
    #[allow(dead_code)]
    pub fn like_article(&self, article_id: String, _user_id: String) -> Result<(), String> {
        let mut analytics = self.article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(article_analytics) = analytics.get_mut(&article_id) {
            article_analytics.like_count += 1;
        }

        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(article) = articles.get_mut(&article_id) {
            article.like_count += 1;
        }

        Ok(())
    }

    /// Get account analytics
    #[allow(dead_code)]
    pub fn get_account_analytics(&self, account_id: String) -> Option<AccountAnalytics> {
        let analytics = self.account_analytics.lock().ok()?;
        analytics.get(&account_id).cloned()
    }

    /// Get article analytics
    #[allow(dead_code)]
    pub fn get_article_analytics(&self, article_id: String) -> Option<ArticleAnalytics> {
        let analytics = self.article_analytics.lock().ok()?;
        analytics.get(&article_id).cloned()
    }

    /// Upload media to library
    #[allow(dead_code)]
    pub fn upload_media(&self, media: MediaItem) -> Result<(), String> {
        let media_id = media.media_id.clone();
        let account_id = media.account_id.clone();
        
        let mut media_library = self.media_library.lock().map_err(|e| format!("Lock error: {}", e))?;
        media_library.insert(media_id.clone(), media);
        drop(media_library);

        let mut account_media = self.account_media.lock().map_err(|e| format!("Lock error: {}", e))?;
        account_media.entry(account_id).or_insert_with(Vec::new).push(media_id);

        Ok(())
    }

    /// Delete media from library
    #[allow(dead_code)]
    pub fn delete_media(&self, media_id: String) -> Result<(), String> {
        let account_id = {
            let media_library = self.media_library.lock().map_err(|e| format!("Lock error: {}", e))?;
            media_library.get(&media_id).map(|m| m.account_id.clone())
        };

        if let Some(account_id) = account_id {
            let mut media_library = self.media_library.lock().map_err(|e| format!("Lock error: {}", e))?;
            media_library.remove(&media_id);
            drop(media_library);

            let mut account_media = self.account_media.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(media_list) = account_media.get_mut(&account_id) {
                media_list.retain(|id| id != &media_id);
            }
        }

        Ok(())
    }

    /// Get media item
    #[allow(dead_code)]
    pub fn get_media(&self, media_id: String) -> Option<MediaItem> {
        let media_library = self.media_library.lock().ok()?;
        media_library.get(&media_id).cloned()
    }

    /// List media by account
    #[allow(dead_code)]
    pub fn list_media_by_account(&self, account_id: String) -> Vec<MediaItem> {
        let account_media = self.account_media.lock();
        let media_ids = account_media.as_ref().ok().and_then(|am| am.get(&account_id).cloned()).unwrap_or_default();
        drop(account_media);

        let media_library = self.media_library.lock();
        media_library.as_ref().ok().map(|ml| media_ids.iter()
            .filter_map(|id| ml.get(id).cloned())
            .collect())
            .unwrap_or_default()
    }

    /// List media by type
    #[allow(dead_code)]
    pub fn list_media_by_type(&self, account_id: String, media_type: String) -> Vec<MediaItem> {
        let account_media = self.account_media.lock();
        let media_ids = account_media.as_ref().ok().and_then(|am| am.get(&account_id).cloned()).unwrap_or_default();
        drop(account_media);

        let media_library = self.media_library.lock();
        media_library.as_ref().ok().map(|ml| media_ids.iter()
            .filter_map(|id| ml.get(id).cloned())
            .filter(|m| m.media_type == media_type)
            .collect())
            .unwrap_or_default()
    }

    /// Save article as draft
    #[allow(dead_code)]
    pub fn save_draft(&self, article: Article) -> Result<(), String> {
        let article_id = article.article_id.clone();
        
        let mut article = article;
        article.status = "draft".to_string();
        article.published_at = None;

        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        articles.insert(article_id.clone(), article);

        Ok(())
    }

    /// List drafts by account
    #[allow(dead_code)]
    pub fn list_drafts(&self, account_id: String) -> Vec<Article> {
        self.articles.lock()
            .map(|articles| articles.values()
            .filter(|a| a.account_id == account_id && a.status == "draft")
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Delete draft
    #[allow(dead_code)]
    pub fn delete_draft(&self, article_id: String) -> Result<(), String> {
        let mut articles = self.articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(article) = articles.get(&article_id) {
            if article.status == "draft" {
                articles.remove(&article_id);
            }
        }

        Ok(())
    }

    /// Search public accounts by name or description
    #[allow(dead_code)]
    pub fn search_accounts(&self, query: String, limit: Option<usize>) -> Vec<PublicAccount> {
        let accounts = self.accounts.lock();
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<PublicAccount> = accounts.as_ref().ok().map(|accounts| accounts.values()
            .filter(|a| {
                a.name.to_lowercase().contains(&query_lower) ||
                a.description.to_lowercase().contains(&query_lower) ||
                a.category.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()).unwrap_or_default();

        results.sort_by(|a, b| b.follower_count.cmp(&a.follower_count));
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }

    /// Get trending accounts
    #[allow(dead_code)]
    pub fn get_trending_accounts(&self, limit: Option<usize>) -> Vec<PublicAccount> {
        let accounts = self.accounts.lock();
        let mut results: Vec<PublicAccount> = accounts.as_ref().ok().map(|a| a.values().cloned().collect()).unwrap_or_default();
        results.sort_by(|a, b| b.follower_count.cmp(&a.follower_count));
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        results
    }

    /// Get trending articles
    #[allow(dead_code)]
    pub fn get_trending_articles(&self, limit: Option<usize>) -> Vec<Article> {
        let articles = self.articles.lock();
        let article_analytics = self.article_analytics.lock();
        
        if let (Ok(articles), Ok(article_analytics)) = (articles, article_analytics) {
            let mut results: Vec<(Article, u64)> = articles.values()
                .filter(|a| a.status == "published")
                .map(|a| {
                    let views = article_analytics.get(&a.article_id)
                        .map(|stats| stats.view_count)
                        .unwrap_or(0);
                    (a.clone(), views)
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            
            let articles: Vec<Article> = results.into_iter()
                .map(|(a, _)| a)
                .collect();

            if let Some(limit) = limit {
                let mut truncated = articles;
                truncated.truncate(limit);
                truncated
            } else {
                articles
            }
        } else {
            Vec::new()
        }
    }

    /// Get articles by category
    #[allow(dead_code)]
    pub fn get_articles_by_category(&self, category: String, limit: Option<usize>) -> Vec<Article> {
        self.articles.lock()
            .map(|articles| {
                let mut results: Vec<Article> = articles.values()
                    .filter(|a| a.status == "published" && a.category == category)
                    .cloned()
                    .collect();
                
                if let Some(limit) = limit {
                    results.truncate(limit);
                }
                results
            })
            .unwrap_or_default()
    }

    /// Recommend articles based on user subscriptions and preferences
    #[allow(dead_code)]
    pub fn recommend_articles(&self, user_id: String, limit: Option<usize>) -> Vec<Article> {
        let user_subscriptions = self.user_subscriptions.lock();
        let subscriptions = user_subscriptions.as_ref().ok().and_then(|us| us.get(&user_id).cloned()).unwrap_or_default();
        drop(user_subscriptions);

        let articles = self.articles.lock();
        let article_analytics = self.article_analytics.lock();
        
        if let (Ok(articles), Ok(article_analytics)) = (articles, article_analytics) {
            let mut results: Vec<(Article, f32)> = articles.values()
                .filter(|a| a.status == "published")
                .filter(|a| subscriptions.contains(&a.account_id))
                .map(|a| {
                    let analytics = article_analytics.get(&a.article_id);
                    let view_count = analytics.and_then(|stats| Some(stats.view_count)).unwrap_or(0);
                    let like_count = analytics.and_then(|stats| Some(stats.like_count)).unwrap_or(0);
                    
                    // Simple recommendation score: weighted sum of views and likes
                    let score = (view_count as f32) * 0.7 + (like_count as f32) * 0.3;
                    (a.clone(), score)
                })
                .collect();

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let articles: Vec<Article> = results.into_iter()
                .map(|(a, _)| a)
                .collect();

            if let Some(limit) = limit {
                let mut truncated = articles;
                truncated.truncate(limit);
                truncated
            } else {
                articles
            }
        } else {
            Vec::new()
        }
    }

    /// Get real-time analytics updates for an account
    #[allow(dead_code)]
    pub fn get_realtime_analytics(&self, account_id: String) -> Result<AccountAnalytics, String> {
        let account_analytics = self.account_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // Get or create analytics for the account
        let analytics = account_analytics.get(&account_id)
            .cloned()
            .unwrap_or_else(|| {
                let accounts = self.accounts.lock();
                let follower_count = accounts.as_ref().ok().and_then(|a| a.get(&account_id).map(|acc| acc.follower_count)).unwrap_or(0);
                drop(accounts);
                let articles = self.articles.lock();
                let article_count = articles.as_ref().ok().map(|a| a.values()
                    .filter(|a| a.account_id == account_id)
                    .count() as u32).unwrap_or(0);
                
                AccountAnalytics {
                    account_id: account_id.clone(),
                    follower_count,
                    article_count,
                    total_views: 0,
                    total_likes: 0,
                    total_comments: 0,
                    total_shares: 0,
                    engagement_rate: 0.0,
                    daily_followers: vec![],
                    daily_views: vec![],
                }
            });

        Ok(analytics)
    }

    /// Add custom menu item
    #[allow(dead_code)]
    pub fn add_menu_item(&self, menu: CustomMenuItem) -> Result<(), String> {
        let menu_id = menu.menu_id.clone();
        let account_id = menu.account_id.clone();
        
        let mut custom_menu = self.custom_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
        custom_menu.insert(menu_id.clone(), menu);
        drop(custom_menu);

        let mut account_menu = self.account_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
        account_menu.entry(account_id).or_insert_with(Vec::new).push(menu_id);

        Ok(())
    }

    /// Update custom menu item
    #[allow(dead_code)]
    pub fn update_menu_item(&self, menu: CustomMenuItem) -> Result<(), String> {
        let menu_id = menu.menu_id.clone();
        
        let mut custom_menu = self.custom_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
        custom_menu.insert(menu_id.clone(), menu);

        Ok(())
    }

    /// Delete custom menu item
    #[allow(dead_code)]
    pub fn delete_menu_item(&self, menu_id: String) -> Result<(), String> {
        let account_id = {
            let custom_menu = self.custom_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
            custom_menu.get(&menu_id).map(|m| m.account_id.clone())
        };

        if let Some(account_id) = account_id {
            let mut custom_menu = self.custom_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
            custom_menu.remove(&menu_id);
            drop(custom_menu);

            let mut account_menu = self.account_menu.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(menu_list) = account_menu.get_mut(&account_id) {
                menu_list.retain(|id| id != &menu_id);
            }
        }

        Ok(())
    }

    /// Get menu items by account
    #[allow(dead_code)]
    pub fn get_menu_items(&self, account_id: String) -> Vec<CustomMenuItem> {
        let account_menu = self.account_menu.lock();
        let menu_ids = account_menu.as_ref().ok().and_then(|am| am.get(&account_id).cloned()).unwrap_or_default();
        drop(account_menu);

        let custom_menu = self.custom_menu.lock();
        custom_menu.as_ref().ok().map(|cm| menu_ids.iter()
            .filter_map(|id| cm.get(id).cloned())
            .collect())
            .unwrap_or_default()
    }

    /// Send push notification
    #[allow(dead_code)]
    pub fn send_notification(&self, notification: PushNotification) -> Result<(), String> {
        let notification_id = notification.notification_id.clone();
        let recipient_id = notification.recipient_id.clone();
        
        let mut push_notifications = self.push_notifications.lock().map_err(|e| format!("Lock error: {}", e))?;
        push_notifications.insert(notification_id.clone(), notification);
        drop(push_notifications);

        let mut recipient_notifications = self.recipient_notifications.lock().map_err(|e| format!("Lock error: {}", e))?;
        recipient_notifications.entry(recipient_id).or_insert_with(Vec::new).push(notification_id);

        Ok(())
    }

    /// Mark notification as read
    #[allow(dead_code)]
    pub fn mark_notification_read(&self, notification_id: String) -> Result<(), String> {
        let mut push_notifications = self.push_notifications.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(notification) = push_notifications.get_mut(&notification_id) {
            notification.is_read = true;
        }

        Ok(())
    }

    /// Get notifications by recipient
    #[allow(dead_code)]
    pub fn get_notifications(&self, recipient_id: String) -> Vec<PushNotification> {
        let recipient_notifications = self.recipient_notifications.lock();
        let notification_ids = recipient_notifications.as_ref().ok().and_then(|rn| rn.get(&recipient_id).cloned()).unwrap_or_default();
        drop(recipient_notifications);

        let push_notifications = self.push_notifications.lock();
        push_notifications.as_ref().ok().map(|pn| notification_ids.iter()
            .filter_map(|id| pn.get(id).cloned())
            .collect())
            .unwrap_or_default()
    }

    /// Get unread notifications
    #[allow(dead_code)]
    pub fn get_unread_notifications(&self, recipient_id: String) -> Vec<PushNotification> {
        let recipient_notifications = self.recipient_notifications.lock();
        let notification_ids = recipient_notifications.as_ref().ok().and_then(|rn| rn.get(&recipient_id).cloned()).unwrap_or_default();
        drop(recipient_notifications);

        let push_notifications = self.push_notifications.lock();
        push_notifications.as_ref().ok().map(|pn| notification_ids.iter()
            .filter_map(|id| pn.get(id).cloned())
            .filter(|n| !n.is_read)
            .collect())
            .unwrap_or_default()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

/// JSON-RPC request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: serde_json::Value,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

/// JSON-RPC error
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Handle incoming client connection
async fn handle_client(
    mut stream: tokio::net::UnixStream,
    accounts: Arc<Mutex<HashMap<String, PublicAccount>>>,
    articles: Arc<Mutex<HashMap<String, Article>>>,
    followers: Arc<Mutex<HashMap<String, Vec<Follower>>>>,
    user_subscriptions: Arc<Mutex<HashMap<String, Vec<String>>>>,
    article_analytics: Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
    account_analytics: Arc<Mutex<HashMap<String, AccountAnalytics>>>,
    media_library: Arc<Mutex<HashMap<String, MediaItem>>>,
    account_media: Arc<Mutex<HashMap<String, Vec<String>>>>,
    custom_menu: Arc<Mutex<HashMap<String, CustomMenuItem>>>,
    account_menu: Arc<Mutex<HashMap<String, Vec<String>>>>,
    push_notifications: Arc<Mutex<HashMap<String, PushNotification>>>,
    recipient_notifications: Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let request: JsonRpcRequest = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse request: {}", e))?;
        
        let method = request.method.as_str();
        let params = request.params.clone();
        let id = request.id.clone();

        let result = match method {
            "create_account" => handle_create_account(&params, &accounts).await,
            "get_account" => handle_get_account(&params, &accounts).await,
            "update_account" => handle_update_account(&params, &accounts).await,
            "list_accounts" => handle_list_accounts(&accounts).await,
            "get_accounts_by_owner" => handle_get_accounts_by_owner(&params, &accounts).await,
            "search_accounts" => handle_search_accounts(&params, &accounts).await,
            "get_trending_articles" => handle_get_trending_articles(&params, &articles, &article_analytics).await,
            "get_articles_by_category" => handle_get_articles_by_category(&params, &articles).await,
            "recommend_articles" => handle_recommend_articles(&params, &articles, &article_analytics, &user_subscriptions).await,
            "get_realtime_analytics" => handle_get_realtime_analytics(&params, &accounts, &account_analytics, &articles).await,
            "publish_article" => handle_publish_article(&params, &articles, &accounts).await,
            "schedule_article" => handle_schedule_article(&params, &articles).await,
            "process_scheduled_articles" => handle_process_scheduled_articles(&articles, &accounts, &article_analytics).await,
            "get_scheduled_articles" => handle_get_scheduled_articles(&params, &articles).await,
            "get_article" => handle_get_article(&params, &articles).await,
            "list_articles_by_account" => handle_list_articles_by_account(&params, &articles).await,
            "subscribe" => handle_subscribe(&params, &followers, &user_subscriptions, &accounts).await,
            "unsubscribe" => handle_unsubscribe(&params, &followers, &user_subscriptions, &accounts).await,
            "get_followers" => handle_get_followers(&params, &followers).await,
            "get_user_subscriptions" => handle_get_user_subscriptions(&params, &user_subscriptions).await,
            "record_view" => handle_record_view(&params, &article_analytics, &articles, &accounts).await,
            "like_article" => handle_like_article(&params, &article_analytics, &articles).await,
            "get_account_analytics" => handle_get_account_analytics(&params, &account_analytics).await,
            "get_article_analytics" => handle_get_article_analytics(&params, &article_analytics).await,
            "upload_media" => handle_upload_media(&params, &media_library, &account_media).await,
            "delete_media" => handle_delete_media(&params, &media_library, &account_media).await,
            "get_media" => handle_get_media(&params, &media_library).await,
            "list_media_by_account" => handle_list_media_by_account(&params, &media_library, &account_media).await,
            "list_media_by_type" => handle_list_media_by_type(&params, &media_library, &account_media).await,
            "save_draft" => handle_save_draft(&params, &articles).await,
            "list_drafts" => handle_list_drafts(&params, &articles).await,
            "delete_draft" => handle_delete_draft(&params, &articles).await,
            "add_menu_item" => handle_add_menu_item(&params, &custom_menu, &account_menu).await,
            "update_menu_item" => handle_update_menu_item(&params, &custom_menu).await,
            "delete_menu_item" => handle_delete_menu_item(&params, &custom_menu, &account_menu).await,
            "get_menu_items" => handle_get_menu_items(&params, &custom_menu, &account_menu).await,
            "send_notification" => handle_send_notification(&params, &push_notifications, &recipient_notifications).await,
            "mark_notification_read" => handle_mark_notification_read(&params, &push_notifications).await,
            "get_notifications" => handle_get_notifications(&params, &push_notifications, &recipient_notifications).await,
            "get_unread_notifications" => handle_get_unread_notifications(&params, &push_notifications, &recipient_notifications).await,
            "node_info" => handle_node_info(&node_id).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else if let Err(e) = result {
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": e
                },
                "id": id
            })
        } else {
            continue;
        };

        let response_str = serde_json::to_string(&response)?;
        writer.write_all(response_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}

async fn handle_create_account(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let account: PublicAccount = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid account: {}", e))?;
    
    let account_id = account.account_id.clone();
    let mut guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(account_id.clone(), account);

    Ok(json!({
        "account_id": account_id
    }))
}

async fn handle_get_account(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(account_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Account not found".to_string())
}

async fn handle_update_account(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let account: PublicAccount = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid account: {}", e))?;
    
    let account_id = account.account_id.clone();
    let mut guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(account_id.clone(), account);

    Ok(json!({
        "account_id": account_id
    }))
}

async fn handle_list_accounts(
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let account_list: Vec<PublicAccount> = guard.values().cloned().collect();
    
    Ok(json!({
        "accounts": account_list
    }))
}

async fn handle_get_accounts_by_owner(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let owner_id = params.as_ref()
        .and_then(|p| p.get("owner_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing owner_id")?;
    
    let guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<PublicAccount> = guard.values()
        .filter(|a| a.owner_id == owner_id)
        .cloned()
        .collect();

    Ok(json!({
        "accounts": found
    }))
}

async fn handle_publish_article(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let mut article: Article = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid article: {}", e))?;
    
    let article_id = article.article_id.clone();
    let account_id = article.account_id.clone();
    
    article.published_at = Some(current_timestamp());
    article.status = "published".to_string();

    let mut articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    articles_guard.insert(article_id.clone(), article.clone());
    drop(articles_guard);

    // Update account article count
    let mut accounts_guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(account) = accounts_guard.get_mut(&account_id) {
        account.article_count += 1;
    }

    Ok(json!({
        "article_id": article_id
    }))
}

async fn handle_get_article(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params.as_ref()
        .and_then(|p| p.get("article_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing article_id")?;
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(article_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Article not found".to_string())
}

async fn handle_list_articles_by_account(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Article> = guard.values()
        .filter(|a| a.account_id == account_id && a.status == "published")
        .cloned()
        .collect();

    Ok(json!({
        "articles": found
    }))
}

async fn handle_schedule_article(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let article: Article = serde_json::from_value(params.as_ref()
        .and_then(|p| p.get("article"))
        .cloned()
        .unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid article: {}", e))?;
    
    let scheduled_time = params.as_ref()
        .and_then(|p| p.get("scheduled_time"))
        .and_then(|v| v.as_u64())
        .ok_or("Missing scheduled_time")?;
    
    let article_id = article.article_id.clone();
    
    let mut article = article;
    article.status = "scheduled".to_string();
    article.scheduled_publish_time = Some(scheduled_time);

    let mut guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(article_id.clone(), article);

    Ok(json!({
        "article_id": article_id,
        "scheduled_time": scheduled_time
    }))
}

async fn handle_process_scheduled_articles(
    articles: &Arc<Mutex<HashMap<String, Article>>>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
) -> Result<serde_json::Value, String> {
    let current = current_timestamp();
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut to_publish: Vec<Article> = vec![];

    for (_article_id, article) in articles_guard.iter() {
        if article.status == "scheduled" {
            if let Some(scheduled_time) = article.scheduled_publish_time {
                if scheduled_time <= current {
                    to_publish.push(article.clone());
                }
            }
        }
    }

    drop(articles_guard);

    let mut published_ids = vec![];

    for mut article in to_publish {
        article.published_at = Some(current_timestamp());
        article.status = "published".to_string();
        article.scheduled_publish_time = None;
        
        let account_id = article.account_id.clone();
        let article_id = article.article_id.clone();

        let mut articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
        articles_guard.insert(article_id.clone(), article.clone());
        drop(articles_guard);

        let mut accounts_guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(account) = accounts_guard.get_mut(&account_id) {
            account.article_count += 1;
        }
        drop(accounts_guard);

        let mut analytics_guard = article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
        analytics_guard.insert(article_id.clone(), ArticleAnalytics {
            article_id: article_id.clone(),
            account_id: account_id.clone(),
            view_count: 0,
            unique_views: 0,
            like_count: 0,
            comment_count: 0,
            share_count: 0,
            read_time_avg: 0,
            completion_rate: 0.0,
            daily_views: vec![],
        });

        published_ids.push(article_id);
    }

    Ok(json!({
        "published_articles": published_ids
    }))
}

async fn handle_get_scheduled_articles(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Article> = guard.values()
        .filter(|a| a.account_id == account_id && a.status == "scheduled")
        .cloned()
        .collect();

    Ok(json!({
        "articles": found
    }))
}

async fn handle_subscribe(
    params: &Option<serde_json::Value>,
    followers: &Arc<Mutex<HashMap<String, Vec<Follower>>>>,
    user_subscriptions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let follower_id = params.as_ref()
        .and_then(|p| p.get("follower_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing follower_id")?;
    
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let follower = Follower {
        follower_id: follower_id.to_string(),
        account_id: account_id.to_string(),
        subscribed_at: current_timestamp(),
        last_read_article_id: None,
        notification_enabled: true,
    };

    let mut followers_guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    followers_guard.entry(account_id.to_string()).or_insert_with(Vec::new).push(follower);
    drop(followers_guard);

    let mut user_subscriptions_guard = user_subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    user_subscriptions_guard.entry(follower_id.to_string()).or_insert_with(Vec::new).push(account_id.to_string());
    drop(user_subscriptions_guard);

    let mut accounts_guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(account) = accounts_guard.get_mut(account_id) {
        account.follower_count += 1;
    }

    Ok(json!({
        "subscribed": true
    }))
}

async fn handle_unsubscribe(
    params: &Option<serde_json::Value>,
    followers: &Arc<Mutex<HashMap<String, Vec<Follower>>>>,
    user_subscriptions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let follower_id = params.as_ref()
        .and_then(|p| p.get("follower_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing follower_id")?;
    
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let mut followers_guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(follower_list) = followers_guard.get_mut(account_id) {
        follower_list.retain(|f| f.follower_id != follower_id);
    }
    drop(followers_guard);

    let mut user_subscriptions_guard = user_subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(sub_list) = user_subscriptions_guard.get_mut(follower_id) {
        sub_list.retain(|id| id != account_id);
    }
    drop(user_subscriptions_guard);

    let mut accounts_guard = accounts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(account) = accounts_guard.get_mut(account_id) {
        account.follower_count = account.follower_count.saturating_sub(1);
    }

    Ok(json!({
        "unsubscribed": true
    }))
}

async fn handle_get_followers(
    params: &Option<serde_json::Value>,
    followers: &Arc<Mutex<HashMap<String, Vec<Follower>>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    let follower_list = guard.get(account_id).cloned().unwrap_or_default();

    Ok(json!({
        "followers": follower_list
    }))
}

async fn handle_get_user_subscriptions(
    params: &Option<serde_json::Value>,
    user_subscriptions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.as_ref()
        .and_then(|p| p.get("user_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing user_id")?;
    
    let guard = user_subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let subscriptions = guard.get(user_id).cloned().unwrap_or_default();

    Ok(json!({
        "subscriptions": subscriptions
    }))
}

async fn handle_record_view(
    params: &Option<serde_json::Value>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
    _accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params.as_ref()
        .and_then(|p| p.get("article_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing article_id")?;
    
    let mut analytics_guard = article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(analytics) = analytics_guard.get_mut(article_id) {
        analytics.view_count += 1;
        analytics.unique_views += 1;
    }
    drop(analytics_guard);

    let mut articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(article) = articles_guard.get_mut(article_id) {
        article.view_count += 1;
    }

    Ok(json!({
        "recorded": true
    }))
}

async fn handle_like_article(
    params: &Option<serde_json::Value>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params.as_ref()
        .and_then(|p| p.get("article_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing article_id")?;
    
    let mut analytics_guard = article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(analytics) = analytics_guard.get_mut(article_id) {
        analytics.like_count += 1;
    }
    drop(analytics_guard);

    let mut articles_guard = articles.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(article) = articles_guard.get_mut(article_id) {
        article.like_count += 1;
    }

    Ok(json!({
        "liked": true
    }))
}

async fn handle_get_account_analytics(
    params: &Option<serde_json::Value>,
    account_analytics: &Arc<Mutex<HashMap<String, AccountAnalytics>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = account_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(account_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Analytics not found".to_string())
}

async fn handle_get_article_analytics(
    params: &Option<serde_json::Value>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params.as_ref()
        .and_then(|p| p.get("article_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing article_id")?;
    
    let guard = article_analytics.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(article_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Analytics not found".to_string())
}

async fn handle_upload_media(
    params: &Option<serde_json::Value>,
    media_library: &Arc<Mutex<HashMap<String, MediaItem>>>,
    account_media: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let media: MediaItem = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid media: {e}"))?;
    let media_id = media.media_id.clone();
    let account_id = media.account_id.clone();
    {
        let mut lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
        lib.insert(media_id.clone(), media);
    }
    {
        let mut am = account_media.lock().map_err(|e| format!("Lock error: {e}"))?;
        am.entry(account_id).or_default().push(media_id.clone());
    }
    Ok(json!({ "media_id": media_id }))
}

async fn handle_delete_media(
    params: &Option<serde_json::Value>,
    media_library: &Arc<Mutex<HashMap<String, MediaItem>>>,
    account_media: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let media_id = params
        .as_ref()
        .and_then(|p| p.get("media_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing media_id")?
        .to_string();
    let account_id = {
        let lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
        lib.get(&media_id).map(|m| m.account_id.clone())
    };
    if let Some(account_id) = account_id {
        let mut lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
        lib.remove(&media_id);
        let mut am = account_media.lock().map_err(|e| format!("Lock error: {e}"))?;
        if let Some(list) = am.get_mut(&account_id) {
            list.retain(|id| id != &media_id);
        }
    }
    Ok(json!({ "deleted": true }))
}

async fn handle_get_media(
    params: &Option<serde_json::Value>,
    media_library: &Arc<Mutex<HashMap<String, MediaItem>>>,
) -> Result<serde_json::Value, String> {
    let media_id = params
        .as_ref()
        .and_then(|p| p.get("media_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing media_id")?;
    let lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
    lib.get(media_id)
        .map(|m| json!(m))
        .ok_or_else(|| "Media not found".to_string())
}

async fn handle_list_media_by_account(
    params: &Option<serde_json::Value>,
    media_library: &Arc<Mutex<HashMap<String, MediaItem>>>,
    account_media: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params
        .as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    let ids = {
        let am = account_media.lock().map_err(|e| format!("Lock error: {e}"))?;
        am.get(account_id).cloned().unwrap_or_default()
    };
    let lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
    let items: Vec<&MediaItem> = ids.iter().filter_map(|id| lib.get(id)).collect();
    Ok(json!({ "media": items }))
}

async fn handle_list_media_by_type(
    params: &Option<serde_json::Value>,
    media_library: &Arc<Mutex<HashMap<String, MediaItem>>>,
    account_media: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params
        .as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    let media_type = params
        .as_ref()
        .and_then(|p| p.get("media_type"))
        .and_then(|v| v.as_str())
        .ok_or("Missing media_type")?;
    let ids = {
        let am = account_media.lock().map_err(|e| format!("Lock error: {e}"))?;
        am.get(account_id).cloned().unwrap_or_default()
    };
    let lib = media_library.lock().map_err(|e| format!("Lock error: {e}"))?;
    let items: Vec<&MediaItem> = ids
        .iter()
        .filter_map(|id| lib.get(id))
        .filter(|m| m.media_type == media_type)
        .collect();
    Ok(json!({ "media": items }))
}

async fn handle_node_info(
    node_id: &str,
) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "service": "public_account"
    }))
}

async fn handle_save_draft(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let article: Article = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid article: {e}"))?;
    let article_id = article.article_id.clone();
    
    let mut article = article;
    article.status = "draft".to_string();
    article.published_at = None;

    let mut guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    guard.insert(article_id.clone(), article);

    Ok(json!({ "article_id": article_id }))
}

async fn handle_list_drafts(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params
        .as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    let drafts: Vec<Article> = guard.values()
        .filter(|a| a.account_id == account_id && a.status == "draft")
        .cloned()
        .collect();

    Ok(json!({ "drafts": drafts }))
}

async fn handle_delete_draft(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let article_id = params
        .as_ref()
        .and_then(|p| p.get("article_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing article_id")?;
    
    let mut guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(article) = guard.get(article_id) {
        if article.status == "draft" {
            guard.remove(article_id);
        }
    }

    Ok(json!({ "deleted": true }))
}

async fn handle_add_menu_item(
    params: &Option<serde_json::Value>,
    custom_menu: &Arc<Mutex<HashMap<String, CustomMenuItem>>>,
    account_menu: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let menu: CustomMenuItem = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid menu: {e}"))?;
    let menu_id = menu.menu_id.clone();
    let account_id = menu.account_id.clone();
    
    let mut guard = custom_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
    guard.insert(menu_id.clone(), menu);
    drop(guard);

    let mut account_guard = account_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
    account_guard.entry(account_id).or_default().push(menu_id.clone());

    Ok(json!({ "menu_id": menu_id }))
}

async fn handle_update_menu_item(
    params: &Option<serde_json::Value>,
    custom_menu: &Arc<Mutex<HashMap<String, CustomMenuItem>>>,
) -> Result<serde_json::Value, String> {
    let menu: CustomMenuItem = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid menu: {e}"))?;
    let menu_id = menu.menu_id.clone();
    
    let mut guard = custom_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
    guard.insert(menu_id.clone(), menu);

    Ok(json!({ "updated": true }))
}

async fn handle_delete_menu_item(
    params: &Option<serde_json::Value>,
    custom_menu: &Arc<Mutex<HashMap<String, CustomMenuItem>>>,
    account_menu: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let menu_id = params
        .as_ref()
        .and_then(|p| p.get("menu_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing menu_id")?;
    
    let account_id = {
        let guard = custom_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
        guard.get(menu_id).map(|m| m.account_id.clone())
    };

    if let Some(account_id) = account_id {
        let mut guard = custom_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
        guard.remove(menu_id);
        drop(guard);

        let mut account_guard = account_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
        if let Some(menu_list) = account_guard.get_mut(&account_id) {
            menu_list.retain(|id| id != menu_id);
        }
    }

    Ok(json!({ "deleted": true }))
}

async fn handle_get_menu_items(
    params: &Option<serde_json::Value>,
    custom_menu: &Arc<Mutex<HashMap<String, CustomMenuItem>>>,
    account_menu: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params
        .as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let account_guard = account_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
    let menu_ids = account_guard.get(account_id).cloned().unwrap_or_default();
    drop(account_guard);

    let guard = custom_menu.lock().map_err(|e| format!("Lock error: {e}"))?;
    let menu_items: Vec<CustomMenuItem> = menu_ids.iter()
        .filter_map(|id| guard.get(id).cloned())
        .collect();

    Ok(json!({ "menu_items": menu_items }))
}

async fn handle_send_notification(
    params: &Option<serde_json::Value>,
    push_notifications: &Arc<Mutex<HashMap<String, PushNotification>>>,
    recipient_notifications: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let notification: PushNotification = serde_json::from_value(params.clone().unwrap_or(json!(null)))
        .map_err(|e| format!("Invalid notification: {e}"))?;
    let notification_id = notification.notification_id.clone();
    let recipient_id = notification.recipient_id.clone();
    
    let mut guard = push_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    guard.insert(notification_id.clone(), notification);
    drop(guard);

    let mut recipient_guard = recipient_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    recipient_guard.entry(recipient_id).or_default().push(notification_id.clone());

    Ok(json!({ "notification_id": notification_id }))
}

async fn handle_mark_notification_read(
    params: &Option<serde_json::Value>,
    push_notifications: &Arc<Mutex<HashMap<String, PushNotification>>>,
) -> Result<serde_json::Value, String> {
    let notification_id = params
        .as_ref()
        .and_then(|p| p.get("notification_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing notification_id")?;
    
    let mut guard = push_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(notification) = guard.get_mut(notification_id) {
        notification.is_read = true;
    }

    Ok(json!({ "marked": true }))
}

async fn handle_get_notifications(
    params: &Option<serde_json::Value>,
    push_notifications: &Arc<Mutex<HashMap<String, PushNotification>>>,
    recipient_notifications: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let recipient_id = params
        .as_ref()
        .and_then(|p| p.get("recipient_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing recipient_id")?;
    
    let recipient_guard = recipient_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    let notification_ids = recipient_guard.get(recipient_id).cloned().unwrap_or_default();
    drop(recipient_guard);

    let guard = push_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    let notifications: Vec<PushNotification> = notification_ids.iter()
        .filter_map(|id| guard.get(id).cloned())
        .collect();

    Ok(json!({ "notifications": notifications }))
}

async fn handle_get_unread_notifications(
    params: &Option<serde_json::Value>,
    push_notifications: &Arc<Mutex<HashMap<String, PushNotification>>>,
    recipient_notifications: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let recipient_id = params
        .as_ref()
        .and_then(|p| p.get("recipient_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing recipient_id")?;
    
    let recipient_guard = recipient_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    let notification_ids = recipient_guard.get(recipient_id).cloned().unwrap_or_default();
    drop(recipient_guard);

    let guard = push_notifications.lock().map_err(|e| format!("Lock error: {e}"))?;
    let notifications: Vec<PushNotification> = notification_ids.iter()
        .filter_map(|id| guard.get(id).cloned())
        .filter(|n| !n.is_read)
        .collect();

    Ok(json!({ "notifications": notifications }))
}

async fn handle_search_accounts(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
) -> Result<serde_json::Value, String> {
    let query = params
        .as_ref()
        .and_then(|p| p.get("query"))
        .and_then(|v| v.as_str())
        .ok_or("Missing query")?;
    let limit = params
        .as_ref()
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|l| l as usize);
    
    let guard = accounts.lock().map_err(|e| format!("Lock error: {e}"))?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<PublicAccount> = guard.values()
        .filter(|a| {
            a.name.to_lowercase().contains(&query_lower) ||
            a.description.to_lowercase().contains(&query_lower) ||
            a.category.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.follower_count.cmp(&a.follower_count));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({ "accounts": results }))
}

async fn handle_get_trending_articles(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
) -> Result<serde_json::Value, String> {
    let limit = params
        .as_ref()
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|l| l as usize);
    
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    let analytics_guard = article_analytics.lock().map_err(|e| format!("Lock error: {e}"))?;
    
    let mut results: Vec<(Article, u64)> = articles_guard.values()
        .filter(|a| a.status == "published")
        .map(|a| {
            let views = analytics_guard.get(&a.article_id)
                .map(|stats| stats.view_count)
                .unwrap_or(0);
            (a.clone(), views)
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    
    let articles: Vec<Article> = results.into_iter()
        .map(|(a, _)| a)
        .collect();

    if let Some(limit) = limit {
        let mut truncated = articles;
        truncated.truncate(limit);
        Ok(json!({ "articles": truncated }))
    } else {
        Ok(json!({ "articles": articles }))
    }
}

async fn handle_get_articles_by_category(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let category = params
        .as_ref()
        .and_then(|p| p.get("category"))
        .and_then(|v| v.as_str())
        .ok_or("Missing category")?;
    let limit = params
        .as_ref()
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|l| l as usize);
    
    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    
    let mut results: Vec<Article> = articles_guard.values()
        .filter(|a| a.status == "published" && a.category == category)
        .cloned()
        .collect();

    results.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({ "articles": results }))
}

async fn handle_recommend_articles(
    params: &Option<serde_json::Value>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
    article_analytics: &Arc<Mutex<HashMap<String, ArticleAnalytics>>>,
    user_subscriptions: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params
        .as_ref()
        .and_then(|p| p.get("user_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing user_id")?;
    let limit = params
        .as_ref()
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|l| l as usize);
    
    let subscriptions_guard = user_subscriptions.lock().map_err(|e| format!("Lock error: {e}"))?;
    let subscriptions = subscriptions_guard.get(user_id).cloned().unwrap_or_default();
    drop(subscriptions_guard);

    let articles_guard = articles.lock().map_err(|e| format!("Lock error: {e}"))?;
    let analytics_guard = article_analytics.lock().map_err(|e| format!("Lock error: {e}"))?;
    
    let mut results: Vec<(Article, f32)> = articles_guard.values()
        .filter(|a| a.status == "published")
        .filter(|a| subscriptions.contains(&a.account_id))
        .map(|a| {
            let analytics = analytics_guard.get(&a.article_id);
            let view_count = analytics.map(|stats| stats.view_count).unwrap_or(0);
            let like_count = analytics.map(|stats| stats.like_count).unwrap_or(0);
            
            let score = (view_count as f32) * 0.7 + (like_count as f32) * 0.3;
            (a.clone(), score)
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let articles: Vec<Article> = results.into_iter()
        .map(|(a, _)| a)
        .collect();

    if let Some(limit) = limit {
        let mut truncated = articles;
        truncated.truncate(limit);
        Ok(json!({ "articles": truncated }))
    } else {
        Ok(json!({ "articles": articles }))
    }
}

async fn handle_get_realtime_analytics(
    params: &Option<serde_json::Value>,
    accounts: &Arc<Mutex<HashMap<String, PublicAccount>>>,
    account_analytics: &Arc<Mutex<HashMap<String, AccountAnalytics>>>,
    articles: &Arc<Mutex<HashMap<String, Article>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params
        .as_ref()
        .and_then(|p| p.get("account_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing account_id")?;
    
    let analytics_guard = account_analytics.lock().map_err(|e| format!("Lock error: {e}"))?;
    
    // Get or create analytics for the account
    let analytics = analytics_guard.get(account_id)
        .cloned()
        .unwrap_or_else(|| {
            let accounts_guard = accounts.lock();
            let follower_count = accounts_guard.as_ref().ok().and_then(|ag| ag.get(account_id).map(|a| a.follower_count)).unwrap_or(0);
            drop(accounts_guard);
            let articles_guard = articles.lock();
            let article_count = articles_guard.as_ref().ok().map(|ag| ag.values()
                .filter(|a| a.account_id == account_id)
                .count() as u32).unwrap_or(0);
            
            AccountAnalytics {
                account_id: account_id.to_string(),
                follower_count,
                article_count,
                total_views: 0,
                total_likes: 0,
                total_comments: 0,
                total_shares: 0,
                engagement_rate: 0.0,
                daily_followers: vec![],
                daily_views: vec![],
            }
        });

    Ok(json!(analytics))
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_public_account_service_config_default() {
        let config = PublicAccountServiceConfig::default();
        assert!(config.socket_path.ends_with("exodus_public_account.sock"));
    }
    
    #[test]
    fn test_public_account_service_creation() {
        let config = PublicAccountServiceConfig::default();
        let service = PublicAccountService::new(config);
        assert!(service.is_ok());
    }
}
