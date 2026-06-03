//! Exodus Browser — vector embeddings via OpenAI-compatible API (Ollama / exodus-core).

use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ExodusConfig;
use crate::rag::WebPage;
use crate::microservice::resilience::{RetryConfig, RetryPolicy, retry_with_backoff};

/// Log module init with UTC timestamp.
fn log_embeddings_init() {
    println!(
        "[{}] embeddings module loaded",
        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
    );
}

#[derive(Debug, Clone, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Option<Vec<EmbeddingData>>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// Whether the local embeddings endpoint responds.
pub async fn embeddings_available(config: &ExodusConfig) -> bool {
    log_embeddings_init();
    let url = format!("http://127.0.0.1:{}/v1/embeddings", config.ai_port);
    let client = match Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let body = EmbeddingRequest {
        model: config.embedding_model.clone(),
        input: "ping".to_string(),
    };

    match client.post(&url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => resp
            .json::<EmbeddingResponse>()
            .await
            .ok()
            .and_then(|r| r.data)
            .map(|d| !d.is_empty() && !d[0].embedding.is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

/// Fetch a single embedding vector for text (title + content snippet).
pub async fn fetch_embedding(config: &ExodusConfig, text: &str) -> Result<Vec<f32>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Empty text for embedding".to_string());
    }

    let url = format!("http://127.0.0.1:{}/v1/embeddings", config.ai_port);
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let body = EmbeddingRequest {
        model: config.embedding_model.clone(),
        input: trimmed.chars().take(8000).collect::<String>(),
    };

    let url_clone = url.clone();
    let client_clone = client.clone();
    let body_clone = body.clone();

    let resp = retry_with_backoff(
        || async {
            let resp = client_clone
                .post(&url_clone)
                .json(&body_clone)
                .send()
                .await
                .map_err(|e| format!("Embeddings request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_body = resp.text().await.unwrap_or_default();
                return Err(format!("Embeddings API {}: {}", status, err_body));
            }

            Ok(resp)
        },
        RetryPolicy::Transient.to_config(),
    ).await?;

    let parsed: EmbeddingResponse = resp
        .json()
        .await
        .map_err(|e| format!("Invalid embeddings JSON: {}", e))?;

    parsed
        .data
        .and_then(|mut d| d.pop())
        .map(|row| row.embedding)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Embeddings API returned no vectors".to_string())
}

/// Cosine similarity between two vectors (0..1 when normalized).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Build text used for page embedding (title + content excerpt).
pub fn embed_text_for_page(title: &str, content: &str) -> String {
    let excerpt: String = content.chars().take(4000).collect();
    format!("{}\n\n{}", title.trim(), excerpt.trim())
}

/// Score a stored page embedding against a query vector.
pub fn score_page_embedding(page: &WebPage, query_embedding: &[f32]) -> f32 {
    page.embedding
        .as_ref()
        .map(|emb| cosine_similarity(emb, query_embedding))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 0.001);
    }

    #[test]
    fn cosine_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);
    }

    #[test]
    fn embed_text_for_page_includes_title() {
        let text = embed_text_for_page("My Title", "body content");
        assert!(text.contains("My Title"));
        assert!(text.contains("body content"));
    }

    #[test]
    fn score_page_embedding_without_vector_is_zero() {
        use crate::rag::WebPage;
        use chrono::Utc;
        let page = WebPage {
            id: "1".into(),
            url: "https://x.com".into(),
            title: "X".into(),
            content: "".into(),
            timestamp: Utc::now(),
            embedding: None,
        };
        assert_eq!(score_page_embedding(&page, &[1.0, 0.0]), 0.0);
    }
}
