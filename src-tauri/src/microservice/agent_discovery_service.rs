//! Agent Discovery and Recommendation Service
//! 
//! This service provides intelligent agent discovery and recommendation
//! capabilities based on user interests, activities, and capabilities.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Discovery criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryCriteria {
    pub user_id: String,
    pub interests: Vec<String>, // User interests
    pub recent_activities: Vec<String>, // Recent user activities
    pub exclude_contacts: Vec<String>, // Exclude already added contacts
    pub exclude_self: bool,
    pub limit: Option<usize>,
}

/// Agent recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendation {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub node_id: String,
    pub status: String,
    pub match_score: f32, // 0.0 to 1.0
    pub match_reasons: Vec<String>, // Why this agent was recommended
    pub last_seen: u64,
}

/// Trending agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_type: String,
    pub popularity_score: f32, // Based on usage, interactions, etc.
    pub activity_count: u32,
    pub time_window: String, // "hour", "day", "week"
}

/// Configuration for Agent Discovery Service
#[derive(Debug, Clone)]
pub struct AgentDiscoveryServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for AgentDiscoveryServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_agent_discovery.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_agent_discovery");
        
        Self { socket_path, storage_dir }
    }
}

/// Agent Discovery Service
pub struct AgentDiscoveryService {
    config: AgentDiscoveryServiceConfig,
    agent_index: Arc<Mutex<HashMap<String, AgentInfo>>>, // agent_id -> agent info
    activity_tracker: Arc<Mutex<HashMap<String, u32>>>, // agent_id -> activity count
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

/// Agent information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    agent_id: String,
    agent_type: String,
    capabilities: Vec<String>,
    tags: Vec<String>,
    node_id: String,
    status: String,
    last_seen: u64,
    popularity: u32,
}

impl AgentDiscoveryService {
    pub fn new(config: AgentDiscoveryServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            agent_index: Arc::new(Mutex::new(HashMap::new())),
            activity_tracker: Arc::new(Mutex::new(HashMap::new())),
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
        let agent_index = Arc::clone(&self.agent_index);
        let activity_tracker = Arc::clone(&self.activity_tracker);
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
                                let agent_index = Arc::clone(&agent_index);
                                let activity_tracker = Arc::clone(&activity_tracker);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, agent_index, activity_tracker, node_id).await;
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

        println!("Agent Discovery Service started on {:?}", socket_path);
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

        println!("Agent Discovery Service stopped");
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

    /// Register agent for discovery
    #[allow(dead_code)]
    pub fn register_agent(&self, agent: AgentInfo) -> Result<(), String> {
        let agent_id = agent.agent_id.clone();
        let mut index = self.agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        index.insert(agent_id, agent);
        Ok(())
    }

    /// Update agent activity
    #[allow(dead_code)]
    pub fn update_agent_activity(&self, agent_id: String) -> Result<(), String> {
        let mut tracker = self.activity_tracker.lock().map_err(|e| format!("Lock error: {}", e))?;
        *tracker.entry(agent_id).or_insert(0) += 1;
        Ok(())
    }

    /// Discover agents based on criteria
    #[allow(dead_code)]
    pub fn discover_agents(&self, criteria: DiscoveryCriteria) -> Vec<AgentRecommendation> {
        let index = self.agent_index.lock().ok();
        let tracker = self.activity_tracker.lock().ok();
        
        if let (Some(index), Some(tracker)) = (index, tracker) {
            let mut recommendations: Vec<AgentRecommendation> = Vec::new();
            
            for (agent_id, agent_info) in index.iter() {
                // Skip if in exclude list
                if criteria.exclude_contacts.contains(agent_id) {
                    continue;
                }
                
                if criteria.exclude_self && agent_id == &criteria.user_id {
                    continue;
                }
                
                // Calculate match score
                let (score, reasons) = self.calculate_match_score(agent_info, &criteria);
                
                if score > 0.3 { // Minimum threshold
                    let _activity = tracker.get(agent_id).copied().unwrap_or(0);
                    recommendations.push(AgentRecommendation {
                        agent_id: agent_id.clone(),
                        agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                        agent_type: agent_info.agent_type.clone(),
                        capabilities: agent_info.capabilities.clone(),
                        node_id: agent_info.node_id.clone(),
                        status: agent_info.status.clone(),
                        match_score: score,
                        match_reasons: reasons,
                        last_seen: agent_info.last_seen,
                    });
                }
            }

        // Sort by match score
        recommendations.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(limit) = criteria.limit {
            recommendations.truncate(limit);
        }

        recommendations
    } else {
        Vec::new()
    }
}

    /// Get trending agents
    #[allow(dead_code)]
    pub fn get_trending_agents(&self, time_window: String, limit: Option<usize>) -> Vec<TrendingAgent> {
        let index = self.agent_index.lock().ok();
        let tracker = self.activity_tracker.lock().ok();
        
        if let (Some(index), Some(tracker)) = (index, tracker) {
            let mut trending: Vec<TrendingAgent> = Vec::new();
        
        for (agent_id, agent_info) in index.iter() {
            let activity = tracker.get(agent_id).copied().unwrap_or(0);
            let popularity = (activity as f32) / (agent_info.popularity as f32 + 1.0);
            
            if activity > 0 {
                trending.push(TrendingAgent {
                    agent_id: agent_id.clone(),
                    agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                    agent_type: agent_info.agent_type.clone(),
                    popularity_score: popularity,
                    activity_count: activity,
                    time_window: time_window.clone(),
                });
            }
        }

        // Sort by popularity
        trending.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(limit) = limit {
            trending.truncate(limit);
        }

        trending
    } else {
        Vec::new()
    }
}

    /// Calculate match score for an agent
    #[allow(dead_code)]
    fn calculate_match_score(&self, agent: &AgentInfo, criteria: &DiscoveryCriteria) -> (f32, Vec<String>) {
        calculate_match_score_static(agent, criteria)
    }

    /// Search agents by capability
    #[allow(dead_code)]
    pub fn search_by_capability(&self, capability: String, limit: Option<usize>) -> Vec<AgentRecommendation> {
        self.agent_index.lock()
            .map(|index| {
                let mut results: Vec<AgentRecommendation> = Vec::new();
        
                for (agent_id, agent_info) in index.iter() {
                    for cap in &agent_info.capabilities {
                        if cap.to_lowercase().contains(&capability.to_lowercase()) {
                            results.push(AgentRecommendation {
                                agent_id: agent_id.clone(),
                                agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                                agent_type: agent_info.agent_type.clone(),
                                capabilities: agent_info.capabilities.clone(),
                                node_id: agent_info.node_id.clone(),
                                status: agent_info.status.clone(),
                                match_score: 1.0,
                                match_reasons: vec![format!("Capability match: {}", cap)],
                                last_seen: agent_info.last_seen,
                            });
                            break;
                        }
                    }
                }

                if let Some(limit) = limit {
                    results.truncate(limit);
                }

                results
            })
            .unwrap_or_default()
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    agent_index: Arc<Mutex<HashMap<String, AgentInfo>>>,
    activity_tracker: Arc<Mutex<HashMap<String, u32>>>,
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
            "register_agent" => handle_register_agent(&params, &agent_index).await,
            "update_agent_activity" => handle_update_agent_activity(&params, &activity_tracker).await,
            "discover_agents" => handle_discover_agents(&params, &agent_index, &activity_tracker).await,
            "get_trending_agents" => handle_get_trending_agents(&params, &agent_index, &activity_tracker).await,
            "search_by_capability" => handle_search_by_capability(&params, &agent_index).await,
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

async fn handle_register_agent(
    params: &serde_json::Value,
    agent_index: &Arc<Mutex<HashMap<String, AgentInfo>>>,
) -> Result<serde_json::Value, String> {
    let agent: AgentInfo = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid agent: {}", e))?;
    
    let agent_id = agent.agent_id.clone();
    let mut guard = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(agent_id, agent);

    Ok(json!({
        "registered": true
    }))
}

async fn handle_update_agent_activity(
    params: &serde_json::Value,
    activity_tracker: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    
    let mut guard = activity_tracker.lock().map_err(|e| format!("Lock error: {}", e))?;
    *guard.entry(agent_id.to_string()).or_insert(0) += 1;

    Ok(json!({
        "updated": true
    }))
}

async fn handle_discover_agents(
    params: &serde_json::Value,
    agent_index: &Arc<Mutex<HashMap<String, AgentInfo>>>,
    activity_tracker: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let criteria: DiscoveryCriteria = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid criteria: {}", e))?;
    
    let index_guard = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let tracker_guard = activity_tracker.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut recommendations: Vec<AgentRecommendation> = Vec::new();
    
    for (agent_id, agent_info) in index_guard.iter() {
        // Skip if in exclude list
        if criteria.exclude_contacts.contains(agent_id) {
            continue;
        }
        
        if criteria.exclude_self && agent_id == &criteria.user_id {
            continue;
        }
        
        // Calculate match score
        let (score, reasons) = calculate_match_score_static(agent_info, &criteria);
        
        if score > 0.3 { // Minimum threshold
                let _activity = tracker_guard.get(agent_id).copied().unwrap_or(0);
            recommendations.push(AgentRecommendation {
                agent_id: agent_id.clone(),
                agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                agent_type: agent_info.agent_type.clone(),
                capabilities: agent_info.capabilities.clone(),
                node_id: agent_info.node_id.clone(),
                status: agent_info.status.clone(),
                match_score: score,
                match_reasons: reasons,
                last_seen: agent_info.last_seen,
            });
        }
    }

    drop(index_guard);
    drop(tracker_guard);

    // Sort by match score
    recommendations.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
    
    if let Some(limit) = criteria.limit {
        recommendations.truncate(limit);
    }

    Ok(json!({
        "recommendations": recommendations
    }))
}

async fn handle_get_trending_agents(
    params: &serde_json::Value,
    agent_index: &Arc<Mutex<HashMap<String, AgentInfo>>>,
    activity_tracker: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let time_window = params.get("time_window").and_then(|t| t.as_str()).unwrap_or("day");
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let index_guard = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let tracker_guard = activity_tracker.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut trending: Vec<TrendingAgent> = Vec::new();
    
    for (agent_id, agent_info) in index_guard.iter() {
        let activity = tracker_guard.get(agent_id).copied().unwrap_or(0);
        let popularity = (activity as f32) / (agent_info.popularity as f32 + 1.0);
        
        if activity > 0 {
            trending.push(TrendingAgent {
                agent_id: agent_id.clone(),
                agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                agent_type: agent_info.agent_type.clone(),
                popularity_score: popularity,
                activity_count: activity,
                time_window: time_window.to_string(),
            });
        }
    }

    drop(index_guard);
    drop(tracker_guard);

    // Sort by popularity
    trending.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap_or(std::cmp::Ordering::Equal));
    
    if let Some(limit) = limit {
        trending.truncate(limit);
    }

    Ok(json!({
        "trending": trending
    }))
}

async fn handle_search_by_capability(
    params: &serde_json::Value,
    agent_index: &Arc<Mutex<HashMap<String, AgentInfo>>>,
) -> Result<serde_json::Value, String> {
    let capability = params.get("capability").and_then(|c| c.as_str()).ok_or("Missing capability")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let index_guard = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let mut results: Vec<AgentRecommendation> = Vec::new();
    
    for (agent_id, agent_info) in index_guard.iter() {
        for cap in &agent_info.capabilities {
            if cap.to_lowercase().contains(&capability.to_lowercase()) {
                results.push(AgentRecommendation {
                    agent_id: agent_id.clone(),
                    agent_name: format!("Agent {}", agent_id.split('-').last().unwrap_or(agent_id)),
                    agent_type: agent_info.agent_type.clone(),
                    capabilities: agent_info.capabilities.clone(),
                    node_id: agent_info.node_id.clone(),
                    status: agent_info.status.clone(),
                    match_score: 1.0,
                    match_reasons: vec![format!("Capability match: {}", cap)],
                    last_seen: agent_info.last_seen,
                });
                break;
            }
        }
    }

    drop(index_guard);

    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "results": results
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

fn calculate_match_score_static(agent: &AgentInfo, criteria: &DiscoveryCriteria) -> (f32, Vec<String>) {
    let mut score = 0.0f32;
    let mut reasons = Vec::new();

    // Match by capabilities
    for interest in &criteria.interests {
        for capability in &agent.capabilities {
            if interest.to_lowercase() == capability.to_lowercase() || 
               capability.to_lowercase().contains(&interest.to_lowercase()) {
                score += 0.3;
                reasons.push(format!("Capability match: {}", capability));
                break;
            }
        }
    }

    // Match by tags
    for interest in &criteria.interests {
        for tag in &agent.tags {
            if interest.to_lowercase() == tag.to_lowercase() || 
               tag.to_lowercase().contains(&interest.to_lowercase()) {
                score += 0.2;
                reasons.push(format!("Tag match: {}", tag));
                break;
            }
        }
    }

    // Match by recent activities
    for activity in &criteria.recent_activities {
        for capability in &agent.capabilities {
            if activity.to_lowercase() == capability.to_lowercase() || 
               capability.to_lowercase().contains(&activity.to_lowercase()) {
                score += 0.25;
                reasons.push(format!("Activity match: {}", capability));
                break;
            }
        }
    }

    // Boost for online status
    if agent.status == "online" {
        score += 0.15;
        reasons.push("Agent is online".to_string());
    }

    // Cap score at 1.0
    score = score.min(1.0);

    (score, reasons)
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("agent_discovery_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_discover() {
        let config = AgentDiscoveryServiceConfig::default();
        let service = AgentDiscoveryService::new(config).expect("Failed to create service");
        
        let agent = AgentInfo {
            agent_id: "test-agent-1".to_string(),
            agent_type: "llm".to_string(),
            capabilities: vec!["text-generation".to_string(), "code-generation".to_string()],
            tags: vec!["ai".to_string(), "assistant".to_string()],
            node_id: "node-1".to_string(),
            status: "online".to_string(),
            last_seen: current_timestamp(),
            popularity: 100,
        };

        service.register_agent(agent).expect("Failed to register agent");
        
        let criteria = DiscoveryCriteria {
            user_id: "user-1".to_string(),
            interests: vec!["text-generation".to_string()],
            recent_activities: vec![],
            exclude_contacts: vec![],
            exclude_self: true,
            limit: Some(10),
        };

        let recommendations = service.discover_agents(criteria);
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].match_score > 0.3);
    }
}
