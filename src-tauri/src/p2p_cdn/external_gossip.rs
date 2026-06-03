//! Exodus P2P CDN — bridge to `p2p_gossip` microservice (`exodus-cdn-{room}` topics).

use chrono::Utc;
use serde_json::json;

use crate::microservice::gossip_client::gossip_json_rpc;
use crate::microservice::P2pGossipConfig;

use super::gossip_bridge::GossipBridge;
use super::types::{cdn_gossip_topic, CdnAnnouncement};

/// Whether the gossip socket is reachable (service started).
pub fn gossip_service_available() -> bool {
    P2pGossipConfig::default().socket_path.exists()
}

/// Subscribe this CDN node to a room topic (best-effort).
pub async fn try_subscribe_room(node_id: &str, room_id: &str) -> Result<(), String> {
    let config = P2pGossipConfig::default();
    if !config.socket_path.exists() {
        return Ok(());
    }
    let topic = cdn_gossip_topic(room_id);
    let subscriber_id = format!("cdn-{node_id}");
    gossip_json_rpc(
        &config.socket_path,
        "subscribe",
        json!({ "topic": topic, "subscriber_id": subscriber_id }),
    )
    .await?;
    tracing::info!(
        "[{}] P2P CDN subscribed to gossip topic={}",
        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
        topic
    );
    Ok(())
}

/// Publish a CDN announcement to the gossip overlay (best-effort).
pub async fn try_publish(ann: &CdnAnnouncement) -> Result<(), String> {
    let config = P2pGossipConfig::default();
    if !config.socket_path.exists() {
        return Ok(());
    }
    let topic = cdn_gossip_topic(&ann.room_id);
    gossip_json_rpc(
        &config.socket_path,
        "publish",
        json!({
            "topic": topic,
            "payload": GossipBridge::announcement_payload(ann),
        }),
    )
    .await?;
    Ok(())
}

/// Pull recent CDN announcements from gossip for a room.
pub async fn try_pull_room(room_id: &str, limit: usize) -> Result<Vec<CdnAnnouncement>, String> {
    let config = P2pGossipConfig::default();
    if !config.socket_path.exists() {
        return Ok(Vec::new());
    }
    let topic = cdn_gossip_topic(room_id);
    let result = gossip_json_rpc(
        &config.socket_path,
        "get_messages",
        json!({ "topic": topic, "limit": limit }),
    )
    .await?;
    let messages = result
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for msg in messages {
        let payload = msg.get("payload").unwrap_or(&msg);
        if let Some(ann) = GossipBridge::announcement_from_payload(payload) {
            out.push(ann);
        }
    }
    Ok(out)
}
