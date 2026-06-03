//! Social Feed Service - Timeline and social networking features
//! 
//! This service provides social media-like functionality including post creation,
//! timeline browsing, likes, comments, and social interactions.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Social post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub post_id: String,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub attachments: Vec<PostAttachment>,
    pub tags: Vec<String>,
    pub visibility: String, // "public", "friends", "private"
    pub location: Option<String>,
    pub mentions: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub like_count: u32,
    pub comment_count: u32,
    pub share_count: u32,
    pub public_account_id: Option<String>, // Associated public account ID
}

/// Post attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostAttachment {
    pub attachment_id: String,
    pub attachment_type: String, // "image", "video", "audio", "file"
    pub blob_hash: String, // Reference to p2p-blobs
    pub file_name: String,
    pub file_size: u64,
    pub thumbnail_hash: Option<String>,
    pub caption: Option<String>,
}

/// Social comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialComment {
    pub comment_id: String,
    pub post_id: String,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub parent_id: Option<String>, // For nested comments
    pub mentions: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub like_count: u32,
    pub reply_count: u32,
}

/// Social reaction (like, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialReaction {
    pub reaction_id: String,
    pub target_id: String, // post_id or comment_id
    pub target_type: String, // "post" or "comment"
    pub user_id: String,
    pub reaction_type: String, // "like", "love", "laugh", "wow", "sad", "angry"
    pub created_at: u64,
}

/// User follow relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FollowRelationship {
    pub follower_id: String,
    pub following_id: String,
    pub created_at: u64,
}

/// Configuration for Social Feed Service
#[derive(Debug, Clone)]
pub struct SocialFeedServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for SocialFeedServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_social_feed.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_social_feed");
        
        Self { socket_path, storage_dir }
    }
}

/// Social Feed Service
pub struct SocialFeedService {
    config: SocialFeedServiceConfig,
    posts: Arc<Mutex<HashMap<String, SocialPost>>>, // post_id -> post
    user_posts: Arc<Mutex<HashMap<String, Vec<String>>>>, // user_id -> post_ids
    comments: Arc<Mutex<HashMap<String, Vec<SocialComment>>>>, // post_id -> comments
    reactions: Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>, // target_id -> reactions
    follows: Arc<Mutex<HashMap<String, Vec<String>>>>, // follower_id -> following_ids
    followers: Arc<Mutex<HashMap<String, Vec<String>>>>, // following_id -> follower_ids
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl SocialFeedService {
    pub fn new(config: SocialFeedServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            posts: Arc::new(Mutex::new(HashMap::new())),
            user_posts: Arc::new(Mutex::new(HashMap::new())),
            comments: Arc::new(Mutex::new(HashMap::new())),
            reactions: Arc::new(Mutex::new(HashMap::new())),
            follows: Arc::new(Mutex::new(HashMap::new())),
            followers: Arc::new(Mutex::new(HashMap::new())),
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
        let posts = Arc::clone(&self.posts);
        let user_posts = Arc::clone(&self.user_posts);
        let comments = Arc::clone(&self.comments);
        let reactions = Arc::clone(&self.reactions);
        let follows = Arc::clone(&self.follows);
        let followers = Arc::clone(&self.followers);
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
                                let posts = Arc::clone(&posts);
                                let user_posts = Arc::clone(&user_posts);
                                let comments = Arc::clone(&comments);
                                let reactions = Arc::clone(&reactions);
                                let follows = Arc::clone(&follows);
                                let followers = Arc::clone(&followers);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, posts, user_posts, comments, reactions, follows, followers, node_id).await;
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

        println!("Social Feed Service started on {:?}", socket_path);
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

        println!("Social Feed Service stopped");
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

    /// Create a new post
    #[allow(dead_code)]
    pub fn create_post(&self, post: SocialPost) -> Result<(), String> {
        let post_id = post.post_id.clone();
        let author_id = post.author_id.clone();
        
        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        posts.insert(post_id.clone(), post.clone());
        drop(posts);

        let mut user_posts = self.user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        user_posts.entry(author_id).or_insert_with(Vec::new).push(post_id.clone());

        Ok(())
    }

    /// Update post
    #[allow(dead_code)]
    pub fn update_post(&self, post: SocialPost) -> Result<(), String> {
        let post_id = post.post_id.clone();
        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        posts.insert(post_id, post);
        Ok(())
    }

    /// Delete post
    #[allow(dead_code)]
    pub fn delete_post(&self, post_id: String) -> Result<(), String> {
        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        let post = posts.remove(&post_id);
        drop(posts);

        if let Some(post) = post {
            let mut user_posts = self.user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(post_ids) = user_posts.get_mut(&post.author_id) {
                post_ids.retain(|id| id != &post_id);
                if post_ids.is_empty() {
                    user_posts.remove(&post.author_id);
                }
            }
            
            let mut comments = self.comments.lock().map_err(|e| format!("Lock error: {}", e))?;
            comments.remove(&post_id);
            
            let mut reactions = self.reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
            reactions.remove(&post_id);
        }

        Ok(())
    }

    /// Get post
    #[allow(dead_code)]
    pub fn get_post(&self, post_id: String) -> Option<SocialPost> {
        let posts = self.posts.lock().ok()?;
        posts.get(&post_id).cloned()
    }

    /// Get user's posts
    #[allow(dead_code)]
    pub fn get_user_posts(&self, user_id: String, limit: Option<usize>) -> Vec<SocialPost> {
        let user_posts = self.user_posts.lock();
        let posts = self.posts.lock();
        
        if let (Ok(user_posts), Ok(posts)) = (user_posts, posts) {
            if let Some(post_ids) = user_posts.get(&user_id) {
                let mut post_list: Vec<SocialPost> = post_ids.iter()
                    .filter_map(|id| posts.get(id).cloned())
                    .collect();
                post_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                
                if let Some(limit) = limit {
                    post_list.truncate(limit);
                }
                post_list
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get timeline (posts from user and followings)
    #[allow(dead_code)]
    pub fn get_timeline(&self, user_id: String, limit: Option<usize>) -> Vec<SocialPost> {
        let follows = self.follows.lock();
        let user_posts = self.user_posts.lock();
        let posts = self.posts.lock();
        
        if let (Ok(follows), Ok(user_posts), Ok(posts)) = (follows, user_posts, posts) {
            let following_ids: Vec<String> = follows.get(&user_id).cloned().unwrap_or_default();
            
            let mut all_post_ids: Vec<String> = vec![user_id.clone()];
            all_post_ids.extend(following_ids.clone());
            
            let mut timeline: Vec<SocialPost> = Vec::new();
            for uid in all_post_ids {
                if let Some(post_ids) = user_posts.get(&uid) {
                    for pid in post_ids {
                        if let Some(post) = posts.get(pid) {
                            // Check visibility
                            if post.visibility == "public" || 
                               post.author_id == user_id || 
                               following_ids.contains(&post.author_id) {
                                timeline.push(post.clone());
                            }
                        }
                    }
                }
            }

            timeline.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
            if let Some(limit) = limit {
                timeline.truncate(limit);
            }
            timeline
        } else {
            Vec::new()
        }
    }

    /// Search posts by content or tags
    #[allow(dead_code)]
    pub fn search_posts(&self, query: String, limit: Option<usize>) -> Vec<SocialPost> {
        self.posts.lock()
            .map(|posts| {
                let query_lower = query.to_lowercase();
                
                let mut results: Vec<SocialPost> = posts.values()
                    .filter(|p| {
                        p.visibility == "public" &&
                        (p.content.to_lowercase().contains(&query_lower) ||
                         p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)))
                    })
                    .cloned()
                    .collect();

                results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                
                if let Some(limit) = limit {
                    results.truncate(limit);
                }

                results
            })
            .unwrap_or_default()
    }

    /// Link post to public account
    #[allow(dead_code)]
    pub fn link_to_public_account(&self, post_id: String, account_id: String) -> Result<(), String> {
        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post) = posts.get_mut(&post_id) {
            post.public_account_id = Some(account_id);
        }
        Ok(())
    }

    /// Unlink post from public account
    #[allow(dead_code)]
    pub fn unlink_from_public_account(&self, post_id: String) -> Result<(), String> {
        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post) = posts.get_mut(&post_id) {
            post.public_account_id = None;
        }
        Ok(())
    }

    /// Get posts linked to a public account
    #[allow(dead_code)]
    pub fn get_posts_by_public_account(&self, account_id: String, limit: Option<usize>) -> Vec<SocialPost> {
        self.posts.lock()
            .map(|posts| {
                let mut results: Vec<SocialPost> = posts.values()
                    .filter(|p| p.public_account_id.as_ref() == Some(&account_id))
                    .cloned()
                    .collect();

                results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                
                if let Some(limit) = limit {
                    results.truncate(limit);
                }

                results
            })
            .unwrap_or_default()
    }

    /// Add comment
    #[allow(dead_code)]
    pub fn add_comment(&self, comment: SocialComment) -> Result<(), String> {
        let post_id = comment.post_id.clone();
        
        let mut comments = self.comments.lock().map_err(|e| format!("Lock error: {}", e))?;
        comments.entry(post_id.clone()).or_insert_with(Vec::new).push(comment.clone());
        drop(comments);

        let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post) = posts.get_mut(&post_id) {
            post.comment_count += 1;
        }

        // Update parent comment reply count
        if let Some(parent_id) = &comment.parent_id {
            let mut comments_guard = self.comments.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(comment_list) = comments_guard.get_mut(&post_id) {
                if let Some(parent_comment) = comment_list.iter_mut().find(|c| c.comment_id == *parent_id) {
                    parent_comment.reply_count += 1;
                }
            }
        }

        Ok(())
    }

    /// Get post comments
    #[allow(dead_code)]
    pub fn get_comments(&self, post_id: String, limit: Option<usize>) -> Vec<SocialComment> {
        self.comments.lock()
            .map(|comments| {
                if let Some(comment_list) = comments.get(&post_id) {
                    let mut result = comment_list.clone();
                    result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                    
                    if let Some(limit) = limit {
                        result.truncate(limit);
                    }
                    
                    result
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    /// Delete comment
    #[allow(dead_code)]
    pub fn delete_comment(&self, post_id: String, comment_id: String) -> Result<(), String> {
        let mut comments = self.comments.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(comment_list) = comments.get_mut(&post_id) {
            comment_list.retain(|c| c.comment_id != comment_id);
        }
        Ok(())
    }

    /// Add reaction
    #[allow(dead_code)]
    pub fn add_reaction(&self, reaction: SocialReaction) -> Result<(), String> {
        let target_id = reaction.target_id.clone();
        let target_type = reaction.target_type.clone();
        let user_id = reaction.user_id.clone();
        
        // Remove existing reaction from same user
        let mut reactions = self.reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(reaction_list) = reactions.get_mut(&target_id) {
            reaction_list.retain(|r| !(r.user_id == user_id && r.target_type == target_type));
        }
        
        reactions.entry(target_id.clone()).or_insert_with(Vec::new).push(reaction.clone());
        
        let like_count = reactions.get(&target_id).map(|r| r.len() as u32).unwrap_or(0);
        drop(reactions);

        if target_type == "post" {
            let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(post) = posts.get_mut(&target_id) {
                post.like_count = like_count;
            }
        } else if target_type == "comment" {
            let mut comments = self.comments.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(comment_list) = comments.get_mut(&target_id) {
                if let Some(comment) = comment_list.iter_mut().find(|c| c.comment_id == target_id) {
                    comment.like_count = like_count;
                }
            }
        }

        Ok(())
    }

    /// Remove reaction
    #[allow(dead_code)]
    pub fn remove_reaction(&self, target_id: String, target_type: String, user_id: String) -> Result<(), String> {
        let mut reactions = self.reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(reaction_list) = reactions.get_mut(&target_id) {
            reaction_list.retain(|r| !(r.user_id == user_id && r.target_type == target_type));
        }
        
        let like_count = reactions.get(&target_id).map(|r| r.len() as u32).unwrap_or(0);
        drop(reactions);

        if target_type == "post" {
            let mut posts = self.posts.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(post) = posts.get_mut(&target_id) {
                post.like_count = like_count;
            }
        }

        Ok(())
    }

    /// Get reactions
    #[allow(dead_code)]
    pub fn get_reactions(&self, target_id: String) -> Vec<SocialReaction> {
        self.reactions.lock()
            .map(|reactions| reactions.get(&target_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Follow user
    #[allow(dead_code)]
    pub fn follow_user(&self, follower_id: String, following_id: String) -> Result<(), String> {
        let mut follows = self.follows.lock().map_err(|e| format!("Lock error: {}", e))?;
        follows.entry(follower_id.clone()).or_insert_with(Vec::new).push(following_id.clone());
        drop(follows);

        let mut followers = self.followers.lock().map_err(|e| format!("Lock error: {}", e))?;
        followers.entry(following_id).or_insert_with(Vec::new).push(follower_id);

        Ok(())
    }

    /// Unfollow user
    #[allow(dead_code)]
    pub fn unfollow_user(&self, follower_id: String, following_id: String) -> Result<(), String> {
        let mut follows = self.follows.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(following_list) = follows.get_mut(&follower_id) {
            following_list.retain(|id| id != &following_id);
        }
        drop(follows);

        let mut followers = self.followers.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(follower_list) = followers.get_mut(&following_id) {
            follower_list.retain(|id| id != &follower_id);
        }

        Ok(())
    }

    /// Get user's followings
    #[allow(dead_code)]
    pub fn get_followings(&self, user_id: String) -> Vec<String> {
        self.follows.lock()
            .map(|follows| follows.get(&user_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Get user's followers
    #[allow(dead_code)]
    pub fn get_followers(&self, user_id: String) -> Vec<String> {
        self.followers.lock()
            .map(|followers| followers.get(&user_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Check if following
    #[allow(dead_code)]
    pub fn is_following(&self, follower_id: String, following_id: String) -> bool {
        self.follows.lock()
            .map(|follows| {
                if let Some(following_list) = follows.get(&follower_id) {
                    following_list.contains(&following_id)
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    posts: Arc<Mutex<HashMap<String, SocialPost>>>,
    user_posts: Arc<Mutex<HashMap<String, Vec<String>>>>,
    comments: Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
    reactions: Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>,
    follows: Arc<Mutex<HashMap<String, Vec<String>>>>,
    followers: Arc<Mutex<HashMap<String, Vec<String>>>>,
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
            "create_post" => handle_create_post(&params, &posts, &user_posts).await,
            "update_post" => handle_update_post(&params, &posts).await,
            "delete_post" => handle_delete_post(&params, &posts, &user_posts, &comments, &reactions).await,
            "get_post" => handle_get_post(&params, &posts).await,
            "get_user_posts" => handle_get_user_posts(&params, &user_posts, &posts).await,
            "get_timeline" => handle_get_timeline(&params, &follows, &user_posts, &posts).await,
            "search_posts" => handle_search_posts(&params, &posts).await,
            "link_to_public_account" => handle_link_to_public_account(&params, &posts).await,
            "unlink_from_public_account" => handle_unlink_from_public_account(&params, &posts).await,
            "get_posts_by_public_account" => handle_get_posts_by_public_account(&params, &posts).await,
            "add_comment" => handle_add_comment(&params, &comments, &posts).await,
            "get_comments" => handle_get_comments(&params, &comments).await,
            "delete_comment" => handle_delete_comment(&params, &comments).await,
            "add_reaction" => handle_add_reaction(&params, &reactions, &posts, &comments).await,
            "remove_reaction" => handle_remove_reaction(&params, &reactions, &posts).await,
            "get_reactions" => handle_get_reactions(&params, &reactions).await,
            "follow_user" => handle_follow_user(&params, &follows, &followers).await,
            "unfollow_user" => handle_unfollow_user(&params, &follows, &followers).await,
            "get_followings" => handle_get_followings(&params, &follows).await,
            "get_followers" => handle_get_followers(&params, &followers).await,
            "is_following" => handle_is_following(&params, &follows).await,
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

async fn handle_create_post(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
    user_posts: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let post: SocialPost = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid post: {}", e))?;
    
    let post_id = post.post_id.clone();
    let author_id = post.author_id.clone();
    
    let mut posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    posts_guard.insert(post_id.clone(), post.clone());
    drop(posts_guard);

    let mut user_guard = user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    user_guard.entry(author_id).or_insert_with(Vec::new).push(post_id.clone());

    Ok(json!({
        "post_id": post_id
    }))
}

async fn handle_update_post(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let post: SocialPost = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid post: {}", e))?;
    
    let post_id = post.post_id.clone();
    let mut guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(post_id, post);

    Ok(json!({
        "updated": true
    }))
}

async fn handle_delete_post(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
    user_posts: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    comments: &Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
    reactions: &Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    
    let mut posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let post = posts_guard.remove(post_id);
    drop(posts_guard);

    if let Some(post) = post {
        let mut user_guard = user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post_ids) = user_guard.get_mut(&post.author_id) {
            post_ids.retain(|id| id != post_id);
            if post_ids.is_empty() {
                user_guard.remove(&post.author_id);
            }
        }
        
        let mut comments_guard = comments.lock().map_err(|e| format!("Lock error: {}", e))?;
        comments_guard.remove(post_id);
        
        let mut reactions_guard = reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
        reactions_guard.remove(post_id);
    }

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_get_post(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    
    let guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(post_id)
        .map(|p| json!(p))
        .ok_or_else(|| "Post not found".to_string())
}

async fn handle_get_user_posts(
    params: &serde_json::Value,
    user_posts: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let user_guard = user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(post_ids) = user_guard.get(user_id) {
        let mut post_list: Vec<SocialPost> = post_ids.iter()
            .filter_map(|id| posts_guard.get(id).cloned())
            .collect();
        post_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        if let Some(limit) = limit {
            post_list.truncate(limit);
        }
        
        return Ok(json!({ "posts": post_list }));
    }

    Ok(json!({
        "posts": Vec::<SocialPost>::new()
    }))
}

async fn handle_get_timeline(
    params: &serde_json::Value,
    follows: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    user_posts: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let follows_guard = follows.lock().map_err(|e| format!("Lock error: {}", e))?;
    let following_ids: Vec<String> = follows_guard.get(user_id).cloned().unwrap_or_default();
    drop(follows_guard);

    let user_posts_guard = user_posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut all_post_ids: Vec<String> = vec![user_id.to_string()];
    all_post_ids.extend(following_ids.clone());
    
    let mut timeline: Vec<SocialPost> = Vec::new();
    for uid in all_post_ids {
        if let Some(post_ids) = user_posts_guard.get(&uid) {
            for pid in post_ids {
                if let Some(post) = posts_guard.get(pid) {
                    if post.visibility == "public" || 
                       post.author_id == user_id || 
                       following_ids.contains(&post.author_id) {
                        timeline.push(post.clone());
                    }
                }
            }
        }
    }

    timeline.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    if let Some(limit) = limit {
        timeline.truncate(limit);
    }

    Ok(json!({
        "posts": timeline
    }))
}

async fn handle_search_posts(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let query = params.get("query").and_then(|q| q.as_str()).ok_or("Missing query")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<SocialPost> = guard.values()
        .filter(|p| {
            p.visibility == "public" &&
            (p.content.to_lowercase().contains(&query_lower) ||
             p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)))
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "posts": results
    }))
}

async fn handle_link_to_public_account(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let mut guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(post) = guard.get_mut(post_id) {
        post.public_account_id = Some(account_id.to_string());
    }

    Ok(json!({
        "linked": true
    }))
}

async fn handle_unlink_from_public_account(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    
    let mut guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(post) = guard.get_mut(post_id) {
        post.public_account_id = None;
    }

    Ok(json!({
        "unlinked": true
    }))
}

async fn handle_get_posts_by_public_account(
    params: &serde_json::Value,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut results: Vec<SocialPost> = guard.values()
        .filter(|p| p.public_account_id.as_ref() == Some(&account_id.to_string()))
        .cloned()
        .collect();

    results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "posts": results
    }))
}

async fn handle_add_comment(
    params: &serde_json::Value,
    comments: &Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let comment: SocialComment = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid comment: {}", e))?;
    
    let post_id = comment.post_id.clone();
    
    let mut comments_guard = comments.lock().map_err(|e| format!("Lock error: {}", e))?;
    comments_guard.entry(post_id.clone()).or_insert_with(Vec::new).push(comment.clone());
    drop(comments_guard);

    let mut posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(post) = posts_guard.get_mut(&post_id) {
        post.comment_count += 1;
    }

    Ok(json!({
        "comment_id": comment.comment_id
    }))
}

async fn handle_get_comments(
    params: &serde_json::Value,
    comments: &Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = comments.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(comment_list) = guard.get(post_id) {
        let mut result = comment_list.clone();
        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(json!({
            "comments": result
        }))
    } else {
        Ok(json!({
            "comments": Vec::<SocialComment>::new()
        }))
    }
}

async fn handle_delete_comment(
    params: &serde_json::Value,
    comments: &Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
) -> Result<serde_json::Value, String> {
    let post_id = params.get("post_id").and_then(|p| p.as_str()).ok_or("Missing post_id")?;
    let comment_id = params.get("comment_id").and_then(|c| c.as_str()).ok_or("Missing comment_id")?;
    
    let mut guard = comments.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(comment_list) = guard.get_mut(post_id) {
        comment_list.retain(|c| c.comment_id != comment_id);
    }

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_add_reaction(
    params: &serde_json::Value,
    reactions: &Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
    _comments: &Arc<Mutex<HashMap<String, Vec<SocialComment>>>>,
) -> Result<serde_json::Value, String> {
    let reaction: SocialReaction = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid reaction: {}", e))?;
    
    let target_id = reaction.target_id.clone();
    let target_type = reaction.target_type.clone();
    let user_id = reaction.user_id.clone();
    
    let mut reactions_guard = reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(reaction_list) = reactions_guard.get_mut(&target_id) {
        reaction_list.retain(|r| !(r.user_id == user_id && r.target_type == target_type));
    }
    reactions_guard.entry(target_id.clone()).or_insert_with(Vec::new).push(reaction.clone());
    
    let like_count = reactions_guard.get(&target_id).map(|r| r.len() as u32).unwrap_or(0);
    drop(reactions_guard);

    if target_type == "post" {
        let mut posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post) = posts_guard.get_mut(&target_id) {
            post.like_count = like_count;
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_remove_reaction(
    params: &serde_json::Value,
    reactions: &Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>,
    posts: &Arc<Mutex<HashMap<String, SocialPost>>>,
) -> Result<serde_json::Value, String> {
    let target_id = params.get("target_id").and_then(|t| t.as_str()).ok_or("Missing target_id")?;
    let target_type = params.get("target_type").and_then(|t| t.as_str()).ok_or("Missing target_type")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let mut guard = reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(reaction_list) = guard.get_mut(target_id) {
        reaction_list.retain(|r| !(r.user_id == user_id && r.target_type == target_type));
    }
    let like_count = guard.get(target_id).map(|r| r.len() as u32).unwrap_or(0);
    drop(guard);

    if target_type == "post" {
        let mut posts_guard = posts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(post) = posts_guard.get_mut(target_id) {
            post.like_count = like_count;
        }
    }

    Ok(json!({
        "removed": true
    }))
}

async fn handle_get_reactions(
    params: &serde_json::Value,
    reactions: &Arc<Mutex<HashMap<String, Vec<SocialReaction>>>>,
) -> Result<serde_json::Value, String> {
    let target_id = params.get("target_id").and_then(|t| t.as_str()).ok_or("Missing target_id")?;
    
    let guard = reactions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let reaction_list = guard.get(target_id).cloned().unwrap_or_default();

    Ok(json!({
        "reactions": reaction_list
    }))
}

async fn handle_follow_user(
    params: &serde_json::Value,
    follows: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    followers: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let follower_id = params.get("follower_id").and_then(|f| f.as_str()).ok_or("Missing follower_id")?;
    let following_id = params.get("following_id").and_then(|f| f.as_str()).ok_or("Missing following_id")?;
    
    let mut follows_guard = follows.lock().map_err(|e| format!("Lock error: {}", e))?;
    follows_guard.entry(follower_id.to_string()).or_insert_with(Vec::new).push(following_id.to_string());
    drop(follows_guard);

    let mut followers_guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    followers_guard.entry(following_id.to_string()).or_insert_with(Vec::new).push(follower_id.to_string());

    Ok(json!({
        "followed": true
    }))
}

async fn handle_unfollow_user(
    params: &serde_json::Value,
    follows: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    followers: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let follower_id = params.get("follower_id").and_then(|f| f.as_str()).ok_or("Missing follower_id")?;
    let following_id = params.get("following_id").and_then(|f| f.as_str()).ok_or("Missing following_id")?;
    
    let mut follows_guard = follows.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(following_list) = follows_guard.get_mut(follower_id) {
        following_list.retain(|id| id != following_id);
    }
    drop(follows_guard);

    let mut followers_guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(follower_list) = followers_guard.get_mut(following_id) {
        follower_list.retain(|id| id != follower_id);
    }

    Ok(json!({
        "unfollowed": true
    }))
}

async fn handle_get_followings(
    params: &serde_json::Value,
    follows: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let guard = follows.lock().map_err(|e| format!("Lock error: {}", e))?;
    let following_list = guard.get(user_id).cloned().unwrap_or_default();

    Ok(json!({
        "followings": following_list
    }))
}

async fn handle_get_followers(
    params: &serde_json::Value,
    followers: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let guard = followers.lock().map_err(|e| format!("Lock error: {}", e))?;
    let follower_list = guard.get(user_id).cloned().unwrap_or_default();

    Ok(json!({
        "followers": follower_list
    }))
}

async fn handle_is_following(
    params: &serde_json::Value,
    follows: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let follower_id = params.get("follower_id").and_then(|f| f.as_str()).ok_or("Missing follower_id")?;
    let following_id = params.get("following_id").and_then(|f| f.as_str()).ok_or("Missing following_id")?;
    
    let guard = follows.lock().map_err(|e| format!("Lock error: {}", e))?;
    let is_following = if let Some(following_list) = guard.get(follower_id) {
        following_list.contains(&following_id.to_string())
    } else {
        false
    };

    Ok(json!({
        "is_following": is_following
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("social_feed_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_post() {
        let config = SocialFeedServiceConfig::default();
        let service = SocialFeedService::new(config).expect("Failed to create service");
        
        let post = SocialPost {
            post_id: "post-1".to_string(),
            author_id: "user-1".to_string(),
            author_name: "User 1".to_string(),
            author_avatar: None,
            content: "Hello world!".to_string(),
            attachments: vec![],
            tags: vec![],
            visibility: "public".to_string(),
            location: None,
            mentions: vec![],
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            like_count: 0,
            comment_count: 0,
            share_count: 0,
            public_account_id: None,
        };

        service.create_post(post).expect("Failed to create post");
        let retrieved = service.get_post("post-1".to_string()).expect("Expected post");
        assert_eq!(retrieved.content, "Hello world!");
    }

    #[test]
    fn test_follow_and_get_followings() {
        let config = SocialFeedServiceConfig::default();
        let service = SocialFeedService::new(config).expect("Failed to create service");
        
        service.follow_user("user-1".to_string(), "user-2".to_string()).expect("Failed to follow user");
        let followings = service.get_followings("user-1".to_string());
        assert_eq!(followings.len(), 1);
        assert_eq!(followings[0], "user-2");
    }
}
