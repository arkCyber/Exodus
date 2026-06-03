//! AI Agent Communication Service - Agent identity and messaging
//! 
//! This service provides AI agent registration, discovery, and communication
//! using the p2p-gossip service for distributed agent messaging.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// AI Agent identity and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub agent_type: String, // "llm", "vision", "audio", "multimodal", etc.
    pub deployment_type: String, // "service" or "personal_assistant"
    pub name: String,
    pub capabilities: Vec<String>, // ["text-generation", "image-analysis", "code-generation", etc.]
    pub model_id: Option<String>, // Reference to AI model
    pub node_id: String,
    pub digit_id: Option<String>, // 12-digit number for personal assistants
    pub status: String, // "online", "busy", "offline"
    pub last_seen: u64,
    pub metadata: serde_json::Value,
    pub public_account_id: Option<String>, // Associated public account ID
}

/// Agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: String,
    pub from_agent: String,
    pub to_agent: Option<String>, // None = broadcast
    pub message_type: String, // "request", "response", "notification", "command"
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub priority: u8, // 0-255, higher = more urgent
}

/// Agent presence announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPresence {
    pub agent_id: String,
    pub status: String,
    pub timestamp: u64,
}

/// Configuration for AI Agent Service
#[derive(Debug, Clone)]
pub struct AiAgentServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for AiAgentServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_ai_agent.sock");
        Self { socket_path }
    }
}

/// AI Agent Service
pub struct AiAgentService {
    config: AiAgentServiceConfig,
    agents: Arc<Mutex<HashMap<String, AgentIdentity>>>, // agent_id -> identity
    messages: Arc<Mutex<Vec<AgentMessage>>>, // message history
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl AiAgentService {
    pub fn new(config: AiAgentServiceConfig) -> Self {
        Self {
            config,
            agents: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
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
        let agents = Arc::clone(&self.agents);
        let messages = Arc::clone(&self.messages);
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
                                let agents = Arc::clone(&agents);
                                let messages = Arc::clone(&messages);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, agents, messages, node_id).await;
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

        println!("AI Agent Service started on {:?}", socket_path);
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

        println!("AI Agent Service stopped");
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

    /// Register an agent
    #[allow(dead_code)]
    pub fn register_agent(&self, mut agent: AgentIdentity) -> Result<(), String> {
        // Auto-generate 12-digit number for personal assistants
        if agent.deployment_type == "personal_assistant" && agent.digit_id.is_none() {
            if let Some(digit_id) = self.generate_digit_for_node(&agent.node_id) {
                agent.digit_id = Some(digit_id);
            }
        }
        
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        agents.insert(agent.agent_id.clone(), agent);
        Ok(())
    }

    /// Generate 12-digit number for a node
    fn generate_digit_for_node(&self, node_id: &str) -> Option<String> {
        // Use crypto service to extract 12-digit number from node_id
        // For now, generate a simple hash-based 12-digit number
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        node_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        let digit = hash % 1_000_000_000_000u64;
        Some(format!("{:012}", digit))
    }

    /// Register agent with explicit 12-digit number
    #[allow(dead_code)]
    pub fn register_agent_with_digit(&self, agent: AgentIdentity, digit_id: String) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let mut agent = agent;
        agent.digit_id = Some(digit_id);
        agents.insert(agent.agent_id.clone(), agent);
        Ok(())
    }

    /// Unregister an agent
    #[allow(dead_code)]
    pub fn unregister_agent(&self, agent_id: String) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        agents.remove(&agent_id);
        Ok(())
    }

    /// Get agent identity
    #[allow(dead_code)]
    pub fn get_agent(&self, agent_id: String) -> Option<AgentIdentity> {
        let agents = self.agents.lock().ok()?;
        agents.get(&agent_id).cloned()
    }

    /// List all agents
    #[allow(dead_code)]
    pub fn list_agents(&self) -> Vec<AgentIdentity> {
        self.agents.lock()
            .map(|agents| agents.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Find agents by capability
    #[allow(dead_code)]
    pub fn find_by_capability(&self, capability: String) -> Vec<AgentIdentity> {
        self.agents.lock()
            .map(|agents| agents.values()
            .filter(|a| a.capabilities.contains(&capability))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Find agents by type
    #[allow(dead_code)]
    pub fn find_by_type(&self, agent_type: String) -> Vec<AgentIdentity> {
        self.agents.lock()
            .map(|agents| agents.values()
            .filter(|a| a.agent_type == agent_type)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Link agent to public account
    #[allow(dead_code)]
    pub fn link_to_public_account(&self, agent_id: String, account_id: String) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.public_account_id = Some(account_id);
        }
        Ok(())
    }

    /// Unlink agent from public account
    #[allow(dead_code)]
    pub fn unlink_from_public_account(&self, agent_id: String) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.public_account_id = None;
        }
        Ok(())
    }

    /// Get agents linked to a public account
    #[allow(dead_code)]
    pub fn get_agents_by_public_account(&self, account_id: String) -> Vec<AgentIdentity> {
        self.agents.lock()
            .map(|agents| agents.values()
            .filter(|a| a.public_account_id.as_ref() == Some(&account_id))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Update agent status
    #[allow(dead_code)]
    pub fn update_agent_status(&self, agent_id: String, status: String) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.status = status;
            agent.last_seen = current_timestamp();
        }
        Ok(())
    }

    /// Send message to agent
    #[allow(dead_code)]
    pub fn send_message(&self, message: AgentMessage) -> Result<(), String> {
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        messages.push(message.clone());
        
        // Keep only last 1000 messages
        let len = messages.len();
        if len > 1000 {
            messages.drain(0..len - 1000);
        }
        
        Ok(())
    }

    /// Get messages for an agent
    #[allow(dead_code)]
    pub fn get_messages(&self, agent_id: String, limit: Option<usize>) -> Vec<AgentMessage> {
        self.messages.lock()
            .map(|messages| {
                let limit = limit.unwrap_or(50);
                
                messages.iter()
                    .filter(|m| {
                        m.from_agent == agent_id || 
                        m.to_agent.as_ref().map(|t| t == &agent_id).unwrap_or(false) ||
                        m.to_agent.is_none() // broadcast
                    })
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Broadcast presence update
    #[allow(dead_code)]
    pub fn broadcast_presence(&self, presence: AgentPresence) -> Result<(), String> {
        let mut agents = self.agents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(agent) = agents.get_mut(&presence.agent_id) {
            agent.status = presence.status.clone();
            agent.last_seen = presence.timestamp;
        }
        Ok(())
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    agents: Arc<Mutex<HashMap<String, AgentIdentity>>>,
    messages: Arc<Mutex<Vec<AgentMessage>>>,
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
            "register_agent" => handle_register_agent(&params, &agents).await,
            "unregister_agent" => handle_unregister_agent(&params, &agents).await,
            "get_agent" => handle_get_agent(&params, &agents).await,
            "list_agents" => handle_list_agents(&agents).await,
            "find_agents_by_capability" => handle_find_agents_by_capability(&params, &agents).await,
            "find_agents_by_type" => handle_find_agents_by_type(&params, &agents).await,
            "update_agent_status" => handle_update_agent_status(&params, &agents).await,
            "link_to_public_account" => handle_link_to_public_account(&params, &agents).await,
            "unlink_from_public_account" => handle_unlink_from_public_account(&params, &agents).await,
            "get_agents_by_public_account" => handle_get_agents_by_public_account(&params, &agents).await,
            "send_message" => handle_send_message(&params, &messages).await,
            "get_messages" => handle_get_messages(&params, &messages).await,
            "broadcast_presence" => handle_broadcast_presence(&params, &agents).await,
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
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let mut agent: AgentIdentity = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid agent: {}", e))?;
    
    // Auto-generate 12-digit number for personal assistants
    if agent.deployment_type == "personal_assistant" && agent.digit_id.is_none() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        agent.node_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        let digit = hash % 1_000_000_000_000u64;
        agent.digit_id = Some(format!("{:012}", digit));
    }
    
    let agent_id = agent.agent_id.clone();
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(agent_id.clone(), agent);

    Ok(json!({
        "agent_id": agent_id
    }))
}

async fn handle_unregister_agent(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.remove(agent_id);

    Ok(json!({
        "unregistered": true,
        "agent_id": agent_id
    }))
}

async fn handle_get_agent(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    
    let guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(agent_id)
        .map(|a| json!(a))
        .ok_or_else(|| "Agent not found".to_string())
}

async fn handle_list_agents(
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let agent_list: Vec<AgentIdentity> = guard.values().cloned().collect();
    
    Ok(json!({
        "agents": agent_list
    }))
}

async fn handle_find_agents_by_capability(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let capability = params.get("capability").and_then(|c| c.as_str()).ok_or("Missing capability")?;
    
    let guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<AgentIdentity> = guard.values()
        .filter(|a| a.capabilities.contains(&capability.to_string()))
        .cloned()
        .collect();
    
    Ok(json!({
        "agents": found
    }))
}

async fn handle_find_agents_by_type(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_type = params.get("agent_type").and_then(|t| t.as_str()).ok_or("Missing agent_type")?;
    
    let guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<AgentIdentity> = guard.values()
        .filter(|a| a.agent_type == agent_type)
        .cloned()
        .collect();
    
    Ok(json!({
        "agents": found
    }))
}

async fn handle_update_agent_status(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    let status = params.get("status").and_then(|s| s.as_str()).ok_or("Missing status")?;
    
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(agent) = guard.get_mut(agent_id) {
        agent.status = status.to_string();
        agent.last_seen = current_timestamp();
    }

    Ok(json!({
        "updated": true,
        "agent_id": agent_id
    }))
}

async fn handle_link_to_public_account(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(agent) = guard.get_mut(agent_id) {
        agent.public_account_id = Some(account_id.to_string());
    }

    Ok(json!({
        "linked": true
    }))
}

async fn handle_unlink_from_public_account(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(agent) = guard.get_mut(agent_id) {
        agent.public_account_id = None;
    }

    Ok(json!({
        "unlinked": true
    }))
}

async fn handle_get_agents_by_public_account(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<AgentIdentity> = guard.values()
        .filter(|a| a.public_account_id.as_ref() == Some(&account_id.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "agents": found
    }))
}

async fn handle_send_message(
    params: &serde_json::Value,
    messages: &Arc<Mutex<Vec<AgentMessage>>>,
) -> Result<serde_json::Value, String> {
    let message: AgentMessage = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid message: {}", e))?;
    
    let message_id = message.message_id.clone();
    let mut guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.push(message.clone());
    
    // Keep only last 1000 messages
    let len = guard.len();
    if len > 1000 {
        guard.drain(0..len - 1000);
    }

    Ok(json!({
        "sent": true,
        "message_id": message_id
    }))
}

async fn handle_get_messages(
    params: &serde_json::Value,
    messages: &Arc<Mutex<Vec<AgentMessage>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    let limit = limit.unwrap_or(50);
    
    let found: Vec<AgentMessage> = guard.iter()
        .filter(|m| {
            m.from_agent == agent_id || 
            m.to_agent.as_ref().map(|t| t == agent_id).unwrap_or(false) ||
            m.to_agent.is_none()
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();

    Ok(json!({
        "messages": found
    }))
}

async fn handle_broadcast_presence(
    params: &serde_json::Value,
    agents: &Arc<Mutex<HashMap<String, AgentIdentity>>>,
) -> Result<serde_json::Value, String> {
    let presence: AgentPresence = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid presence: {}", e))?;
    
    let agent_id = presence.agent_id.clone();
    let mut guard = agents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(agent) = guard.get_mut(&presence.agent_id) {
        agent.status = presence.status.clone();
        agent.last_seen = presence.timestamp;
    }

    Ok(json!({
        "broadcasted": true,
        "agent_id": agent_id
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
    format!("agent_node_{:x}", timestamp)
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

    #[tokio::test]
    async fn test_register_and_get_agent() {
        let config = AiAgentServiceConfig::default();
        let service = AiAgentService::new(config);
        
        let agent = AgentIdentity {
            agent_id: "test-agent-1".to_string(),
            agent_type: "llm".to_string(),
            deployment_type: "service".to_string(),
            name: "Test Agent".to_string(),
            capabilities: vec!["text-generation".to_string()],
            model_id: Some("model-1".to_string()),
            node_id: service.node_id().to_string(),
            digit_id: None,
            public_account_id: None,
            status: "online".to_string(),
            last_seen: current_timestamp(),
            metadata: json!({}),
        };

        service.register_agent(agent).expect("Failed to register agent");
        let retrieved = service.get_agent("test-agent-1".to_string()).expect("Expected agent");
        assert_eq!(retrieved.name, "Test Agent");
    }

    #[tokio::test]
    async fn test_find_agents_by_capability() {
        let config = AiAgentServiceConfig::default();
        let service = AiAgentService::new(config);
        
        let agent = AgentIdentity {
            agent_id: "test-agent-2".to_string(),
            agent_type: "llm".to_string(),
            deployment_type: "service".to_string(),
            name: "Test Agent 2".to_string(),
            capabilities: vec!["text-generation".to_string(), "code-generation".to_string()],
            model_id: Some("model-1".to_string()),
            node_id: service.node_id().to_string(),
            digit_id: None,
            public_account_id: None,
            status: "online".to_string(),
            last_seen: current_timestamp(),
            metadata: json!({}),
        };

        service.register_agent(agent).expect("Failed to register agent");
        
        let found = service.find_by_capability("text-generation".to_string());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].agent_id, "test-agent-2");
    }
}
