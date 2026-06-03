//! Exodus P2P CDN — room gossip bridge (iroh-gossip topic compatible).

use std::collections::{HashMap, VecDeque};

use chrono::Utc;

use super::types::{cdn_gossip_topic, CdnAnnouncement, CdnContentKind};

/// In-process gossip bus; mirrors `exodus-cdn-{room_id}` topics for iroh-gossip migration.
pub struct GossipBridge {
    topics: HashMap<String, VecDeque<CdnAnnouncement>>,
}

impl GossipBridge {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    /// Publish seed announcement to a room topic.
    pub fn publish(&mut self, ann: &CdnAnnouncement) {
        let topic = cdn_gossip_topic(&ann.room_id);
        let queue = self.topics.entry(topic).or_default();
        queue.push_back(ann.clone());
        while queue.len() > 500 {
            queue.pop_front();
        }
    }

    /// Drain pending announcements for a room (subscribers consume).
    pub fn drain_room(&mut self, room_id: &str) -> Vec<CdnAnnouncement> {
        let topic = cdn_gossip_topic(room_id);
        let Some(queue) = self.topics.get_mut(&topic) else {
            return Vec::new();
        };
        queue.drain(..).collect()
    }

    /// Serialize announcement for external gossip (`p2p_gossip_publish` payload).
    pub fn announcement_payload(ann: &CdnAnnouncement) -> serde_json::Value {
        serde_json::to_value(ann).unwrap_or(serde_json::Value::Null)
    }

    /// Parse gossip JSON payload back into announcement.
    pub fn announcement_from_payload(value: &serde_json::Value) -> Option<CdnAnnouncement> {
        serde_json::from_value(value.clone()).ok()
    }

    /// Parse AI recommendation payload into CDN announcement.
    pub fn from_ai_recommendation(
        room_id: &str,
        node_id: &str,
        payload: &serde_json::Value,
    ) -> Option<CdnAnnouncement> {
        let content_hash = payload
            .get("contentHash")
            .or_else(|| payload.get("content_hash"))
            .and_then(|v| v.as_str())?
            .to_string();
        let title = payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Shared content")
            .to_string();
        let kind = payload
            .get("kind")
            .and_then(|v| v.as_str())
            .map(parse_kind)
            .unwrap_or(CdnContentKind::GenericFile);
        let size_bytes = payload
            .get("sizeBytes")
            .or_else(|| payload.get("size_bytes"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Some(CdnAnnouncement {
            room_id: room_id.to_string(),
            content_hash,
            node_id: node_id.to_string(),
            title,
            kind,
            size_bytes,
            mime_type: payload.get("mimeType").and_then(|v| v.as_str()).map(String::from),
            source_url: payload
                .get("sourceUrl")
                .or_else(|| payload.get("source_url"))
                .and_then(|v| v.as_str())
                .map(String::from),
            ticket: None,
            timestamp: Utc::now().timestamp_millis() as u64,
        })
    }
}

fn parse_kind(raw: &str) -> CdnContentKind {
    match raw.to_ascii_lowercase().as_str() {
        "article" | "longread" => CdnContentKind::Article,
        "ai_model" | "aimodel" | "llm" => CdnContentKind::AiModel,
        "video_model" | "videomodel" => CdnContentKind::VideoModel,
        "dataset" => CdnContentKind::Dataset,
        _ => CdnContentKind::GenericFile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn publish_and_drain_room() {
        let mut g = GossipBridge::new();
        let ann = CdnAnnouncement {
            room_id: "lobby".into(),
            content_hash: "abc".into(),
            node_id: "n1".into(),
            title: "Model".into(),
            kind: CdnContentKind::AiModel,
            size_bytes: 5_000_000_000,
            mime_type: None,
            source_url: None,
            ticket: None,
            timestamp: 1,
        };
        g.publish(&ann);
        let drained = g.drain_room("lobby");
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].content_hash, "abc");
    }

    #[test]
    fn announcement_json_roundtrip() {
        let ann = CdnAnnouncement {
            room_id: "lobby".into(),
            content_hash: "abc".into(),
            node_id: "n1".into(),
            title: "T".into(),
            kind: CdnContentKind::Article,
            size_bytes: 1,
            mime_type: None,
            source_url: None,
            ticket: None,
            timestamp: 2,
        };
        let payload = GossipBridge::announcement_payload(&ann);
        let back = GossipBridge::announcement_from_payload(&payload).expect("back");
        assert_eq!(back.content_hash, "abc");
    }

    #[test]
    fn parse_ai_recommendation_payload() {
        let ann = GossipBridge::from_ai_recommendation(
            "group-1",
            "node-x",
            &json!({
                "contentHash": "deadbeef",
                "title": "Llama 8B GGUF",
                "kind": "ai_model",
                "sizeBytes": 5000000000_i64
            }),
        )
        .expect("parse");
        assert_eq!(ann.kind, CdnContentKind::AiModel);
        assert_eq!(ann.size_bytes, 5_000_000_000);
    }
}
