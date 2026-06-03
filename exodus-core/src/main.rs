//! Exodus Core — minimal OpenAI-compatible sidecar for local dev.
//! Serves `/v1/models`, `/v1/embeddings`, and streaming `/v1/chat/completions`.

use std::time::Duration;

use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

/// Embedding vector size (matches common Ollama embedding models).
const EMBED_DIM: usize = 768;

/// Log with UTC timestamp.
fn log(msg: &str) {
    println!(
        "[{}] exodus-core: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
        msg
    );
}

/// Parse `--port <n>` from argv (default 11434).
fn parse_port(args: &[String]) -> u16 {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" {
            if let Some(next) = args.get(i + 1) {
                if let Ok(p) = next.parse::<u16>() {
                    return p;
                }
            }
        }
    }
    11434
}

#[derive(Debug, Deserialize)]
struct EmbeddingRequest {
    model: Option<String>,
    input: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    #[allow(dead_code)]
    model: Option<String>,
    messages: Option<Vec<ChatMessage>>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Deterministic pseudo-embedding for dev (normalized hash projection).
fn pseudo_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0f32; EMBED_DIM];
    for (i, b) in text.as_bytes().iter().enumerate() {
        let idx = i % EMBED_DIM;
        vec[idx] += (*b as f32 + 1.0) / 256.0;
    }
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vec {
            *x /= norm;
        }
    }
    vec
}

fn embedding_input_string(input: &serde_json::Value) -> String {
    match input {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>()
            .join(" "),
        _ => input.to_string(),
    }
}

fn handle_models() -> Response<std::io::Cursor<Vec<u8>>> {
    let body = json!({
        "object": "list",
        "data": [
            { "id": "llama2", "object": "model", "owned_by": "exodus" },
            { "id": "nomic-embed-text", "object": "model", "owned_by": "exodus" }
        ]
    });
    json_response(StatusCode(200), &body)
}

fn handle_embeddings(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let req: EmbeddingRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return error_response(StatusCode(400), &format!("Invalid JSON: {}", e)),
    };
    let text = embedding_input_string(&req.input);
    if text.trim().is_empty() {
        return error_response(StatusCode(400), "input cannot be empty");
    }
    let model = req.model.unwrap_or_else(|| "nomic-embed-text".into());
    let vector = pseudo_embedding(&text);
    let body = json!({
        "object": "list",
        "data": [{ "object": "embedding", "index": 0, "embedding": vector }],
        "model": model
    });
    json_response(StatusCode(200), &body)
}

fn handle_chat(body: &str, port: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let req: ChatRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return error_response(StatusCode(400), &format!("Invalid JSON: {}", e)),
    };
    let stream = req.stream.unwrap_or(false);
    let user_text = req
        .messages
        .as_ref()
        .and_then(|m| m.iter().rfind(|x| x.role == "user"))
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let reply = if user_text.trim().is_empty() {
        "Exodus Core (dev) is running. Use Ollama on this port for full models.".to_string()
    } else {
        format!(
            "[Exodus Core dev] {} chars received. For real LLM replies, run Ollama on port {}.\n\nPreview: {}",
            user_text.len(),
            port,
            user_text.chars().take(200).collect::<String>()
        )
    };

    if stream {
        stream_chat_response(&reply)
    } else {
        let body = json!({
            "id": "exodus-dev",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": reply },
                "finish_reason": "stop"
            }]
        });
        json_response(StatusCode(200), &body)
    }
}

fn stream_chat_response(text: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut payload = String::new();
    for word in text.split_whitespace() {
        let chunk = json!({
            "choices": [{ "delta": { "content": format!("{} ", word) } }]
        });
        payload.push_str(&format!("data: {}\n\n", chunk));
    }
    payload.push_str("data: [DONE]\n\n");

    Response::from_string(payload)
        .with_status_code(StatusCode(200))
        .with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap(),
        )
}

fn json_response(status: StatusCode, value: &serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    match serde_json::to_vec(value) {
        Ok(bytes) => Response::from_data(bytes)
            .with_status_code(status)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()),
        Err(e) => error_response(StatusCode(500), &format!("Serialize error: {}", e)),
    }
}

fn error_response(status: StatusCode, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = json!({ "error": { "message": message } });
    json_response(status, &body)
}

fn handle_request(mut request: Request, port: u16) {
    let path = request.url().to_string();
    let path_only = path.split('?').next().unwrap_or(&path).to_string();
    let method = request.method().clone();

    let mut body = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut body) {
        let _ = request.respond(error_response(
            StatusCode(400),
            &format!("Read body failed: {}", e),
        ));
        return;
    }

    let response = match (method, path_only.as_str()) {
        (Method::Get, "/v1/models") => handle_models(),
        (Method::Post, "/v1/embeddings") => handle_embeddings(&body),
        (Method::Post, "/v1/chat/completions") => handle_chat(&body, port),
        (Method::Get, "/health") => json_response(StatusCode(200), &json!({ "status": "ok" })),
        _ => error_response(StatusCode(404), "Not found"),
    };

    let _ = request.respond(response);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = parse_port(&args);
    let addr = format!("127.0.0.1:{}", port);

    log(&format!("Starting dev API on http://{}", addr));

    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("exodus-core: bind {} failed: {}", addr, e);
            std::process::exit(1);
        }
    };

    loop {
        match server.recv_timeout(Duration::from_secs(1)) {
            Ok(Some(request)) => handle_request(request, port),
            Ok(None) => {}
            Err(e) => {
                eprintln!("exodus-core: server error: {}", e);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pseudo_embedding_normalized() {
        let v = pseudo_embedding("hello world");
        assert_eq!(v.len(), EMBED_DIM);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
}
