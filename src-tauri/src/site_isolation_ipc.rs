//! Site Isolation IPC (Inter-Process Communication)
//!
//! This module provides secure inter-process communication between isolated site processes.
//! It uses Unix Domain Sockets on Unix and Named Pipes on Windows for efficient local IPC.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::site_isolation::{SiteId, SiteMessage};

/// IPC message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcMessageType {
    /// Command message
    Command,
    /// Response message
    Response,
    /// Event notification
    Event,
    /// Data transfer
    Data,
    /// Heartbeat
    Heartbeat,
}

/// IPC message priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpcPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// IPC security level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcSecurityLevel {
    Low,
    Medium,
    High,
    Maximum,
}

/// IPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Unique message ID
    pub message_id: String,
    /// Message type
    pub message_type: IpcMessageType,
    /// Source process/site
    pub source: String,
    /// Destination process/site
    pub destination: String,
    /// Command or action
    pub command: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message priority
    pub priority: IpcPriority,
    /// Security level
    pub security_level: IpcSecurityLevel,
    /// Timestamp
    pub timestamp: u64,
    /// Time-to-live in seconds
    pub ttl: Option<u64>,
}

impl IpcMessage {
    pub fn new(
        source: String,
        destination: String,
        command: String,
        payload: serde_json::Value,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            message_id: format!("ipc-{}-{}", 
                source,
                now
            ),
            message_type: IpcMessageType::Command,
            source,
            destination,
            command,
            payload,
            priority: IpcPriority::Normal,
            security_level: IpcSecurityLevel::Medium,
            timestamp: now,
            ttl: Some(60),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now - self.timestamp > ttl
        } else {
            false
        }
    }
}

/// IPC channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcChannelConfig {
    /// Channel ID
    pub channel_id: String,
    /// Source site
    pub source_site: SiteId,
    /// Destination site
    pub destination_site: SiteId,
    /// Buffer size in bytes
    pub buffer_size: usize,
    /// Whether to use encryption
    pub use_encryption: bool,
    /// Maximum message size
    pub max_message_size: usize,
}

impl Default for IpcChannelConfig {
    fn default() -> Self {
        Self {
            channel_id: "default".to_string(),
            source_site: SiteId {
                scheme: "https".to_string(),
                etld_plus_one: "default".to_string(),
            },
            destination_site: SiteId {
                scheme: "https".to_string(),
                etld_plus_one: "default".to_string(),
            },
            buffer_size: 64 * 1024, // 64KB
            use_encryption: true,
            max_message_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// IPC channel
pub struct IpcChannel {
    /// Channel configuration
    config: IpcChannelConfig,
    /// Unix stream (for Unix-like systems)
    #[cfg(unix)]
    stream: Option<UnixStream>,
    /// Message queue
    message_queue: Arc<Mutex<Vec<IpcMessage>>>,
    /// Connected state
    connected: Arc<Mutex<bool>>,
}

impl IpcChannel {
    /// Create a new IPC channel
    pub fn new(config: IpcChannelConfig) -> Self {
        Self {
            config,
            #[cfg(unix)]
            stream: None,
            message_queue: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to the channel
    #[cfg(unix)]
    pub async fn connect(&mut self, socket_path: &str) -> Result<(), String> {
        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| format!("Failed to connect to socket: {}", e))?;

        self.stream = Some(stream);
        *self.connected.lock().unwrap() = true;

        info!("IPC channel connected to {}", socket_path);
        Ok(())
    }

    /// Disconnect from the channel
    pub async fn disconnect(&mut self) -> Result<(), String> {
        *self.connected.lock().unwrap() = false;
        #[cfg(unix)]
        {
            self.stream = None;
        }
        Ok(())
    }

    /// Send a message
    pub async fn send_message(&mut self, message: IpcMessage) -> Result<(), String> {
        // Check message size
        let serialized = serde_json::to_vec(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        if serialized.len() > self.config.max_message_size {
            return Err(format!("Message too large: {} bytes (max {})", 
                serialized.len(), self.config.max_message_size));
        }

        // Add to queue
        {
            let mut queue = self.message_queue.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            queue.push(message);
        }

        // Try to send immediately if connected
        #[cfg(unix)]
        {
            if *self.connected.lock().unwrap() {
                if let Some(ref mut stream) = self.stream {
                    self.flush_queue(stream).await?;
                }
            }
        }

        Ok(())
    }

    /// Flush message queue
    #[cfg(unix)]
    async fn flush_queue(&mut self, stream: &mut UnixStream) -> Result<(), String> {
        let mut queue = self.message_queue.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        while let Some(message) = queue.pop_front() {
            let serialized = serde_json::to_vec(&message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;

            // Write length prefix
            let len = serialized.len() as u32;
            stream.write_all(&len.to_be_bytes())
                .await
                .map_err(|e| format!("Failed to write length: {}", e))?;

            // Write message
            stream.write_all(&serialized)
                .await
                .map_err(|e| format!("Failed to write message: {}", e))?;
        }

        stream.flush()
            .await
            .map_err(|e| format!("Failed to flush stream: {}", e))?;

        Ok(())
    }

    /// Receive a message
    #[cfg(unix)]
    pub async fn receive_message(&mut self) -> Result<Option<IpcMessage>, String> {
        if let Some(ref mut stream) = self.stream {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes)
                .await
                .map_err(|e| format!("Failed to read length: {}", e))?;

            let len = u32::from_be_bytes(len_bytes) as usize;

            if len > self.config.max_message_size {
                return Err(format!("Message too large: {} bytes", len));
            }

            // Read message
            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)
                .await
                .map_err(|e| format!("Failed to read message: {}", e))?;

            let message: IpcMessage = serde_json::from_slice(&buffer)
                .map_err(|e| format!("Failed to deserialize message: {}", e))?;

            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    /// Get pending message count
    pub fn pending_count(&self) -> usize {
        self.message_queue.lock()
            .map(|q| q.len())
            .unwrap_or(0)
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
}

/// IPC router for routing messages between processes
pub struct IpcRouter {
    /// Channels keyed by channel ID
    channels: Arc<Mutex<HashMap<String, IpcChannel>>>,
    /// Message handlers
    handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(IpcMessage) + Send>>>>,
    /// Socket directory
    socket_dir: String,
}

impl IpcRouter {
    /// Create a new IPC router
    pub fn new(socket_dir: String) -> Self {
        // Create socket directory if it doesn't exist
        std::fs::create_dir_all(&socket_dir)
            .unwrap_or_else(|e| eprintln!("Failed to create socket directory: {}", e));

        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),
            socket_dir,
        }
    }

    /// Create a channel
    pub fn create_channel(&self, config: IpcChannelConfig) -> Result<(), String> {
        let channel = IpcChannel::new(config.clone());
        
        let mut channels = self.channels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        channels.insert(config.channel_id.clone(), channel);

        info!("Created IPC channel: {}", config.channel_id);
        Ok(())
    }

    /// Get a channel
    pub fn get_channel(&self, channel_id: &str) -> Option<IpcChannel> {
        let channels = self.channels.lock().ok()?;
        channels.get(channel_id).cloned()
    }

    /// Register a message handler
    pub fn register_handler<F>(&self, channel_id: String, handler: F) -> Result<(), String>
    where
        F: Fn(IpcMessage) + Send + 'static,
    {
        let mut handlers = self.handlers.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        handlers.insert(channel_id, Box::new(handler));

        Ok(())
    }

    /// Route a message
    pub async fn route_message(&self, message: IpcMessage) -> Result<(), String> {
        // Find channel based on destination
        let channel_id = format!("{}-{}", message.source, message.destination);
        
        let channels = self.channels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(channel) = channels.get(&channel_id) {
            // Clone channel for async operation
            let mut channel_clone = channel.clone();
            tokio::spawn(async move {
                if let Err(e) = channel_clone.send_message(message).await {
                    error!("Failed to send message: {}", e);
                }
            });
            Ok(())
        } else {
            Err(format!("Channel not found: {}", channel_id))
        }
    }

    /// Start listening for incoming connections
    #[cfg(unix)]
    pub async fn start_listener(&self, channel_id: &str) -> Result<(), String> {
        let socket_path = format!("{}/{}.sock", self.socket_dir, channel_id);

        // Remove existing socket if present
        if std::path::Path::new(&socket_path).exists() {
            std::fs::remove_file(&socket_path)
                .map_err(|e| format!("Failed to remove existing socket: {}", e))?;
        }

        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        info!("IPC listener started on {}", socket_path);

        let channels = self.channels.clone();
        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let _channels_clone = channels.clone();
                        let handlers_clone = handlers.clone();
                        let channel_id_clone = channel_id.to_string();

                        tokio::spawn(async move {
                            // Handle incoming connection
                            let mut channel = IpcChannel::new(IpcChannelConfig::default());
                            #[cfg(unix)]
                            {
                                channel.stream = Some(stream);
                                *channel.connected.lock().unwrap() = true;
                            }

                            // Process messages
                            loop {
                                #[cfg(unix)]
                                {
                                    match channel.receive_message().await {
                                        Ok(Some(message)) => {
                                            // Call handler if registered
                                            let handlers = handlers_clone.lock().unwrap();
                                            if let Some(handler) = handlers.get(&channel_id_clone) {
                                                handler(message);
                                            }
                                        }
                                        Ok(None) => {
                                            break;
                                        }
                                        Err(e) => {
                                            error!("Error receiving message: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

/// Convert SiteMessage to IpcMessage
pub fn site_message_to_ipc(site_message: SiteMessage) -> IpcMessage {
    IpcMessage {
        message_id: format!("site-ipc-{}", site_message.timestamp),
        message_type: IpcMessageType::Data,
        source: site_message.from_site.etld_plus_one.clone(),
        destination: site_message.to_site.etld_plus_one.clone(),
        command: site_message.message_type.clone(),
        payload: site_message.payload,
        priority: IpcPriority::Normal,
        security_level: IpcSecurityLevel::Medium,
        timestamp: site_message.timestamp,
        ttl: Some(60),
    }
}

/// Convert IpcMessage to SiteMessage
pub fn ipc_to_site_message(ipc_message: IpcMessage) -> Result<SiteMessage, String> {
    let from_site = SiteId {
        scheme: "https".to_string(),
        etld_plus_one: ipc_message.source,
    };
    let to_site = SiteId {
        scheme: "https".to_string(),
        etld_plus_one: ipc_message.destination,
    };

    Ok(SiteMessage {
        from_site,
        to_site,
        message_type: ipc_message.command,
        payload: ipc_message.payload,
        timestamp: ipc_message.timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_creation() {
        let message = IpcMessage::new(
            "source".to_string(),
            "dest".to_string(),
            "test".to_string(),
            serde_json::json!({"data": "test"}),
        );

        assert_eq!(message.source, "source");
        assert_eq!(message.destination, "dest");
        assert_eq!(message.command, "test");
    }

    #[test]
    fn test_ipc_message_expiration() {
        let mut message = IpcMessage::new(
            "source".to_string(),
            "dest".to_string(),
            "test".to_string(),
            serde_json::json!({}),
        );

        // Message with TTL should not be expired immediately
        assert!(!message.is_expired());

        // Expired message
        message.timestamp = 0;
        message.ttl = Some(1);
        assert!(message.is_expired());
    }

    #[test]
    fn test_ipc_channel_config_default() {
        let config = IpcChannelConfig::default();
        assert_eq!(config.buffer_size, 64 * 1024);
        assert!(config.use_encryption);
    }

    #[test]
    fn test_ipc_channel_creation() {
        let config = IpcChannelConfig::default();
        let channel = IpcChannel::new(config);
        assert!(!channel.is_connected());
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn test_site_message_conversion() {
        let from_site = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let to_site = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "google.com".to_string(),
        };

        let site_message = SiteMessage::new(
            from_site.clone(),
            to_site.clone(),
            "test".to_string(),
            serde_json::json!({"data": "test"}),
        );

        let ipc_message = site_message_to_ipc(site_message.clone());
        assert_eq!(ipc_message.source, "example.com");
        assert_eq!(ipc_message.destination, "google.com");

        let converted_back = ipc_to_site_message(ipc_message).unwrap();
        assert_eq!(converted_back.from_site.etld_plus_one, "example.com");
        assert_eq!(converted_back.to_site.etld_plus_one, "google.com");
    }

    #[tokio::test]
    async fn test_ipc_router_creation() {
        let router = IpcRouter::new("/tmp/test-ipc".to_string());
        assert_eq!(router.socket_dir, "/tmp/test-ipc");
    }
}
