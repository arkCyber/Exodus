//! Exodus P2P CDN — shared types (iroh-blobs BLAKE3 content addressing).

use serde::{Deserialize, Serialize};

/// Content kind for AI browser recommendations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CdnContentKind {
    Article,
    AiModel,
    VideoModel,
    Dataset,
    GenericFile,
}

/// Metadata for a content-addressed asset in a room or lobby.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnAsset {
    pub content_hash: String,
    pub title: String,
    pub kind: CdnContentKind,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub source_url: Option<String>,
    pub room_id: String,
    pub announcer_node_id: String,
    pub announced_at: u64,
}

/// Gossip payload: a peer has (part or all of) a blob and can serve it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnAnnouncement {
    pub room_id: String,
    pub content_hash: String,
    pub node_id: String,
    pub title: String,
    pub kind: CdnContentKind,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub source_url: Option<String>,
    pub ticket: Option<String>,
    pub timestamp: u64,
}

/// Known peer that can serve a hash (from gossip or local registry).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnPeerSource {
    pub node_id: String,
    pub content_hash: String,
    pub ticket: Option<String>,
    pub last_seen: u64,
    pub rtt_ms: Option<u32>,
}

/// Active or completed download job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnDownloadJob {
    pub job_id: String,
    pub content_hash: String,
    pub title: String,
    pub status: String,
    pub progress_percent: f32,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub source: String,
    pub peer_count: u32,
    pub local_path: Option<String>,
    pub error: Option<String>,
}

/// Progress event payload for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnDownloadProgress {
    pub job_id: String,
    pub content_hash: String,
    pub progress_percent: f32,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub source: String,
    pub peer_count: u32,
}

/// Room feed: trending assets + peer availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnRoomFeed {
    pub room_id: String,
    pub assets: Vec<CdnAsset>,
    pub peer_map: std::collections::HashMap<String, Vec<CdnPeerSource>>,
}

/// Topic name for gossip (iroh-gossip compatible naming).
pub fn cdn_gossip_topic(room_id: &str) -> String {
    format!("exodus-cdn-{room_id}")
}
