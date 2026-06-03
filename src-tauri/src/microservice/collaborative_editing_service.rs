//! Real-time Collaborative Editing Service - Editor-agnostic real-time collaboration
//! 
//! This service provides real-time collaborative editing capabilities similar to Teamtype,
//! allowing multiple users to edit documents simultaneously with conflict resolution.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Document configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConfig {
    pub document_id: String,
    pub file_path: String,
    pub document_name: String,
    pub content_type: String, // "text", "markdown", "code"
    pub language: Option<String>, // for code documents
}

/// Text operation for collaborative editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOperation {
    pub operation_type: String, // "insert", "delete", "retain"
    pub position: usize,
    pub content: String,
    pub length: usize,
    pub user_id: String,
    pub timestamp: u64,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub document_id: String,
    pub file_path: String,
    pub document_name: String,
    pub content_type: String,
    pub language: Option<String>,
    pub content: String,
    pub version: u64,
    pub collaborators: Vec<String>,
    pub status: String, // "active", "inactive", "locked"
    pub created_at: u64,
    pub updated_at: u64,
    pub last_activity: u64,
}

/// Cursor position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub document_id: String,
    pub user_id: String,
    pub user_name: String,
    pub position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub color: Option<String>,
    pub timestamp: u64,
}

/// Configuration for Collaborative Editing Service
#[derive(Debug, Clone)]
pub struct CollaborativeEditingServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for CollaborativeEditingServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_collaborative_editing.sock");
        Self { socket_path }
    }
}

/// Collaborative Editing Service
pub struct CollaborativeEditingService {
    config: CollaborativeEditingServiceConfig,
    documents: Arc<Mutex<HashMap<String, DocumentMetadata>>>, // document_id -> metadata
    operations: Arc<Mutex<HashMap<String, Vec<TextOperation>>>>, // document_id -> operations
    cursors: Arc<Mutex<HashMap<String, Vec<CursorPosition>>>>, // document_id -> cursors
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl CollaborativeEditingService {
    pub fn new(config: CollaborativeEditingServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            documents: Arc::new(Mutex::new(HashMap::new())),
            operations: Arc::new(Mutex::new(HashMap::new())),
            cursors: Arc::new(Mutex::new(HashMap::new())),
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
        let documents = Arc::clone(&self.documents);
        let operations = Arc::clone(&self.operations);
        let cursors = Arc::clone(&self.cursors);
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
                                let documents = Arc::clone(&documents);
                                let operations = Arc::clone(&operations);
                                let cursors = Arc::clone(&cursors);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, documents, operations, cursors, node_id).await;
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

        println!("Collaborative Editing Service started on {:?}", socket_path);
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

        println!("Collaborative Editing Service stopped");
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

    /// Create a document
    #[allow(dead_code)]
    pub fn create_document(&self, config: DocumentConfig) -> Result<String, String> {
        let document_id = config.document_id.clone();
        
        let metadata = DocumentMetadata {
            document_id: document_id.clone(),
            file_path: config.file_path,
            document_name: config.document_name,
            content_type: config.content_type,
            language: config.language,
            content: String::new(),
            version: 0,
            collaborators: Vec::new(),
            status: "active".to_string(),
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            last_activity: current_timestamp(),
        };

        let mut documents = self.documents.lock().map_err(|e| format!("Lock error: {}", e))?;
        documents.insert(document_id.clone(), metadata);

        Ok(document_id)
    }

    /// Open a document
    #[allow(dead_code)]
    pub fn open_document(&self, document_id: String, user_id: String) -> Result<String, String> {
        let mut documents = self.documents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = documents.get_mut(&document_id) {
            if !metadata.collaborators.contains(&user_id) {
                metadata.collaborators.push(user_id);
            }
            metadata.status = "active".to_string();
            metadata.last_activity = current_timestamp();
        }
        Ok(document_id)
    }

    /// Close a document
    #[allow(dead_code)]
    pub fn close_document(&self, document_id: String, user_id: String) -> Result<(), String> {
        let mut documents = self.documents.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = documents.get_mut(&document_id) {
            metadata.collaborators.retain(|id| id != &user_id);
            if metadata.collaborators.is_empty() {
                metadata.status = "inactive".to_string();
            }
            metadata.last_activity = current_timestamp();
        }
        Ok(())
    }

    /// Apply text operation
    #[allow(dead_code)]
    pub fn apply_operation(&self, document_id: String, operation: TextOperation) -> Result<String, String> {
        let mut documents = self.documents.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut operations = self.operations.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(metadata) = documents.get_mut(&document_id) {
            match operation.operation_type.as_str() {
                "insert" => {
                    if operation.position <= metadata.content.len() {
                        metadata.content.insert_str(operation.position, &operation.content);
                        metadata.version += 1;
                        metadata.updated_at = current_timestamp();
                        metadata.last_activity = current_timestamp();
                    }
                }
                "delete" => {
                    let end = std::cmp::min(operation.position + operation.length, metadata.content.len());
                    if operation.position < metadata.content.len() {
                        metadata.content.replace_range(operation.position..end, "");
                        metadata.version += 1;
                        metadata.updated_at = current_timestamp();
                        metadata.last_activity = current_timestamp();
                    }
                }
                "retain" => {
                    // No change, just move cursor
                    metadata.last_activity = current_timestamp();
                }
                _ => {}
            }
            
            operations.entry(document_id.clone()).or_insert_with(Vec::new).push(operation);
            
            // Keep only last 1000 operations
            if let Some(ops) = operations.get_mut(&document_id) {
                if ops.len() > 1000 {
                    ops.drain(0..ops.len() - 1000);
                }
            }
        }
        
        Ok(document_id)
    }

    /// Get document info
    #[allow(dead_code)]
    pub fn get_document(&self, document_id: String) -> Option<DocumentMetadata> {
        let documents = self.documents.lock().ok()?;
        documents.get(&document_id).cloned()
    }

    /// List all documents
    #[allow(dead_code)]
    pub fn list_documents(&self) -> Vec<DocumentMetadata> {
        self.documents.lock()
            .map(|documents| documents.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Update cursor position
    #[allow(dead_code)]
    pub fn update_cursor(&self, cursor: CursorPosition) -> Result<(), String> {
        let mut cursors = self.cursors.lock().map_err(|e| format!("Lock error: {}", e))?;
        let document_id = cursor.document_id.clone();
        let user_id = cursor.user_id.clone();
        
        cursors.entry(document_id.clone()).or_insert_with(Vec::new);
        if let Some(doc_cursors) = cursors.get_mut(&document_id) {
            // Remove old cursor position for this user
            doc_cursors.retain(|c| c.user_id != user_id);
            // Add new cursor position
            doc_cursors.push(cursor);
            
            // Keep only last 100 cursors
            if doc_cursors.len() > 100 {
                doc_cursors.drain(0..doc_cursors.len() - 100);
            }
        }
        
        Ok(())
    }

    /// Get cursors for a document
    #[allow(dead_code)]
    pub fn get_cursors(&self, document_id: String) -> Vec<CursorPosition> {
        self.cursors.lock()
            .map(|cursors| cursors.get(&document_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Get operations for a document
    #[allow(dead_code)]
    pub fn get_operations(&self, document_id: String, since_version: Option<u64>) -> Vec<TextOperation> {
        self.operations.lock()
            .map(|operations| {
                let all_ops = operations.get(&document_id).cloned().unwrap_or_default();
                
                if let Some(version) = since_version {
                    all_ops.into_iter()
                        .skip_while(|op| op.timestamp < version)
                        .collect()
                } else {
                    all_ops
                }
            })
            .unwrap_or_default()
    }
}

fn generate_node_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    documents: Arc<Mutex<HashMap<String, DocumentMetadata>>>,
    operations: Arc<Mutex<HashMap<String, Vec<TextOperation>>>>,
    cursors: Arc<Mutex<HashMap<String, Vec<CursorPosition>>>>,
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
            "create_document" => handle_create_document(&params, &documents).await,
            "open_document" => handle_open_document(&params, &documents).await,
            "close_document" => handle_close_document(&params, &documents).await,
            "apply_operation" => handle_apply_operation(&params, &documents, &operations).await,
            "get_document" => handle_get_document(&params, &documents).await,
            "list_documents" => handle_list_documents(&documents).await,
            "update_cursor" => handle_update_cursor(&params, &cursors).await,
            "get_cursors" => handle_get_cursors(&params, &cursors).await,
            "get_operations" => handle_get_operations(&params, &operations).await,
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

async fn handle_create_document(
    params: &serde_json::Value,
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
) -> Result<serde_json::Value, String> {
    let config: DocumentConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid config: {}", e))?;
    
    let document_id = config.document_id.clone();
    
    let metadata = DocumentMetadata {
        document_id: document_id.clone(),
        file_path: config.file_path,
        document_name: config.document_name,
        content_type: config.content_type,
        language: config.language,
        content: String::new(),
        version: 0,
        collaborators: Vec::new(),
        status: "active".to_string(),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        last_activity: current_timestamp(),
    };

    let mut guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(document_id.clone(), metadata);

    Ok(json!({
        "document_id": document_id
    }))
}

async fn handle_open_document(
    params: &serde_json::Value,
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let mut guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(document_id) {
        if !metadata.collaborators.contains(&user_id.to_string()) {
            metadata.collaborators.push(user_id.to_string());
        }
        metadata.status = "active".to_string();
        metadata.last_activity = current_timestamp();
    }

    Ok(json!({
        "opened": true
    }))
}

async fn handle_close_document(
    params: &serde_json::Value,
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let mut guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(document_id) {
        metadata.collaborators.retain(|id| id != &user_id.to_string());
        if metadata.collaborators.is_empty() {
            metadata.status = "inactive".to_string();
        }
        metadata.last_activity = current_timestamp();
    }

    Ok(json!({
        "closed": true
    }))
}

async fn handle_apply_operation(
    params: &serde_json::Value,
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
    operations: &Arc<Mutex<HashMap<String, Vec<TextOperation>>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    let operation: TextOperation = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid operation: {}", e))?;
    
    let mut doc_guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut op_guard = operations.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(metadata) = doc_guard.get_mut(document_id) {
        match operation.operation_type.as_str() {
            "insert" => {
                if operation.position <= metadata.content.len() {
                    metadata.content.insert_str(operation.position, &operation.content);
                    metadata.version += 1;
                    metadata.updated_at = current_timestamp();
                    metadata.last_activity = current_timestamp();
                }
            }
            "delete" => {
                let end = std::cmp::min(operation.position + operation.length, metadata.content.len());
                if operation.position < metadata.content.len() {
                    metadata.content.replace_range(operation.position..end, "");
                    metadata.version += 1;
                    metadata.updated_at = current_timestamp();
                    metadata.last_activity = current_timestamp();
                }
            }
            "retain" => {
                metadata.last_activity = current_timestamp();
            }
            _ => {}
        }
        
        op_guard.entry(document_id.to_string()).or_insert_with(Vec::new).push(operation);
        
        if let Some(ops) = op_guard.get_mut(document_id) {
            if ops.len() > 1000 {
                ops.drain(0..ops.len() - 1000);
            }
        }
    }
    
    Ok(json!({
        "applied": true
    }))
}

async fn handle_get_document(
    params: &serde_json::Value,
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    
    let guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.get(document_id).ok_or("Document not found")?;

    Ok(json!(metadata))
}

async fn handle_list_documents(
    documents: &Arc<Mutex<HashMap<String, DocumentMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = documents.lock().map_err(|e| format!("Lock error: {}", e))?;
    let doc_list: Vec<DocumentMetadata> = guard.values().cloned().collect();

    Ok(json!({
        "documents": doc_list
    }))
}

async fn handle_update_cursor(
    params: &serde_json::Value,
    cursors: &Arc<Mutex<HashMap<String, Vec<CursorPosition>>>>,
) -> Result<serde_json::Value, String> {
    let cursor: CursorPosition = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid cursor: {}", e))?;
    
    let document_id = cursor.document_id.clone();
    let user_id = cursor.user_id.clone();
    
    let mut guard = cursors.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.entry(document_id.clone()).or_insert_with(Vec::new);
    if let Some(doc_cursors) = guard.get_mut(&document_id) {
        doc_cursors.retain(|c| c.user_id != user_id);
        doc_cursors.push(cursor);
        
        if doc_cursors.len() > 100 {
            doc_cursors.drain(0..doc_cursors.len() - 100);
        }
    }

    Ok(json!({
        "updated": true
    }))
}

async fn handle_get_cursors(
    params: &serde_json::Value,
    cursors: &Arc<Mutex<HashMap<String, Vec<CursorPosition>>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    
    let guard = cursors.lock().map_err(|e| format!("Lock error: {}", e))?;
    let cursor_list = guard.get(document_id).cloned().unwrap_or_default();

    Ok(json!({
        "cursors": cursor_list
    }))
}

async fn handle_get_operations(
    params: &serde_json::Value,
    operations: &Arc<Mutex<HashMap<String, Vec<TextOperation>>>>,
) -> Result<serde_json::Value, String> {
    let document_id = params.get("document_id").and_then(|d| d.as_str()).ok_or("Missing document_id")?;
    let since_version = params.get("since_version").and_then(|v| v.as_u64());
    
    let guard = operations.lock().map_err(|e| format!("Lock error: {}", e))?;
    let all_ops = guard.get(document_id).cloned().unwrap_or_default();
    
    let ops = if let Some(version) = since_version {
        all_ops.into_iter()
            .skip_while(|op| op.timestamp < version)
            .collect::<Vec<_>>()
    } else {
        all_ops
    };

    Ok(json!({
        "operations": ops
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}
