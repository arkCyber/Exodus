//! Tauri commands for Social Feed Service
//! 
//! These commands allow the frontend to interact with the Social Feed Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{SocialFeedService, SocialFeedServiceConfig, SocialPost, SocialComment, SocialReaction};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Social Feed Service instance
pub struct ManagedSocialFeedService {
    service: Arc<SocialFeedService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedSocialFeedService {
    pub fn new(service: SocialFeedService) -> Self {
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

/// Send JSON-RPC request to Social Feed Service
async fn send_social_feed_request(
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
        .map_err(|e| format!("Failed to connect to Social Feed Service: {}", e))?;

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
        return Err(format!("Social Feed Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Social Feed Service
#[tauri::command]
pub async fn social_feed_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = SocialFeedService::new(config)
        .map_err(|e| format!("Failed to create Social Feed Service: {}", e))?;
    
    let managed = ManagedSocialFeedService::new(service);
    managed.start().await?;
    
    let _ = app.emit("social-feed-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Social Feed Service
#[tauri::command]
pub async fn social_feed_service_stop() -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let service = SocialFeedService::new(config)
        .map_err(|e| format!("Failed to create Social Feed Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a new post
#[tauri::command]
pub async fn social_post_create(post: SocialPost) -> Result<String, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!(post);
    let result = send_social_feed_request(&config.socket_path, "create_post", params).await?;
    
    result.get("post_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid post_id response".to_string())
}

/// Update post
#[tauri::command]
pub async fn social_post_update(post: SocialPost) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!(post);
    send_social_feed_request(&config.socket_path, "update_post", params).await?;
    Ok(())
}

/// Delete post
#[tauri::command]
pub async fn social_post_delete(post_id: String) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "post_id": post_id });
    send_social_feed_request(&config.socket_path, "delete_post", params).await?;
    Ok(())
}

/// Get post
#[tauri::command]
pub async fn social_post_get(post_id: String) -> Result<SocialPost, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "post_id": post_id });
    let result = send_social_feed_request(&config.socket_path, "get_post", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse post: {}", e))
}

/// Get user's posts
#[tauri::command]
pub async fn social_post_get_user(user_id: String, limit: Option<usize>) -> Result<Vec<SocialPost>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "limit": limit
    });
    let result = send_social_feed_request(&config.socket_path, "get_user_posts", params).await?;
    
    let posts = result.get("posts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid posts response".to_string())?;
    
    posts.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get timeline
#[tauri::command]
pub async fn social_feed_get_timeline(user_id: String, limit: Option<usize>) -> Result<Vec<SocialPost>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "limit": limit
    });
    let result = send_social_feed_request(&config.socket_path, "get_timeline", params).await?;
    
    let posts = result.get("posts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid posts response".to_string())?;
    
    posts.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Search posts
#[tauri::command]
pub async fn social_post_search(query: String, limit: Option<usize>) -> Result<Vec<SocialPost>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "query": query,
        "limit": limit
    });
    let result = send_social_feed_request(&config.socket_path, "search_posts", params).await?;
    
    let posts = result.get("posts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid posts response".to_string())?;
    
    posts.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Link post to public account
#[tauri::command]
pub async fn social_post_link_to_public_account(post_id: String, account_id: String) -> Result<bool, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "post_id": post_id,
        "account_id": account_id
    });
    let result = send_social_feed_request(&config.socket_path, "link_to_public_account", params).await?;
    
    result.get("linked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid linked response".to_string())
}

/// Unlink post from public account
#[tauri::command]
pub async fn social_post_unlink_from_public_account(post_id: String) -> Result<bool, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "post_id": post_id });
    let result = send_social_feed_request(&config.socket_path, "unlink_from_public_account", params).await?;
    
    result.get("unlinked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid unlinked response".to_string())
}

/// Get posts linked to a public account
#[tauri::command]
pub async fn social_post_get_by_public_account(account_id: String, limit: Option<usize>) -> Result<Vec<SocialPost>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "account_id": account_id,
        "limit": limit
    });
    let result = send_social_feed_request(&config.socket_path, "get_posts_by_public_account", params).await?;
    
    let posts = result.get("posts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid posts response".to_string())?;
    
    posts.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Add comment
#[tauri::command]
pub async fn social_comment_add(comment: SocialComment) -> Result<String, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!(comment);
    let result = send_social_feed_request(&config.socket_path, "add_comment", params).await?;
    
    result.get("comment_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid comment_id response".to_string())
}

/// Get post comments
#[tauri::command]
pub async fn social_comment_get(post_id: String, limit: Option<usize>) -> Result<Vec<SocialComment>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "post_id": post_id,
        "limit": limit
    });
    let result = send_social_feed_request(&config.socket_path, "get_comments", params).await?;
    
    let comments = result.get("comments")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid comments response".to_string())?;
    
    comments.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Delete comment
#[tauri::command]
pub async fn social_comment_delete(post_id: String, comment_id: String) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "post_id": post_id,
        "comment_id": comment_id
    });
    send_social_feed_request(&config.socket_path, "delete_comment", params).await?;
    Ok(())
}

/// Add reaction
#[tauri::command]
pub async fn social_reaction_add(reaction: SocialReaction) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!(reaction);
    send_social_feed_request(&config.socket_path, "add_reaction", params).await?;
    Ok(())
}

/// Remove reaction
#[tauri::command]
pub async fn social_reaction_remove(target_id: String, target_type: String, user_id: String) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "target_id": target_id,
        "target_type": target_type,
        "user_id": user_id
    });
    send_social_feed_request(&config.socket_path, "remove_reaction", params).await?;
    Ok(())
}

/// Get reactions
#[tauri::command]
pub async fn social_reaction_get(target_id: String) -> Result<Vec<SocialReaction>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "target_id": target_id });
    let result = send_social_feed_request(&config.socket_path, "get_reactions", params).await?;
    
    let reactions = result.get("reactions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid reactions response".to_string())?;
    
    reactions.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Follow user
#[tauri::command]
pub async fn social_user_follow(follower_id: String, following_id: String) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "follower_id": follower_id,
        "following_id": following_id
    });
    send_social_feed_request(&config.socket_path, "follow_user", params).await?;
    Ok(())
}

/// Unfollow user
#[tauri::command]
pub async fn social_user_unfollow(follower_id: String, following_id: String) -> Result<(), String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "follower_id": follower_id,
        "following_id": following_id
    });
    send_social_feed_request(&config.socket_path, "unfollow_user", params).await?;
    Ok(())
}

/// Get user's followings
#[tauri::command]
pub async fn social_user_get_followings(user_id: String) -> Result<Vec<String>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_social_feed_request(&config.socket_path, "get_followings", params).await?;
    
    result.get("followings")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid followings response".to_string())
}

/// Get user's followers
#[tauri::command]
pub async fn social_user_get_followers(user_id: String) -> Result<Vec<String>, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_social_feed_request(&config.socket_path, "get_followers", params).await?;
    
    result.get("followers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid followers response".to_string())
}

/// Check if following
#[tauri::command]
pub async fn social_user_is_following(follower_id: String, following_id: String) -> Result<bool, String> {
    let config = SocialFeedServiceConfig::default();
    let params = json!({ 
        "follower_id": follower_id,
        "following_id": following_id
    });
    let result = send_social_feed_request(&config.socket_path, "is_following", params).await?;
    
    result.get("is_following")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid is_following response".to_string())
}
