//! Exodus RAG Service - Independent Retrieval-Augmented Generation Service
//!
//! This service provides local vector storage, semantic search, and automatic
//! file watching for incremental re-indexing.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Document for RAG indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    content: String,
    metadata: HashMap<String, String>,
    embedding: Option<Vec<f32>>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    document: Document,
    score: f32,
}

/// RAG Service state
struct RagService {
    db: sled::Db,
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl RagService {
    fn new(db_path: &Path) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self {
            db,
            documents: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn upsert_document(&self, doc: Document) -> Result<serde_json::Value> {
        let mut docs = self.documents.write().await;
        docs.insert(doc.id.clone(), doc.clone());
        
        // Store in sled
        let key = doc.id.as_bytes();
        let value = serde_json::to_vec(&doc)?;
        self.db.insert(key, value)?;
        
        Ok(serde_json::json!({ "success": true, "id": doc.id }))
    }

    async fn search(&self, query: String, top_k: usize) -> Result<Vec<SearchResult>> {
        let docs = self.documents.read().await;
        
        // Simple keyword search (in production, use vector similarity)
        let mut results = Vec::new();
        for (_id, doc) in docs.iter() {
            if doc.content.to_lowercase().contains(&query.to_lowercase()) {
                results.push(SearchResult {
                    document: doc.clone(),
                    score: 1.0, // Placeholder score
                });
            }
        }
        
        results.truncate(top_k);
        Ok(results)
    }

    async fn get_document(&self, id: String) -> Result<Option<Document>> {
        let docs = self.documents.read().await;
        Ok(docs.get(&id).cloned())
    }

    async fn list_documents(&self) -> Result<Vec<String>> {
        let docs = self.documents.read().await;
        Ok(docs.keys().cloned().collect())
    }

    async fn delete_document(&self, id: String) -> Result<bool> {
        let mut docs = self.documents.write().await;
        let removed = docs.remove(&id).is_some();
        
        if removed {
            self.db.remove(id.as_bytes())?;
        }
        
        Ok(removed)
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "rag.upsert" => {
                let params: serde_json::Value = request.params;
                if let Some(doc) = params.get("document") {
                    match serde_json::from_value(doc.clone()) {
                        Ok(document) => self.upsert_document(document).await,
                        Err(e) => Err(anyhow::anyhow!("Invalid document: {}", e)),
                    }
                } else {
                    Err(anyhow::anyhow!("Missing document parameter"))
                }
            }
            "rag.search" => {
                let params: serde_json::Value = request.params;
                let query = params.get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let top_k = params.get("top_k")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                
                self.search(query, top_k)
                    .await
                    .map(|results| serde_json::to_value(results).unwrap())
            }
            "rag.get" => {
                let params: serde_json::Value = request.params;
                let id = params.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                self.get_document(id)
                    .await
                    .map(|doc| serde_json::to_value(doc).unwrap())
            }
            "rag.list" => {
                self.list_documents()
                    .await
                    .map(|ids| serde_json::to_value(ids).unwrap())
            }
            "rag.delete" => {
                let params: serde_json::Value = request.params;
                let id = params.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                self.delete_document(id)
                    .await
                    .map(|deleted| serde_json::json!({ "deleted": deleted }))
            }
            "health.check" => {
                Ok(serde_json::json!({ "status": "healthy" }))
            }
            _ => Err(anyhow::anyhow!("Method not found: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: request.id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: e.to_string(),
                }),
                id: request.id,
            },
        }
    }
}

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Socket path for Unix Domain Socket
    #[arg(short, long, default_value = "/tmp/exodus-rag.sock")]
    socket: String,

    /// Database path
    #[arg(short, long, default_value = "/tmp/exodus-rag-db")]
    db_path: String,

    /// Watch directory for automatic re-indexing
    #[arg(short, long)]
    watch_dir: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    let args = Args::parse();

    info!("Starting Exodus RAG Service");
    info!("Socket: {}", args.socket);
    info!("Database: {}", args.db_path);

    // Remove existing socket if present
    if Path::new(&args.socket).exists() {
        std::fs::remove_file(&args.socket)?;
    }

    // Initialize RAG service
    let service = Arc::new(RagService::new(Path::new(&args.db_path))?);

    // Start file watcher if directory specified
    if let Some(watch_dir) = &args.watch_dir {
        let service_clone = Arc::clone(&service);
        let watch_path = watch_dir.clone();
        tokio::spawn(async move {
            if let Err(e) = start_file_watcher(service_clone, Path::new(&watch_path)).await {
                error!("File watcher error: {}", e);
            }
        });
    }

    // Start Unix Domain Socket listener
    let listener = UnixListener::bind(&args.socket)?;
    info!("Listening on {}", args.socket);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let service_clone = Arc::clone(&service);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, service_clone).await {
                        error!("Client error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

async fn handle_client(mut stream: UnixStream, service: Arc<RagService>) -> Result<()> {
    loop {
        let mut request_str = String::new();
        {
            let mut reader = BufReader::new(&stream);
            match reader.read_line(&mut request_str) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }
        }

        let request: JsonRpcRequest = match serde_json::from_str(&request_str.trim()) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = service.handle_request(request).await;
        let response_str = serde_json::to_string(&response)?;
        
        writeln!(stream, "{}", response_str)?;
    }

    Ok(())
}

async fn start_file_watcher(_service: Arc<RagService>, dir: &Path) -> Result<()> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    
    info!("Watching directory: {}", dir.display());
    
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
        if let Err(e) = tx.blocking_send(res) {
            error!("Failed to send watch event: {}", e);
        }
    })?;

    watcher.watch(dir, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                info!("File event: {:?}", event);
                // In production, process file changes and re-index
            }
            Err(e) => {
                error!("Watch error: {}", e);
            }
        }
    }

    Ok(())
}
