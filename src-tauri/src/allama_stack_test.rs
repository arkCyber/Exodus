//! Exodus Browser — Allama stack integration tests (mock Ollama-compatible HTTP server).

#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex as AsyncMutex;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tokio::sync::oneshot;

use crate::hermes_agent::{HermesAgent, TaskType};
use crate::inference_engine::{
    BackendType, ChatMessage, ChatRequest, InferenceEngine, InferenceRequest, ModelInfo,
};
use crate::microservice::allama_gateway::{scan_and_register_models, start_embedded_gateway};
use crate::microservice::allama_http_client::AllamaHttpClient;
use crate::microservice::allama_process::{discover_bundle_models_dir, resolve_models_dir};
use crate::python_microservice::{PythonExecuteRequest, PythonMicroservice};

/// Serialize mock-server tests to avoid port / runtime races under parallel `cargo test`.
static MOCK_SERVER_TEST_LOCK: LazyLock<AsyncMutex<()>> = LazyLock::new(|| AsyncMutex::new(()));

async fn with_mock_server<F, Fut, T>(f: F) -> T
where
    F: FnOnce(u16) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let _guard = MOCK_SERVER_TEST_LOCK.lock().await;
    let (port, shutdown_tx) = start_mock_allama_server().await;
    let result = f(port).await;
    let _ = shutdown_tx.send(());
    result
}

/// Mock Allama / Ollama HTTP API for tests.
async fn start_mock_allama_server() -> (u16, oneshot::Sender<()>) {
    let app = Router::new()
        .route("/api/tags", get(mock_tags))
        .route("/api/generate", post(mock_generate))
        .route("/api/chat", post(mock_chat))
        .route("/api/embeddings", post(mock_embeddings));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock allama");
    let port = listener.local_addr().expect("local addr").port();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        if let Err(e) = server.await {
            eprintln!("[mock allama] server error: {e}");
        }
    });

    wait_for_mock_server(port).await;
    (port, shutdown_tx)
}

async fn wait_for_mock_server(port: u16) {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/api/tags");
    for _ in 0..100 {
        if client
            .get(&url)
            .timeout(std::time::Duration::from_secs(1))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("mock Allama server did not start on port {port}");
}

async fn mock_tags() -> Json<Value> {
    Json(json!({
        "models": [{
            "name": "exodus-default",
            "model": "exodus-default",
            "modified_at": "2026-01-01T00:00:00Z",
            "size": 0,
            "digest": "test"
        }]
    }))
}

async fn mock_generate(Json(body): Json<Value>) -> Json<Value> {
    let prompt = body.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    Json(json!({
        "model": "exodus-default",
        "response": format!("mock-generate: {prompt}"),
        "done": true
    }))
}

async fn mock_chat(Json(body): Json<Value>) -> Json<Value> {
    let content = body
        .get("messages")
        .and_then(|m| m.as_array())
        .and_then(|a| a.last())
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("hi");
    Json(json!({
        "model": "exodus-default",
        "message": { "role": "assistant", "content": format!("mock-chat: {content}") },
        "done": true
    }))
}

async fn mock_embeddings(Json(body): Json<Value>) -> Json<Value> {
    let _prompt = body.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    Json(json!({
        "embedding": [0.1, 0.2, 0.3, 0.4]
    }))
}

#[tokio::test]
async fn http_client_generate_and_chat_against_mock() {
    with_mock_server(|port| async move {
        let client = AllamaHttpClient::from_port(port);
        assert!(client.probe().await);

        let names = client.list_model_names().await.expect("tags");
        assert!(names.iter().any(|n| n == "exodus-default"));

        let gen = client
            .generate("exodus-default", "hello world", Some(32), None)
            .await
            .expect("generate");
        assert!(gen.contains("mock-generate"));

        let chat = client
            .chat(
                "exodus-default",
                vec![crate::microservice::AllamaChatMessage {
                    role: "user".to_string(),
                    content: "ping".to_string(),
                }],
                None,
                None,
            )
            .await
            .expect("chat");
        assert!(chat.contains("mock-chat"));

        let emb = client.embed("exodus-default", "text").await.expect("embed");
        assert_eq!(emb.len(), 4);
    })
    .await;
}

#[tokio::test]
async fn inference_engine_delegates_to_http_when_native_mode() {
    with_mock_server(|port| async move {
    let engine = Arc::new(InferenceEngine::new());
    engine.set_embedded_gateway_active(false);
    engine.set_allama_http_port(Some(port)).await;

    engine
        .add_model(ModelInfo {
            name: "exodus-default".to_string(),
            path: std::path::PathBuf::from("builtin"),
            size_bytes: 0,
            quantization: "stub".to_string(),
            parameters: "n/a".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::Allama,
        })
        .await
        .expect("Failed to add model");
    engine.load_model("exodus-default".to_string()).await.expect("Failed to load model");

    let resp = engine
        .generate(InferenceRequest {
            model: "exodus-default".to_string(),
            prompt: "test prompt".to_string(),
            max_tokens: Some(16),
            temperature: None,
            top_p: None,
            top_k: None,
            repeat_penalty: None,
            stop: None,
            stream: false,
        })
        .await
        .expect("Failed to generate response");
    let text = resp.text.unwrap_or_default();
    assert!(text.contains("mock-generate"));

    let chat_resp = engine
        .chat(ChatRequest {
            model: "exodus-default".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: false,
        })
        .await
        .expect("Failed to get chat response");
    assert!(chat_resp.text.unwrap_or_default().contains("mock-chat"));
    })
    .await;
}

#[tokio::test]
async fn inference_engine_embedded_gateway_uses_local_stub() {
    with_mock_server(|port| async move {
    let engine = InferenceEngine::new();
    engine.set_embedded_gateway_active(true);
    engine.set_allama_http_port(Some(port)).await;

    engine
        .add_model(ModelInfo {
            name: "exodus-default".to_string(),
            path: std::path::PathBuf::from("builtin"),
            size_bytes: 0,
            quantization: "stub".to_string(),
            parameters: "n/a".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::Allama,
        })
        .await
        .expect("Failed to add model");
    engine.load_model("exodus-default".to_string()).await.expect("Failed to load model");

    let resp = engine
        .generate(InferenceRequest {
            model: "exodus-default".to_string(),
            prompt: "local only".to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            repeat_penalty: None,
            stop: None,
            stream: false,
        })
        .await
        .expect("Failed to generate response for local stub");
    let text = resp.text.unwrap_or_default();
    assert!(text.contains("Generated response"));
    assert!(!text.contains("mock-generate"));
    })
    .await;
}

#[tokio::test]
async fn hermes_analysis_calls_allama_http() {
    with_mock_server(|port| async move {
    let agent = HermesAgent::with_allama_port(port);
    let task_id = agent
        .create_task(
            TaskType::Analysis,
            "Analyze privacy policy".to_string(),
            1,
            HashMap::new(),
        )
        .await
        .expect("Failed to create task");
    let result = agent.execute_task(&task_id).await.expect("Failed to execute task");
    assert_eq!(
        result.get("backend").and_then(|v| v.as_str()),
        Some("allama-http")
    );
    assert!(
        result
            .get("analysis")
            .and_then(|a| a.get("summary"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .contains("mock-chat")
    );
    })
    .await;
}

#[tokio::test]
async fn python_allama_chat_prefix_routes_to_http() {
    with_mock_server(|port| async move {
    let py = PythonMicroservice::with_allama_port(port);
    py.set_running_for_tests().await;
    let resp = py
        .execute_for_tests(PythonExecuteRequest {
            code: "ALLAMA_CHAT:summarize Exodus browser".to_string(),
            variables: None,
            timeout_secs: None,
        })
        .await
        .expect("Failed to execute Python code");
    assert!(resp.success);
    assert!(
        resp
            .output
            .as_deref()
            .unwrap_or("")
            .contains("mock-generate")
    );
    })
    .await;
}

#[tokio::test]
async fn scan_registers_gguf_in_temp_dir() {
    let tmp = std::env::temp_dir().join(format!("exodus-gguf-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("Failed to create temp dir");
    let gguf = tmp.join("tiny-test.gguf");
    std::fs::write(&gguf, b"GGUF").expect("Failed to write GGUF file");

    let engine = InferenceEngine::new();
    let n = scan_and_register_models(&engine, &tmp).await;
    assert_eq!(n, 1);
    let models = engine.list_models().await;
    assert!(models.iter().any(|m| m.name == "tiny-test"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn resolve_models_dir_prefers_app_data() {
    let tmp = std::env::temp_dir().join(format!("exodus-models-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let resolved = resolve_models_dir(Some(&tmp));
    assert!(resolved.starts_with(&tmp));
    assert!(resolved.to_string_lossy().contains("allama"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn discover_bundle_models_dir_is_optional() {
    let _ = discover_bundle_models_dir();
}

#[tokio::test]
async fn embedded_gateway_exposes_ollama_api() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);

    let engine = Arc::new(InferenceEngine::new());
    engine.set_embedded_gateway_active(true);
    engine
        .add_model(ModelInfo {
            name: "exodus-default".to_string(),
            path: std::path::PathBuf::from("builtin"),
            size_bytes: 0,
            quantization: "stub".to_string(),
            parameters: "n/a".to_string(),
            context_length: 2048,
            loaded: false,
            backend: BackendType::Allama,
        })
        .await
        .expect("Failed to add model");
    engine.load_model("exodus-default".to_string()).await.expect("Failed to load model");

    let handle = start_embedded_gateway(engine, port, "exodus-default".to_string())
        .await
        .expect("gateway start");
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    let client = AllamaHttpClient::from_port(port);
    assert!(client.probe().await);
    let text = client
        .generate("exodus-default", "gateway test", None, None)
        .await
        .expect("generate via gateway");
    assert!(text.contains("Generated response"));

    handle.stop();
}
