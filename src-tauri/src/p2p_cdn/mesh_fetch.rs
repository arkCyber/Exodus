//! Exodus P2P CDN — parallel HTTP fetch from peer mesh nodes.

use std::path::Path;

use futures_util::future::join_all;

use super::mesh_ticket::{tickets_from_peers, ExodusCdnTicket};
use super::store::CdnBlobStore;
use super::types::CdnPeerSource;
use crate::microservice::resilience::{RetryConfig, RetryPolicy, retry_with_backoff};

/// Download blob from first available peer over Exodus mesh HTTP.
pub async fn fetch_from_mesh_peers(
    content_hash: &str,
    peers: &[CdnPeerSource],
    dest: &Path,
    store: &CdnBlobStore,
) -> Result<u64, String> {
    let tickets: Vec<ExodusCdnTicket> = tickets_from_peers(peers)
        .into_iter()
        .filter(|t| t.content_hash == content_hash)
        .collect();

    if tickets.is_empty() {
        return Err("No valid exodus-cdn peer tickets".into());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;

    let mut last_err = String::new();
    for ticket in tickets {
        match fetch_from_ticket(&client, &ticket, content_hash, dest, store).await {
            Ok(n) => return Ok(n),
            Err(e) => last_err = e,
        }
    }
    Err(format!("All mesh peers failed: {last_err}"))
}

async fn fetch_from_ticket(
    client: &reqwest::Client,
    ticket: &ExodusCdnTicket,
    content_hash: &str,
    dest: &Path,
    store: &CdnBlobStore,
) -> Result<u64, String> {
    let base = ticket.base_url();
    let meta_url = format!("{base}/blobs/{content_hash}/meta");
    
    let meta_url_clone = meta_url.clone();
    let client_clone = client.clone();
    
    let meta_resp = retry_with_backoff(
        || async {
            let resp = client_clone
                .get(&meta_url_clone)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            if resp.status().is_success() {
                Ok(resp)
            } else {
                Err(format!("HTTP {}", resp.status()))
            }
        },
        RetryPolicy::Transient.to_config(),
    ).await?;
    
    let meta: serde_json::Value = meta_resp.json().await.map_err(|e| e.to_string())?;
    let chunk_count = meta
        .get("chunkCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    
    if chunk_count > 1 {
        return fetch_chunks_parallel(client, &base, content_hash, dest, chunk_count, store)
            .await;
    }

    let url = format!("{base}/blobs/{content_hash}");
    let url_clone = url.clone();
    let client_clone = client.clone();
    
    let resp = retry_with_backoff(
        || async {
            let resp = client_clone
                .get(&url_clone)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            if resp.status().is_success() {
                Ok(resp)
            } else {
                Err(format!("HTTP {}", resp.status()))
            }
        },
        RetryPolicy::Aggressive.to_config(),
    ).await?;
    
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(dest, &bytes).map_err(|e| e.to_string())?;
    verify_and_store(store, content_hash, &bytes)
}

async fn fetch_chunks_parallel(
    client: &reqwest::Client,
    base: &str,
    content_hash: &str,
    dest: &Path,
    chunk_count: u32,
    store: &CdnBlobStore,
) -> Result<u64, String> {
    let fetches: Vec<_> = (0..chunk_count)
        .map(|index| {
            let client = client.clone();
            let url = format!("{base}/blobs/{content_hash}/chunks/{index:06}");
            let url_clone = url.clone();
            async move {
                retry_with_backoff(
                    || async {
                        let resp = client.get(&url_clone).send().await.map_err(|e| e.to_string())?;
                        if !resp.status().is_success() {
                            return Err(format!("chunk {index} HTTP {}", resp.status()));
                        }
                        resp.bytes()
                            .await
                            .map(|b| b.to_vec())
                            .map_err(|e| e.to_string())
                    },
                    RetryPolicy::Transient.to_config(),
                ).await
            }
        })
        .collect();

    let parts = join_all(fetches).await;
    for part in &parts {
        if let Err(e) = part {
            return Err(e.clone());
        }
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut total = 0u64;
    for part in parts {
        let chunk = part.map_err(|e| e)?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        total += chunk.len() as u64;
    }
    let bytes = std::fs::read(dest).map_err(|e| e.to_string())?;
    verify_and_store(store, content_hash, &bytes)?;
    Ok(total)
}

fn verify_and_store(store: &CdnBlobStore, content_hash: &str, bytes: &[u8]) -> Result<u64, String> {
    let (hash, _) = store.put_bytes(bytes)?;
    if hash != content_hash {
        tracing::warn!("mesh peer hash mismatch expected={content_hash} got={hash}");
    }
    Ok(bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::mesh_server::MeshServerHandle;
    use super::super::types::CdnPeerSource;
    use std::sync::Arc;

    #[tokio::test]
    async fn parallel_chunk_fetch_from_mesh() {
        let dir = std::env::temp_dir().join(format!("mesh_fetch_{}", uuid::Uuid::new_v4()));
        let store = Arc::new(CdnBlobStore::new(&dir).expect("Failed to create store"));
        let src = dir.join("big.bin");
        let payload: Vec<u8> = (0..50000).map(|i| (i % 256) as u8).collect();
        std::fs::write(&src, &payload).expect("Failed to write file");
        let (hash, _) = store.import_file(&src).expect("Failed to import file");

        let mesh = MeshServerHandle::start(Arc::clone(&store))
            .await
            .expect("mesh");
        let ticket = ExodusCdnTicket::encode("peer-a", "127.0.0.1", mesh.port, &hash);
        let peers = vec![CdnPeerSource {
            node_id: "peer-a".into(),
            content_hash: hash.clone(),
            ticket: Some(ticket),
            last_seen: 0,
            rtt_ms: None,
        }];

        let dest = dir.join("fetched.bin");
        let n = fetch_from_mesh_peers(&hash, &peers, &dest, &store)
            .await
            .expect("fetch");
        assert_eq!(n, payload.len() as u64);
        assert_eq!(std::fs::read(&dest).expect("Failed to read dest file"), payload);
        mesh.shutdown();
    }

    #[tokio::test]
    async fn rejects_empty_tickets() {
        let store_dir = std::env::temp_dir().join(format!("cdn_fetch_{}", uuid::Uuid::new_v4()));
        let store = CdnBlobStore::new(&store_dir).expect("Failed to create store");
        let dest = store_dir.join("out.bin");
        let err = fetch_from_mesh_peers("abc", &[], &dest, &store)
            .await
            .unwrap_err();
        assert!(err.contains("No valid"));
    }
}
