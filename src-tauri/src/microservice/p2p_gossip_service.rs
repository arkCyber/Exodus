//! P2P Gossip Service - State sync microservice inspired by iroh-gossip
//! 
//! This service provides publish-subscribe overlay network for state synchronization,
//! similar to iroh-gossip but adapted for Exodus microservice architecture.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Configuration for P2P Gossip Service
#[derive(Debug, Clone)]
pub struct P2pGossipConfig {
    pub socket_path: PathBuf,
}

impl Default for P2pGossipConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_p2p_gossip.sock");
        Self { socket_path }
    }
}

/// Gossip message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub topic: String,
    pub payload: serde_json::Value,
    pub from_node: String,
    pub timestamp: u64,
    pub id: String,
}

/// Subscription info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SubscriptionInfo {
    pub topic: String,
    pub subscribed_at: u64,
}

/// P2P Gossip Service
pub struct P2pGossipService {
    config: P2pGossipConfig,
    topics: Arc<Mutex<HashMap<String, Vec<GossipMessage>>>>,
    subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>, // topic -> subscriber IDs
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl P2pGossipService {
    pub fn new(config: P2pGossipConfig) -> Self {
        Self {
            config,
            topics: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let socket_path = self.config.socket_path.clone();
        let topics = Arc::clone(&self.topics);
        let subscriptions = Arc::clone(&self.subscriptions);
        let node_id = self.node_id.clone();
        let _running = Arc::clone(&self.running);
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let topics = Arc::clone(&topics);
                                let subscriptions = Arc::clone(&subscriptions);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, topics, subscriptions, node_id).await;
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

        println!("P2P Gossip Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("P2P Gossip Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Subscribe to a topic
    #[allow(dead_code)]
    pub fn subscribe(&self, topic: String, subscriber_id: String) -> Result<(), String> {
        let mut subs = self.subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
        subs.entry(topic).or_insert_with(HashSet::new).insert(subscriber_id);
        Ok(())
    }

    /// Unsubscribe from a topic
    #[allow(dead_code)]
    pub fn unsubscribe(&self, topic: String, subscriber_id: String) -> Result<(), String> {
        let mut subs = self.subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(subscribers) = subs.get_mut(&topic) {
            subscribers.remove(&subscriber_id);
            if subscribers.is_empty() {
                subs.remove(&topic);
            }
        }
        Ok(())
    }

    /// Publish a message to a topic
    #[allow(dead_code)]
    pub fn publish(&self, topic: String, payload: serde_json::Value) -> Result<String, String> {
        let message = GossipMessage {
            topic: topic.clone(),
            payload,
            from_node: self.node_id.clone(),
            timestamp: current_timestamp(),
            id: generate_message_id(),
        };

        let mut topics = self.topics.lock().map_err(|e| format!("Lock error: {}", e))?;
        topics.entry(topic).or_insert_with(Vec::new).push(message.clone());
        
        // Keep only last 100 messages per topic
        if let Some(messages) = topics.get_mut(&message.topic) {
            if messages.len() > 100 {
                messages.drain(0..messages.len() - 100);
            }
        }

        Ok(message.id)
    }

    /// Get messages for a topic
    #[allow(dead_code)]
    pub fn get_messages(&self, topic: String, limit: Option<usize>) -> Vec<GossipMessage> {
        let topics = self.topics.lock().unwrap();
        if let Some(messages) = topics.get(&topic) {
            let limit = limit.unwrap_or(50);
            if messages.len() > limit {
                messages[messages.len() - limit..].to_vec()
            } else {
                messages.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// List all active topics
    #[allow(dead_code)]
    pub fn list_topics(&self) -> Vec<String> {
        let topics = self.topics.lock().unwrap();
        topics.keys().cloned().collect()
    }

    /// Get subscribers for a topic
    #[allow(dead_code)]
    pub fn get_subscribers(&self, topic: String) -> Vec<String> {
        let subs = self.subscriptions.lock().unwrap();
        subs.get(&topic)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    topics: Arc<Mutex<HashMap<String, Vec<GossipMessage>>>>,
    subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>,
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
            "subscribe" => handle_subscribe(&params, &subscriptions).await,
            "unsubscribe" => handle_unsubscribe(&params, &subscriptions).await,
            "publish" => handle_publish(&params, &topics, &node_id).await,
            "get_messages" => handle_get_messages(&params, &topics).await,
            "list_topics" => handle_list_topics(&topics).await,
            "get_subscribers" => handle_get_subscribers(&params, &subscriptions).await,
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

async fn handle_subscribe(
    params: &serde_json::Value,
    subscriptions: &Arc<Mutex<HashMap<String, HashSet<String>>>>,
) -> Result<serde_json::Value, String> {
    let topic = params.get("topic").and_then(|t| t.as_str()).ok_or("Missing topic")?;
    let subscriber_id = params.get("subscriber_id")
        .and_then(|s| s.as_str())
        .ok_or("Missing subscriber_id")?;
    
    let mut subs = subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    subs.entry(topic.to_string()).or_insert_with(HashSet::new).insert(subscriber_id.to_string());

    Ok(json!({
        "subscribed": true,
        "topic": topic
    }))
}

async fn handle_unsubscribe(
    params: &serde_json::Value,
    subscriptions: &Arc<Mutex<HashMap<String, HashSet<String>>>>,
) -> Result<serde_json::Value, String> {
    let topic = params.get("topic").and_then(|t| t.as_str()).ok_or("Missing topic")?;
    let subscriber_id = params.get("subscriber_id")
        .and_then(|s| s.as_str())
        .ok_or("Missing subscriber_id")?;
    
    let mut subs = subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(subscribers) = subs.get_mut(topic) {
        subscribers.remove(subscriber_id);
        if subscribers.is_empty() {
            subs.remove(topic);
        }
    }

    Ok(json!({
        "unsubscribed": true,
        "topic": topic
    }))
}

async fn handle_publish(
    params: &serde_json::Value,
    topics: &Arc<Mutex<HashMap<String, Vec<GossipMessage>>>>,
    node_id: &str,
) -> Result<serde_json::Value, String> {
    let topic = params.get("topic").and_then(|t| t.as_str()).ok_or("Missing topic")?;
    let payload = params.get("payload").ok_or("Missing payload")?;
    
    let message = GossipMessage {
        topic: topic.to_string(),
        payload: payload.clone(),
        from_node: node_id.to_string(),
        timestamp: current_timestamp(),
        id: generate_message_id(),
    };

    let mut guard = topics.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.entry(topic.to_string()).or_insert_with(Vec::new).push(message.clone());
    
    // Keep only last 100 messages per topic
    if let Some(messages) = guard.get_mut(&message.topic) {
        if messages.len() > 100 {
            messages.drain(0..messages.len() - 100);
        }
    }

    Ok(json!({
        "published": true,
        "message_id": message.id,
        "timestamp": message.timestamp
    }))
}

async fn handle_get_messages(
    params: &serde_json::Value,
    topics: &Arc<Mutex<HashMap<String, Vec<GossipMessage>>>>,
) -> Result<serde_json::Value, String> {
    let topic = params.get("topic").and_then(|t| t.as_str()).ok_or("Missing topic")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = topics.lock().map_err(|e| format!("Lock error: {}", e))?;
    let messages = if let Some(msgs) = guard.get(topic) {
        let limit = limit.unwrap_or(50);
        if msgs.len() > limit {
            msgs[msgs.len() - limit..].to_vec()
        } else {
            msgs.clone()
        }
    } else {
        Vec::new()
    };

    Ok(json!({
        "messages": messages
    }))
}

async fn handle_list_topics(
    topics: &Arc<Mutex<HashMap<String, Vec<GossipMessage>>>>,
) -> Result<serde_json::Value, String> {
    let guard = topics.lock().map_err(|e| format!("Lock error: {}", e))?;
    let topic_list: Vec<String> = guard.keys().cloned().collect();
    
    Ok(json!({
        "topics": topic_list
    }))
}

async fn handle_get_subscribers(
    params: &serde_json::Value,
    subscriptions: &Arc<Mutex<HashMap<String, HashSet<String>>>>,
) -> Result<serde_json::Value, String> {
    let topic = params.get("topic").and_then(|t| t.as_str()).ok_or("Missing topic")?;
    
    let guard = subscriptions.lock().map_err(|e| format!("Lock error: {}", e))?;
    let subs = guard.get(topic)
        .map(|s| s.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    
    Ok(json!({
        "subscribers": subs
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
        .unwrap()
        .as_secs();
    format!("node_{:x}", timestamp)
}

fn generate_message_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("msg_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let config = P2pGossipConfig::default();
        let service = P2pGossipService::new(config);
        
        service.subscribe("test_topic".to_string(), "sub1".to_string()).unwrap();
        let subs = service.get_subscribers("test_topic".to_string());
        assert_eq!(subs.len(), 1);
        
        service.unsubscribe("test_topic".to_string(), "sub1".to_string()).unwrap();
        let subs = service.get_subscribers("test_topic".to_string());
        assert_eq!(subs.len(), 0);
    }

    #[tokio::test]
    async fn test_publish_get_messages() {
        let config = P2pGossipConfig::default();
        let service = P2pGossipService::new(config);
        
        let payload = json!({"key": "value"});
        service.publish("test_topic".to_string(), payload).unwrap();
        
        let messages = service.get_messages("test_topic".to_string(), None);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].topic, "test_topic");
    }

    #[tokio::test]
    async fn test_list_topics() {
        let config = P2pGossipConfig::default();
        let service = P2pGossipService::new(config);
        
        service.publish("topic1".to_string(), json!(null)).unwrap();
        service.publish("topic2".to_string(), json!(null)).unwrap();
        
        let topics = service.list_topics();
        assert_eq!(topics.len(), 2);
    }
}
