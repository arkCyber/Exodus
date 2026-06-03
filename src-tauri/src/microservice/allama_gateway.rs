//! Exodus Browser — Ollama-compatible HTTP gateway on port 11435 (embedded Allama backend).

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};

use crate::inference_engine::{
    ChatMessage as EngineChatMessage, ChatRequest as EngineChatRequest, InferenceEngine,
    InferenceRequest, ModelInfo,
};

/// Shared gateway state.
#[derive(Clone)]
pub struct AllamaGatewayState {
    pub engine: Arc<InferenceEngine>,
    pub port: u16,
    pub default_model: String,
}

/// Handle for a running embedded HTTP server.
pub struct AllamaGatewayHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    pub port: u16,
}

/// Start the embedded Ollama-compatible API server.
pub async fn start_embedded_gateway(
    engine: Arc<InferenceEngine>,
    port: u16,
    default_model: String,
) -> Result<AllamaGatewayHandle, String> {
    let state = AllamaGatewayState {
        engine,
        port,
        default_model,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/version", get(api_version))
        .route("/api/tags", get(api_tags))
        .route("/api/generate", post(api_generate))
        .route("/api/chat", post(api_chat))
        .route("/v1/models", get(v1_models))
        .route("/v1/chat/completions", post(v1_chat_completions))
        .route("/v1/embeddings", post(v1_embeddings))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Allama gateway bind :{port} failed: {e}"))?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        if let Err(e) = server.await {
            eprintln!("[Allama gateway] server error: {e}");
        }
    });

    Ok(AllamaGatewayHandle {
        shutdown_tx: Some(shutdown_tx),
        port,
    })
}

impl AllamaGatewayHandle {
    /// Stop the embedded HTTP server.
    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

async fn api_version() -> Json<serde_json::Value> {
    Json(json!({
        "version": "exodus-allama-embedded-1.0",
        "backend": "exodus-inference-engine",
    }))
}

async fn api_tags(State(state): State<AllamaGatewayState>) -> Json<serde_json::Value> {
    let models = state.engine.list_models().await;
    let tags: Vec<serde_json::Value> = models
        .iter()
        .map(|m| {
            json!({
                "name": m.name,
                "model": m.name,
                "modified_at": chrono::Utc::now().to_rfc3339(),
                "size": m.size_bytes,
                "digest": "",
                "details": {
                    "family": "allama",
                    "parameter_size": m.parameters,
                    "quantization_level": m.quantization,
                }
            })
        })
        .collect();
    Json(json!({ "models": tags }))
}

async fn v1_models(State(state): State<AllamaGatewayState>) -> Json<serde_json::Value> {
    let models = state.engine.list_models().await;
    let data: Vec<serde_json::Value> = models
        .iter()
        .map(|m| {
            json!({
                "id": m.name,
                "object": "model",
                "created": chrono::Utc::now().timestamp(),
                "owned_by": "exodus-allama",
            })
        })
        .collect();
    Json(json!({ "object": "list", "data": data }))
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateRequest {
    model: Option<String>,
    prompt: String,
    stream: Option<bool>,
    #[serde(default)]
    options: serde_json::Value,
}

async fn api_generate(
    State(state): State<AllamaGatewayState>,
    Json(body): Json<OllamaGenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let model = body.model.unwrap_or_else(|| state.default_model.clone());
    ensure_model_loaded(&state, &model).await?;

    let req = InferenceRequest {
        model: model.clone(),
        prompt: body.prompt,
        max_tokens: body.options.get("num_predict").and_then(|v| v.as_u64()).map(|n| n as usize),
        temperature: body.options.get("temperature").and_then(|v| v.as_f64()).map(|f| f as f32),
        top_p: None,
        top_k: None,
        repeat_penalty: None,
        stop: None,
        stream: body.stream.unwrap_or(false),
    };

    if body.stream.unwrap_or(false) {
        return Ok(sse_from_generate(state, req).await.into_response());
    }

    let resp = state
        .engine
        .generate_local(req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(json!({
        "model": model,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "response": resp.text.unwrap_or_default(),
        "done": true,
    }))
    .into_response())
}

#[derive(Debug, Deserialize)]
struct OllamaChatRequest {
    model: Option<String>,
    messages: Vec<OllamaMessage>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

async fn api_chat(
    State(state): State<AllamaGatewayState>,
    Json(body): Json<OllamaChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let model = body.model.unwrap_or_else(|| state.default_model.clone());
    ensure_model_loaded(&state, &model).await?;

    let chat_req = EngineChatRequest {
        model: model.clone(),
        messages: body
            .messages
            .into_iter()
            .map(|m| EngineChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: body.stream.unwrap_or(false),
    };

    if body.stream.unwrap_or(false) {
        return Ok(sse_from_chat(state, chat_req).await.into_response());
    }

    let resp = state
        .engine
        .chat_local(chat_req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(json!({
        "model": model,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "message": { "role": "assistant", "content": resp.text.unwrap_or_default() },
        "done": true,
    }))
    .into_response())
}

#[derive(Debug, Deserialize)]
struct OpenAiChatRequest {
    model: Option<String>,
    messages: Vec<OllamaMessage>,
    stream: Option<bool>,
}

async fn v1_chat_completions(
    State(state): State<AllamaGatewayState>,
    Json(body): Json<OpenAiChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let model = body.model.unwrap_or_else(|| state.default_model.clone());
    ensure_model_loaded(&state, &model).await?;

    let chat_req = EngineChatRequest {
        model: model.clone(),
        messages: body
            .messages
            .into_iter()
            .map(|m| EngineChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: body.stream.unwrap_or(false),
    };

    if body.stream.unwrap_or(false) {
        return Ok(openai_sse_from_chat(state, model, chat_req).await.into_response());
    }

    let resp = state
        .engine
        .chat_local(chat_req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let text = resp.text.unwrap_or_default();
    Ok(Json(json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": text },
            "finish_reason": "stop"
        }],
    }))
    .into_response())
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingRequest {
    model: Option<String>,
    input: serde_json::Value,
}

async fn v1_embeddings(
    State(state): State<AllamaGatewayState>,
    Json(body): Json<OpenAiEmbeddingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let model = body.model.unwrap_or_else(|| state.default_model.clone());
    ensure_model_loaded(&state, &model).await?;

    let text = match &body.input {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .first()
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };

    let emb_req = crate::inference_engine::EmbeddingRequest {
        model,
        text,
    };
    let resp = state
        .engine
        .embed_local(emb_req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let embedding = resp.embedding.unwrap_or_default();
    Ok(Json(json!({
        "object": "list",
        "data": [{ "object": "embedding", "index": 0, "embedding": embedding }],
        "model": resp.dimensions.to_string(),
    })))
}

async fn ensure_model_loaded(state: &AllamaGatewayState, model: &str) -> Result<(), (StatusCode, String)> {
    let loaded = state.engine.get_loaded_model().await;
    if loaded.as_deref() == Some(model) {
        return Ok(());
    }
    state
        .engine
        .load_model(model.to_string())
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn sse_from_generate(state: AllamaGatewayState, req: InferenceRequest) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let resp = state.engine.generate_local(req).await;
    let chunk = match resp {
        Ok(r) => r.text.unwrap_or_default(),
        Err(e) => format!("error: {e}"),
    };
    let events = vec![
        Ok(Event::default().json_data(json!({ "response": chunk, "done": false })).expect("Failed to serialize SSE event")),
        Ok(Event::default().json_data(json!({ "done": true })).expect("Failed to serialize SSE event")),
    ];
    Sse::new(stream::iter(events)).keep_alive(KeepAlive::default())
}

async fn sse_from_chat(state: AllamaGatewayState, req: EngineChatRequest) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let resp = state.engine.chat_local(req).await;
    let chunk = match resp {
        Ok(r) => r.text.unwrap_or_default(),
        Err(e) => format!("error: {e}"),
    };
    let events = vec![
        Ok(Event::default().json_data(json!({
            "message": { "role": "assistant", "content": chunk },
            "done": false
        })).expect("Failed to serialize SSE event")),
        Ok(Event::default().json_data(json!({ "done": true })).expect("Failed to serialize SSE event")),
    ];
    Sse::new(stream::iter(events)).keep_alive(KeepAlive::default())
}

async fn openai_sse_from_chat(
    state: AllamaGatewayState,
    model: String,
    req: EngineChatRequest,
) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let resp = state.engine.chat_local(req).await;
    let chunk = match resp {
        Ok(r) => r.text.unwrap_or_default(),
        Err(e) => format!("error: {e}"),
    };
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let events = vec![
        Ok(Event::default().json_data(json!({
            "id": id,
            "object": "chat.completion.chunk",
            "model": model,
            "choices": [{ "index": 0, "delta": { "content": chunk }, "finish_reason": null }]
        })).expect("Failed to serialize SSE event")),
        Ok(Event::default().data("[DONE]")),
    ];
    Sse::new(stream::iter(events)).keep_alive(KeepAlive::default())
}

/// Register GGUF files under `root` into the inference engine registry.
pub async fn scan_and_register_models(engine: &InferenceEngine, root: &std::path::Path) -> usize {
    let mut count = 0;
    if !root.is_dir() {
        return 0;
    }
    let walker = walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok());
    for entry in walker {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("gguf") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model")
            .to_string();
        let size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf());

        let info = ModelInfo {
            name: name.clone(),
            path: rel,
            size_bytes,
            quantization: "GGUF".to_string(),
            parameters: "unknown".to_string(),
            context_length: 8192,
            loaded: false,
            backend: crate::inference_engine::BackendType::Allama,
        };
        if engine.add_model(info).await.is_ok() {
            count += 1;
        }
    }
    count
}
