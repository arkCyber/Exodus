//! Exodus Browser — local AI API client (OpenAI-compatible / exodus-core).

use std::sync::Mutex;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::config::ExodusConfig;

/// Chunk payload emitted to the frontend during streaming.
#[derive(Debug, Clone, Serialize)]
pub struct AiChunkPayload {
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    content: String,
}

const SYSTEM_CHAT: &str =
    "You are Exodus, a helpful privacy-focused browser assistant. Be concise and practical.";

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Option<Vec<StreamChoice>>,
}

/// Check whether the local AI endpoint responds.
#[tauri::command]
pub async fn ai_health(config: tauri::State<'_, Mutex<ExodusConfig>>) -> Result<bool, String> {
    let url = {
        let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
        format!("http://127.0.0.1:{}/api/tags", cfg.ai_port)
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    match client.get(&url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Return current AI settings for the UI.
#[tauri::command]
pub fn get_ai_config(config: tauri::State<'_, Mutex<ExodusConfig>>) -> Result<ExodusConfig, String> {
    config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| format!("Config lock error: {}", e))
}

/// Update browser and AI settings (persisted to app data directory).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_ai_config(
    app: AppHandle,
    ai_port: u16,
    ai_model: String,
    embedding_model: String,
    homepage_url: String,
    search_engine_url: String,
    status_clear_ms: u32,
    spawn_sidecar: bool,
    spawn_allama: bool,
    config: tauri::State<'_, Mutex<ExodusConfig>>,
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    allama: tauri::State<'_, std::sync::Arc<crate::allama_manager::AllamaManager>>,
) -> Result<(), String> {
    let mut cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
    let port_changed = cfg.ai_port != ai_port;
    let sidecar_pref_changed = cfg.spawn_sidecar != spawn_sidecar;
    let allama_pref_changed = cfg.spawn_allama != spawn_allama;
    cfg.ai_port = ai_port;
    cfg.ai_model = ai_model;
    cfg.embedding_model = if embedding_model.trim().is_empty() {
        ExodusConfig::default_embedding_model()
    } else {
        embedding_model.trim().to_string()
    };
    cfg.homepage_url = if homepage_url.trim().is_empty() {
        ExodusConfig::default_homepage_url()
    } else {
        homepage_url.trim().to_string()
    };
    cfg.search_engine_url = if search_engine_url.trim().is_empty() {
        ExodusConfig::default_search_engine_url()
    } else if search_engine_url.contains("{query}") {
        search_engine_url.trim().to_string()
    } else {
        format!("{}?q={{query}}", search_engine_url.trim().trim_end_matches('?'))
    };
    cfg.status_clear_ms = status_clear_ms.clamp(1000, 60_000);
    cfg.spawn_sidecar = spawn_sidecar;
    cfg.spawn_allama = spawn_allama;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("App data dir error: {}", e))?;
    cfg.save_to(&data_dir)?;
    if port_changed || sidecar_pref_changed {
        let port = cfg.ai_port;
        let enabled = cfg.spawn_sidecar;
        drop(cfg);
        sidecar
            .restart(&app, port, enabled)
            .map_err(|e| format!("Settings saved; sidecar restart failed: {}", e))?;
    } else {
        drop(cfg);
    }

    if port_changed {
        allama.set_port(ai_port);
        if let Some(engine) = app.try_state::<std::sync::Arc<crate::inference_engine::InferenceEngine>>() {
            tauri::async_runtime::block_on(engine.set_allama_http_port(Some(ai_port)));
        }
        if let Some(hermes) = app.try_state::<std::sync::Arc<crate::hermes_agent::HermesAgent>>() {
            tauri::async_runtime::block_on(hermes.set_allama_http_port(ai_port));
        }
        if let Some(py) = app.try_state::<std::sync::Arc<crate::python_microservice::PythonMicroservice>>() {
            tauri::async_runtime::block_on(py.set_allama_port(ai_port));
        }
        if let Some(ext_state) = app.try_state::<crate::plugins::ExtensionState>() {
            if let Ok(mut mgr) = ext_state.lock() {
                mgr.set_allama_http_port(ai_port);
            }
        }
    }

    if port_changed || allama_pref_changed {
        let mgr = allama.inner().clone();
        let start = spawn_allama;
        tauri::async_runtime::spawn(async move {
            if start {
                if let Err(e) = mgr.restart().await {
                    eprintln!("[Allama] restart after settings save failed: {e}");
                }
            } else if let Err(e) = mgr.stop().await {
                eprintln!("[Allama] stop after settings save failed: {e}");
            }
        });
    }
    Ok(())
}

/// Stream chat completion tokens to the frontend via Tauri events.
async fn stream_chat(
    app: AppHandle,
    api_url: String,
    model: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    let body = ChatRequest {
        model,
        stream: true,
        messages,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&api_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let msg = format!("AI API error {}: {}", status, body);
        let _ = app.emit("exodus-ai-error", msg.clone());
        return Err(msg);
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Stream read error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }
            let data = line.trim_start_matches("data: ").trim();
            if data == "[DONE]" {
                let _ = app.emit("exodus-ai-done", ());
                return Ok(());
            }
            if let Ok(parsed) = serde_json::from_str::<StreamChunk>(data) {
                if let Some(choices) = parsed.choices {
                    if let Some(delta) = choices.first().and_then(|c| c.delta.as_ref()) {
                        if let Some(content) = &delta.content {
                            if !content.is_empty() {
                                let _ = app.emit(
                                    "exodus-ai-chunk",
                                    AiChunkPayload {
                                        content: content.clone(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("exodus-ai-done", ());
    Ok(())
}

/// Stream a summary of selected text.
#[tauri::command]
pub async fn ai_summarize_stream(
    app: AppHandle,
    text: String,
    config: tauri::State<'_, Mutex<ExodusConfig>>,
) -> Result<(), String> {
    let (api_url, model) = {
        let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
        (cfg.chat_completions_url(), cfg.ai_model.clone())
    };

    stream_chat(
        app,
        api_url,
        model,
        vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful AI assistant. Provide concise, clear summaries."
                    .to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Please summarize this text:\n\n{}", text),
            },
        ],
    )
    .await
}

/// Stream a chat reply via Tauri events (`exodus-ai-chunk` / `exodus-ai-done`).
///
/// Pass `messages` for multi-turn history, or `prompt` for a single user turn (legacy).
#[tauri::command]
pub async fn ai_chat_stream(
    app: AppHandle,
    prompt: Option<String>,
    messages: Option<Vec<ChatMessage>>,
    config: tauri::State<'_, Mutex<ExodusConfig>>,
) -> Result<(), String> {
    let (api_url, model) = {
        let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
        (cfg.chat_completions_url(), cfg.ai_model.clone())
    };

    let msgs = if let Some(m) = messages {
        if m.is_empty() {
            return Err("messages cannot be empty".to_string());
        }
        m
    } else {
        let p = prompt
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "prompt or messages required".to_string())?;
        vec![
            ChatMessage {
                role: "system".to_string(),
                content: SYSTEM_CHAT.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: p,
            },
        ]
    };

    stream_chat(app, api_url, model, msgs).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExodusConfig::default();
        assert_eq!(config.ai_port, 11435);
        assert_eq!(config.ai_model, "exodus-default");
    }

    #[test]
    fn chat_message_deserializes() {
        let raw = r#"[{"role":"user","content":"hi"}]"#;
        let msgs: Vec<ChatMessage> = serde_json::from_str(raw).expect("parse");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "user");
    }
}
