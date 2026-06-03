//! Exodus P2P CDN — Tauri commands for the AI browser.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use chrono::Utc;

use crate::microservice::group_chat_client::group_chat_json_rpc;
use crate::microservice::{GroupChatServiceConfig, GroupMessage};

use super::download::start_cdn_download;
use super::gossip_bridge::GossipBridge;
use super::swarm::P2pCdnState;
use super::types::{CdnAnnouncement, CdnAsset, CdnContentKind, CdnDownloadJob, CdnRoomFeed};

/// Swarm / cache status for a URL (discovery hash = BLAKE3(url)).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnUrlStatus {
    pub url: String,
    pub discovery_hash: String,
    pub announced: bool,
    pub peer_count: u32,
    pub local_complete: bool,
    pub title: Option<String>,
}

/// Node info for UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnNodeInfo {
    pub node_id: String,
    pub joined_rooms: Vec<String>,
    pub mesh_host: Option<String>,
    pub mesh_port: Option<u16>,
}

#[tauri::command]
pub async fn p2p_cdn_start_mesh(state: State<'_, Arc<P2pCdnState>>) -> Result<CdnNodeInfo, String> {
    let (host, port) = state.ensure_mesh().await?;
    Ok(node_info(&state, Some(host), Some(port)))
}

#[tauri::command]
pub fn p2p_cdn_node_info(state: State<'_, Arc<P2pCdnState>>) -> Result<CdnNodeInfo, String> {
    let ep = state.mesh_endpoint();
    Ok(node_info(
        &state,
        ep.as_ref().map(|e| e.0.clone()),
        ep.map(|e| e.1),
    ))
}

fn node_info(state: &P2pCdnState, mesh_host: Option<String>, mesh_port: Option<u16>) -> CdnNodeInfo {
    let joined_rooms = state.joined_rooms_list().unwrap_or_default();
    CdnNodeInfo {
        node_id: state.node_id.clone(),
        joined_rooms,
        mesh_host,
        mesh_port,
    }
}

#[tauri::command]
pub async fn p2p_cdn_join_room(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
) -> Result<(), String> {
    state.join_room(&room_id)?;
    let node_id = state.node_id.clone();
    let _ = super::external_gossip::try_subscribe_room(&node_id, &room_id).await;
    Ok(())
}

#[tauri::command]
pub fn p2p_cdn_leave_room(state: State<'_, Arc<P2pCdnState>>, room_id: String) -> Result<(), String> {
    state.leave_room(&room_id)
}

#[tauri::command]
pub async fn p2p_cdn_room_feed(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
) -> Result<CdnRoomFeed, String> {
    state.room_feed_async(&room_id).await
}

/// Sync gossip (in-process + external microservice) for a room.
#[tauri::command]
pub async fn p2p_cdn_sync_gossip(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
) -> Result<usize, String> {
    let external = state.pull_external_gossip(&room_id).await?;
    let local = state.sync_gossip()?;
    Ok(external + local)
}

/// BLAKE3 hash + size for a local file (before announce / seed).
#[tauri::command]
pub fn p2p_cdn_hash_file(local_path: String) -> Result<serde_json::Value, String> {
    let (content_hash, size_bytes) =
        super::store::CdnBlobStore::hash_file(PathBuf::from(&local_path).as_path())?;
    Ok(serde_json::json!({
        "contentHash": content_hash,
        "sizeBytes": size_bytes,
    }))
}

#[tauri::command]
pub fn p2p_cdn_list_peers(
    state: State<'_, Arc<P2pCdnState>>,
    content_hash: String,
) -> Result<Vec<super::types::CdnPeerSource>, String> {
    state.list_peers(&content_hash)
}

#[tauri::command]
pub async fn p2p_cdn_announce_asset(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    payload: serde_json::Value,
) -> Result<CdnAnnouncement, String> {
    let ann = GossipBridge::from_ai_recommendation(&room_id, &state.node_id, &payload)
        .ok_or_else(|| "Invalid AI CDN payload".to_string())?;
    state.ingest_announcement(ann.clone())?;
    let mut gossip = state.gossip_mut()?;
    gossip.publish(&ann);
    Ok(ann)
}

/// Group chat / lobby: announce attachment or hot link for P2P CDN.
#[tauri::command]
pub async fn p2p_cdn_announce_group_hot(
    state: State<'_, Arc<P2pCdnState>>,
    group_id: String,
    title: String,
    content_hash: String,
    kind: String,
    size_bytes: u64,
    source_url: Option<String>,
    local_path: Option<String>,
) -> Result<CdnAnnouncement, String> {
    let kind = parse_kind(&kind);
    if let Some(path) = local_path {
        let (hash, size) = state.store().import_file(PathBuf::from(path).as_path())?;
        return state
            .register_local_seed(&group_id, &hash, &title, kind, size, source_url)
            .await;
    }
    let mut ann = CdnAnnouncement {
        room_id: group_id.clone(),
        content_hash: content_hash.clone(),
        node_id: state.node_id.clone(),
        title,
        kind,
        size_bytes,
        mime_type: None,
        source_url,
        ticket: None,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    if state.store().has_complete(&content_hash) {
        ann.ticket = state.make_ticket(&content_hash).await.ok();
    }
    state.announce_and_publish(ann).await
}

#[tauri::command]
pub async fn p2p_cdn_register_local_seed(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    local_path: String,
    title: String,
    kind: String,
    source_url: Option<String>,
) -> Result<CdnAnnouncement, String> {
    let path = PathBuf::from(&local_path);
    let (hash, size) = state.store().import_file(&path)?;
    let kind = parse_kind(&kind);
    state
        .register_local_seed(&room_id, &hash, &title, kind, size, source_url)
        .await
}

#[tauri::command]
pub async fn p2p_cdn_download(
    app: AppHandle,
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    content_hash: String,
    title: String,
    kind: String,
    http_url: Option<String>,
) -> Result<CdnDownloadJob, String> {
    let _ = state.ensure_mesh().await;
    let downloads_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Download dir: {e}"))?;
    std::fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;
    let p2p_dir = downloads_dir.join("exodus-p2p-cdn");
    std::fs::create_dir_all(&p2p_dir).map_err(|e| e.to_string())?;

    start_cdn_download(
        app,
        Arc::clone(&state.inner()),
        room_id,
        content_hash,
        title,
        parse_kind(&kind),
        http_url,
        p2p_dir,
    )
    .await
}

/// Lookup P2P CDN status for a page URL in a room (address bar badge).
#[tauri::command]
pub async fn p2p_cdn_url_status(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    url: String,
) -> Result<CdnUrlStatus, String> {
    let discovery_hash = blake3::hash(url.as_bytes()).to_hex().to_string();
    let _ = state.join_room(&room_id);
    let _ = state.pull_external_gossip(&room_id).await;
    let asset = state.get_asset(&room_id, &discovery_hash);
    let peers = state.list_peers(&discovery_hash)?;
    let external_peers = peers
        .iter()
        .filter(|p| p.node_id != state.node_id)
        .count() as u32;
    Ok(CdnUrlStatus {
        url,
        discovery_hash: discovery_hash.clone(),
        announced: asset.is_some(),
        peer_count: external_peers,
        local_complete: state.store().has_complete(&discovery_hash),
        title: asset.map(|a| a.title),
    })
}

/// Announce a large page / model URL to a room (discovery hash = BLAKE3(url); download verifies file hash).
#[tauri::command]
pub async fn p2p_cdn_announce_url_hot(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    title: String,
    url: String,
    kind: String,
    size_bytes: Option<u64>,
) -> Result<CdnAnnouncement, String> {
    let content_hash = blake3::hash(url.as_bytes()).to_hex().to_string();
    let kind = parse_kind(&kind);
    let _ = state.join_room(&room_id);
    let ann = CdnAnnouncement {
        room_id: room_id.clone(),
        content_hash,
        node_id: state.node_id.clone(),
        title,
        kind,
        size_bytes: size_bytes.unwrap_or(0),
        mime_type: None,
        source_url: Some(url),
        ticket: None,
        timestamp: Utc::now().timestamp_millis() as u64,
    };
    state.announce_and_publish(ann).await
}

/// Send a group chat message and announce each attachment to the group's P2P CDN room.
#[tauri::command]
pub async fn p2p_cdn_group_send_message(
    state: State<'_, Arc<P2pCdnState>>,
    message: GroupMessage,
) -> Result<String, String> {
    let config = GroupChatServiceConfig::default();
    let params = serde_json::to_value(&message).map_err(|e| e.to_string())?;
    let result =
        group_chat_json_rpc(&config.socket_path, "send_message", params).await?;
    let message_id = result
        .get("message_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid message_id response".to_string())?;

    let _ = state.join_room(&message.group_id);
    for att in &message.attachments {
        if att.blob_hash.is_empty() {
            continue;
        }
        let kind = attachment_kind(&att.file_type);
        let mut ann = CdnAnnouncement {
            room_id: message.group_id.clone(),
            content_hash: att.blob_hash.clone(),
            node_id: state.node_id.clone(),
            title: att.file_name.clone(),
            kind,
            size_bytes: att.file_size,
            mime_type: Some(att.file_type.clone()),
            source_url: None,
            ticket: None,
            timestamp: Utc::now().timestamp_millis() as u64,
        };
        if state.store().has_complete(&att.blob_hash) {
            ann.ticket = state.make_ticket(&att.blob_hash).await.ok();
        }
        let _ = state.announce_and_publish(ann).await;
    }

    Ok(message_id)
}

#[tauri::command]
pub fn p2p_cdn_get_asset(
    state: State<'_, Arc<P2pCdnState>>,
    room_id: String,
    content_hash: String,
) -> Result<Option<CdnAsset>, String> {
    Ok(state.get_asset(&room_id, &content_hash))
}

fn parse_kind(kind: &str) -> CdnContentKind {
    match kind.to_ascii_lowercase().as_str() {
        "article" => CdnContentKind::Article,
        "ai_model" | "aimodel" => CdnContentKind::AiModel,
        "video_model" | "videomodel" => CdnContentKind::VideoModel,
        "dataset" => CdnContentKind::Dataset,
        _ => CdnContentKind::GenericFile,
    }
}

fn attachment_kind(file_type: &str) -> CdnContentKind {
    match file_type.to_ascii_lowercase().as_str() {
        "image" => CdnContentKind::GenericFile,
        "video" => CdnContentKind::VideoModel,
        "audio" => CdnContentKind::GenericFile,
        "model" | "ai_model" => CdnContentKind::AiModel,
        _ => CdnContentKind::GenericFile,
    }
}
