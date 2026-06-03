//! Tauri commands for Public Account Service
//! 
//! These commands allow the frontend to interact with the Public Account Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{PublicAccountService, PublicAccountServiceConfig, PublicAccount, Article, Follower, ArticleAnalytics, AccountAnalytics, MediaItem, CustomMenuItem, PushNotification};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Public Account Service instance
pub struct ManagedPublicAccountService {
    service: Arc<PublicAccountService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedPublicAccountService {
    pub fn new(service: PublicAccountService) -> Self {
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

/// Send JSON-RPC request to Public Account Service
async fn send_public_account_request(
    socket_path: &std::path::PathBuf,
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
        .map_err(|e| format!("Failed to connect to Public Account Service: {}", e))?;

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
        return Err(format!("Public Account Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Public Account Service
#[tauri::command]
pub async fn public_account_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = PublicAccountServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = PublicAccountService::new(config)
        .map_err(|e| e.to_string())?;
    
    let managed = ManagedPublicAccountService::new(service);
    managed.start().await?;
    
    let _ = app.emit("public-account-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Public Account Service
#[tauri::command]
pub async fn public_account_service_stop() -> Result<(), String> {
    let config = PublicAccountServiceConfig::default();
    let service = PublicAccountService::new(config)
        .map_err(|e| e.to_string())?;
    
    let managed = ManagedPublicAccountService::new(service);
    managed.stop().await?;
    
    Ok(())
}

/// Create a public account
#[tauri::command]
pub async fn public_account_create(account: PublicAccount) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(account);
    let result = send_public_account_request(&config.socket_path, "create_account", params).await?;
    
    result.get("account_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid account_id response".to_string())
}

/// Get public account
#[tauri::command]
pub async fn public_account_get(account_id: String) -> Result<PublicAccount, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_account", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}

/// Update public account
#[tauri::command]
pub async fn public_account_update(account: PublicAccount) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(account);
    let result = send_public_account_request(&config.socket_path, "update_account", params).await?;
    
    result.get("account_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid account_id response".to_string())
}

/// List all public accounts
#[tauri::command]
pub async fn public_account_list() -> Result<Vec<PublicAccount>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(null);
    let result = send_public_account_request(&config.socket_path, "list_accounts", params).await?;
    
    serde_json::from_value(result.get("accounts").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get accounts by owner
#[tauri::command]
pub async fn public_account_get_by_owner(owner_id: String) -> Result<Vec<PublicAccount>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "owner_id": owner_id });
    let result = send_public_account_request(&config.socket_path, "get_accounts_by_owner", params).await?;
    
    serde_json::from_value(result.get("accounts").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Publish an article
#[tauri::command]
pub async fn public_account_publish_article(article: Article) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(article);
    let result = send_public_account_request(&config.socket_path, "publish_article", params).await?;
    
    result.get("article_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid article_id response".to_string())
}

/// Schedule an article for publishing
#[tauri::command]
pub async fn public_account_schedule_article(article: Article, scheduled_time: u64) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article": article, "scheduled_time": scheduled_time });
    let result = send_public_account_request(&config.socket_path, "schedule_article", params).await?;
    
    result.get("article_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid article_id response".to_string())
}

/// Process scheduled articles
#[tauri::command]
pub async fn public_account_process_scheduled_articles() -> Result<Vec<String>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(null);
    let result = send_public_account_request(&config.socket_path, "process_scheduled_articles", params).await?;
    
    serde_json::from_value(result.get("published_articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get scheduled articles
#[tauri::command]
pub async fn public_account_get_scheduled_articles(account_id: String) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_scheduled_articles", params).await?;
    
    serde_json::from_value(result.get("articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get article
#[tauri::command]
pub async fn public_account_get_article(article_id: String) -> Result<Article, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article_id": article_id });
    let result = send_public_account_request(&config.socket_path, "get_article", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}

/// List articles by account
#[tauri::command]
pub async fn public_account_list_articles(account_id: String) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "list_articles_by_account", params).await?;
    
    serde_json::from_value(result.get("articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Subscribe to a public account
#[tauri::command]
pub async fn public_account_subscribe(follower_id: String, account_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "follower_id": follower_id, "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "subscribe", params).await?;
    
    result.get("subscribed")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid subscribed response".to_string())
}

/// Unsubscribe from a public account
#[tauri::command]
pub async fn public_account_unsubscribe(follower_id: String, account_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "follower_id": follower_id, "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "unsubscribe", params).await?;
    
    result.get("unsubscribed")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid unsubscribed response".to_string())
}

/// Get followers of an account
#[tauri::command]
pub async fn public_account_get_followers(account_id: String) -> Result<Vec<Follower>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_followers", params).await?;
    
    serde_json::from_value(result.get("followers").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get user's subscriptions
#[tauri::command]
pub async fn public_account_get_subscriptions(user_id: String) -> Result<Vec<String>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_public_account_request(&config.socket_path, "get_user_subscriptions", params).await?;
    
    serde_json::from_value(result.get("subscriptions").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Record article view
#[tauri::command]
pub async fn public_account_record_view(article_id: String, user_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article_id": article_id, "user_id": user_id });
    let result = send_public_account_request(&config.socket_path, "record_view", params).await?;
    
    result.get("recorded")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid recorded response".to_string())
}

/// Like an article
#[tauri::command]
pub async fn public_account_like_article(article_id: String, user_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article_id": article_id, "user_id": user_id });
    let result = send_public_account_request(&config.socket_path, "like_article", params).await?;
    
    result.get("liked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid liked response".to_string())
}

/// Get account analytics
#[tauri::command]
pub async fn public_account_get_analytics(account_id: String) -> Result<AccountAnalytics, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_account_analytics", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}

/// Get article analytics
#[tauri::command]
pub async fn public_account_get_article_analytics(article_id: String) -> Result<ArticleAnalytics, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article_id": article_id });
    let result = send_public_account_request(&config.socket_path, "get_article_analytics", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}

/// Upload media to library
#[tauri::command]
pub async fn public_account_upload_media(media: MediaItem) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(media);
    let result = send_public_account_request(&config.socket_path, "upload_media", params).await?;
    
    result.get("media_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid media_id response".to_string())
}

/// Delete media from library
#[tauri::command]
pub async fn public_account_delete_media(media_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "media_id": media_id });
    let result = send_public_account_request(&config.socket_path, "delete_media", params).await?;
    
    result.get("deleted")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid deleted response".to_string())
}

/// Get media item
#[tauri::command]
pub async fn public_account_get_media(media_id: String) -> Result<MediaItem, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "media_id": media_id });
    let result = send_public_account_request(&config.socket_path, "get_media", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}

/// List media by account
#[tauri::command]
pub async fn public_account_list_media(account_id: String) -> Result<Vec<MediaItem>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "list_media_by_account", params).await?;
    
    serde_json::from_value(result.get("media").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// List media by type
#[tauri::command]
pub async fn public_account_list_media_by_type(account_id: String, media_type: String) -> Result<Vec<MediaItem>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id, "media_type": media_type });
    let result = send_public_account_request(&config.socket_path, "list_media_by_type", params).await?;
    
    serde_json::from_value(result.get("media").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Save article as draft
#[tauri::command]
pub async fn public_account_save_draft(article: Article) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(article);
    let result = send_public_account_request(&config.socket_path, "save_draft", params).await?;
    
    result.get("article_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid article_id response".to_string())
}

/// List drafts by account
#[tauri::command]
pub async fn public_account_list_drafts(account_id: String) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "list_drafts", params).await?;
    
    serde_json::from_value(result.get("drafts").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Delete draft
#[tauri::command]
pub async fn public_account_delete_draft(article_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "article_id": article_id });
    let result = send_public_account_request(&config.socket_path, "delete_draft", params).await?;
    
    result.get("deleted")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid deleted response".to_string())
}

/// Add custom menu item
#[tauri::command]
pub async fn public_account_add_menu_item(menu: CustomMenuItem) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(menu);
    let result = send_public_account_request(&config.socket_path, "add_menu_item", params).await?;
    
    result.get("menu_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid menu_id response".to_string())
}

/// Update custom menu item
#[tauri::command]
pub async fn public_account_update_menu_item(menu: CustomMenuItem) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(menu);
    let result = send_public_account_request(&config.socket_path, "update_menu_item", params).await?;
    
    result.get("updated")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid updated response".to_string())
}

/// Delete custom menu item
#[tauri::command]
pub async fn public_account_delete_menu_item(menu_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "menu_id": menu_id });
    let result = send_public_account_request(&config.socket_path, "delete_menu_item", params).await?;
    
    result.get("deleted")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid deleted response".to_string())
}

/// Get menu items by account
#[tauri::command]
pub async fn public_account_get_menu_items(account_id: String) -> Result<Vec<CustomMenuItem>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_menu_items", params).await?;
    
    serde_json::from_value(result.get("menu_items").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Send push notification
#[tauri::command]
pub async fn public_account_send_notification(notification: PushNotification) -> Result<String, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!(notification);
    let result = send_public_account_request(&config.socket_path, "send_notification", params).await?;
    
    result.get("notification_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid notification_id response".to_string())
}

/// Mark notification as read
#[tauri::command]
pub async fn public_account_mark_notification_read(notification_id: String) -> Result<bool, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "notification_id": notification_id });
    let result = send_public_account_request(&config.socket_path, "mark_notification_read", params).await?;
    
    result.get("marked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid marked response".to_string())
}

/// Get notifications by recipient
#[tauri::command]
pub async fn public_account_get_notifications(recipient_id: String) -> Result<Vec<PushNotification>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "recipient_id": recipient_id });
    let result = send_public_account_request(&config.socket_path, "get_notifications", params).await?;
    
    serde_json::from_value(result.get("notifications").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get unread notifications
#[tauri::command]
pub async fn public_account_get_unread_notifications(recipient_id: String) -> Result<Vec<PushNotification>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "recipient_id": recipient_id });
    let result = send_public_account_request(&config.socket_path, "get_unread_notifications", params).await?;
    
    serde_json::from_value(result.get("notifications").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Search public accounts
#[tauri::command]
pub async fn public_account_search(query: String, limit: Option<usize>) -> Result<Vec<PublicAccount>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ 
        "query": query,
        "limit": limit
    });
    let result = send_public_account_request(&config.socket_path, "search_accounts", params).await?;
    
    serde_json::from_value(result.get("accounts").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get trending articles
#[tauri::command]
pub async fn public_account_get_trending_articles(limit: Option<usize>) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "limit": limit });
    let result = send_public_account_request(&config.socket_path, "get_trending_articles", params).await?;
    
    serde_json::from_value(result.get("articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get articles by category
#[tauri::command]
pub async fn public_account_get_articles_by_category(category: String, limit: Option<usize>) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ 
        "category": category,
        "limit": limit
    });
    let result = send_public_account_request(&config.socket_path, "get_articles_by_category", params).await?;
    
    serde_json::from_value(result.get("articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Recommend articles for user
#[tauri::command]
pub async fn public_account_recommend_articles(user_id: String, limit: Option<usize>) -> Result<Vec<Article>, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "limit": limit
    });
    let result = send_public_account_request(&config.socket_path, "recommend_articles", params).await?;
    
    serde_json::from_value(result.get("articles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())
}

/// Get real-time analytics for an account
#[tauri::command]
pub async fn public_account_get_realtime_analytics(account_id: String) -> Result<AccountAnalytics, String> {
    let config = PublicAccountServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_public_account_request(&config.socket_path, "get_realtime_analytics", params).await?;
    
    serde_json::from_value(result)
        .map_err(|e| e.to_string())
}
