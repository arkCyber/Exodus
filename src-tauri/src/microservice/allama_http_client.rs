//! Exodus Browser — HTTP client for the Allama Ollama-compatible API (port 11435).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::resilience::{RetryConfig, RetryPolicy, retry_with_backoff};

/// Default Allama HTTP base URL (Ollama replacement).
pub fn default_allama_base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

/// Client for `/api/*` and `/v1/*` on the Allama server.
#[derive(Debug, Clone)]
pub struct AllamaHttpClient {
    base_url: String,
    http: reqwest::Client,
}

impl AllamaHttpClient {
    /// Build a client for `http://127.0.0.1:{port}`.
    pub fn from_port(port: u16) -> Self {
        Self::new(default_allama_base_url(port))
    }

    /// Build a client with an explicit base URL (no trailing slash).
    pub fn new(base_url: impl Into<String>) -> Self {
        let base = base_url.into().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: base,
            http,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// `GET /api/tags` — returns true when the server responds successfully.
    pub async fn probe(&self) -> bool {
        self.http
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// `GET /api/tags` — model names.
    pub async fn list_model_names(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);
        let http = self.http.clone();

        let resp = retry_with_backoff(
            || async {
                let resp = http
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("Allama tags request failed: {e}"))?;
                
                if !resp.status().is_success() {
                    return Err(format!("Allama tags HTTP {}", resp.status()));
                }
                
                Ok(resp)
            },
            RetryPolicy::Transient.to_config(),
        ).await?;

        let body: TagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Allama tags parse failed: {e}"))?;
        Ok(body
            .models
            .into_iter()
            .map(|m| m.name)
            .collect())
    }

    /// `POST /api/generate` (non-streaming).
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<String, String> {
        let mut payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });
        if let Some(n) = max_tokens {
            payload["options"] = json!({ "num_predict": n });
        }
        if let Some(t) = temperature {
            if payload.get("options").is_none() {
                payload["options"] = json!({});
            }
            payload["options"]["temperature"] = json!(t);
        }

        let url = format!("{}/api/generate", self.base_url);
        let http = self.http.clone();
        let payload_clone = payload.clone();

        let resp = retry_with_backoff(
            || async {
                let resp = http
                    .post(&url)
                    .json(&payload_clone)
                    .send()
                    .await
                    .map_err(|e| format!("Allama generate failed: {e}"))?;
                
                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Allama generate HTTP {status}: {text}"));
                }
                
                Ok(resp)
            },
            RetryPolicy::Aggressive.to_config(),
        ).await?;

        let body: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| format!("Allama generate parse failed: {e}"))?;
        Ok(body.response)
    }

    /// `POST /api/chat` (non-streaming).
    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<String, String> {
        let msgs: Vec<Value> = messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();
        let mut payload = json!({
            "model": model,
            "messages": msgs,
            "stream": false,
        });
        if let Some(n) = max_tokens {
            payload["options"] = json!({ "num_predict": n });
        }
        if let Some(t) = temperature {
            if payload.get("options").is_none() {
                payload["options"] = json!({});
            }
            payload["options"]["temperature"] = json!(t);
        }

        let url = format!("{}/api/chat", self.base_url);
        let http = self.http.clone();
        let payload_clone = payload.clone();

        let resp = retry_with_backoff(
            || async {
                let resp = http
                    .post(&url)
                    .json(&payload_clone)
                    .send()
                    .await
                    .map_err(|e| format!("Allama chat failed: {e}"))?;
                
                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Allama chat HTTP {status}: {text}"));
                }
                
                Ok(resp)
            },
            RetryPolicy::Aggressive.to_config(),
        ).await?;

        let body: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("Allama chat parse failed: {e}"))?;
        Ok(body.message.content)
    }

    /// `POST /api/embeddings`.
    pub async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>, String> {
        let payload = json!({
            "model": model,
            "input": text,
        });

        let url = format!("{}/api/embeddings", self.base_url);
        let http = self.http.clone();
        let payload_clone = payload.clone();

        let resp = retry_with_backoff(
            || async {
                let resp = http
                    .post(&url)
                    .json(&payload_clone)
                    .send()
                    .await
                    .map_err(|e| format!("Allama embed failed: {e}"))?;
                
                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Allama embed HTTP {status}: {text}"));
                }
                
                Ok(resp)
            },
            RetryPolicy::Transient.to_config(),
        ).await?;

        let body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| format!("Allama embed parse failed: {e}"))?;
        
        Ok(body.embedding)
    }

    async fn embed_openai(&self, model: &str, text: &str) -> Result<Vec<f32>, String> {
        let payload = json!({
            "model": model,
            "input": text,
        });

        let url = format!("{}/v1/embeddings", self.base_url);
        let http = self.http.clone();
        let payload_clone = payload.clone();

        let resp = retry_with_backoff(
            || async {
                let resp = http
                    .post(&url)
                    .json(&payload_clone)
                    .send()
                    .await
                    .map_err(|e| format!("Allama OpenAI embed failed: {e}"))?;
                
                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Allama embed HTTP {status}: {text}"));
                }
                
                Ok(resp)
            },
            RetryPolicy::Transient.to_config(),
        ).await?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("Allama embed parse failed: {e}"))?;
        let embedding = body
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|o| o.get("embedding"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| "missing embedding in response".to_string())?;
        Ok(embedding.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_uses_port() {
        assert_eq!(default_allama_base_url(11435), "http://127.0.0.1:11435");
    }

    #[test]
    fn client_strips_trailing_slash() {
        let c = AllamaHttpClient::new("http://127.0.0.1:11435/");
        assert_eq!(c.base_url(), "http://127.0.0.1:11435");
    }
}
