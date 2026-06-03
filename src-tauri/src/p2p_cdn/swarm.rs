//! Exodus P2P CDN — in-memory swarm index and room state.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;

use super::external_gossip;
use super::gossip_bridge::GossipBridge;
use super::mesh_server::MeshServerHandle;
use super::mesh_ticket::ExodusCdnTicket;
use super::store::CdnBlobStore;
use super::types::{
    CdnAnnouncement, CdnAsset, CdnContentKind, CdnDownloadJob, CdnPeerSource, CdnRoomFeed,
};

/// Shared P2P CDN hub (Tauri managed state).
pub struct P2pCdnState {
    pub node_id: String,
    store: Arc<CdnBlobStore>,
    gossip: Mutex<GossipBridge>,
    mesh: Mutex<Option<MeshServerHandle>>,
    assets_by_room: Mutex<HashMap<String, HashMap<String, CdnAsset>>>,
    peers_by_hash: Mutex<HashMap<String, Vec<CdnPeerSource>>>,
    joined_rooms: Mutex<HashSet<String>>,
    jobs: Mutex<HashMap<String, CdnDownloadJob>>,
}

impl P2pCdnState {
    /// Create hub with persistent blob store under app data.
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let store_dir = app_data_dir.join("p2p_cdn").join("blobs");
        let store = Arc::new(CdnBlobStore::new(&store_dir)?);
        let node_id = format!(
            "exodus-{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string()
        );
        tracing::info!(
            "[{}] P2P CDN hub ready node_id={}",
            Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
            node_id
        );
        Ok(Self {
            node_id,
            store,
            gossip: Mutex::new(GossipBridge::new()),
            mesh: Mutex::new(None),
            assets_by_room: Mutex::new(HashMap::new()),
            peers_by_hash: Mutex::new(HashMap::new()),
            joined_rooms: Mutex::new(HashSet::new()),
            jobs: Mutex::new(HashMap::new()),
        })
    }

    pub fn store(&self) -> &CdnBlobStore {
        &self.store
    }

    #[allow(dead_code)]
    pub fn store_arc(&self) -> Arc<CdnBlobStore> {
        Arc::clone(&self.store)
    }

    pub fn gossip_mut(&self) -> Result<std::sync::MutexGuard<'_, GossipBridge>, String> {
        self.gossip.lock().map_err(|e| format!("Gossip lock: {e}"))
    }

    /// Start HTTP mesh server for peer uploads (idempotent).
    pub async fn ensure_mesh(&self) -> Result<(String, u16), String> {
        if let Some((host, port)) = self.mesh_endpoint() {
            return Ok((host, port));
        }
        let handle = MeshServerHandle::start(Arc::clone(&self.store)).await?;
        let host = handle.host.clone();
        let port = handle.port;
        {
            let mut guard = self.mesh.lock().map_err(|e| e.to_string())?;
            *guard = Some(handle);
        }
        Ok((host, port))
    }

    /// Mesh listen endpoint for UI.
    pub fn mesh_endpoint(&self) -> Option<(String, u16)> {
        let guard = self.mesh.lock().ok()?;
        guard.as_ref().map(|h| (h.host.clone(), h.port))
    }

    /// Build exodus-cdn ticket after mesh is up.
    pub async fn make_ticket(&self, content_hash: &str) -> Result<String, String> {
        let (host, port) = self.ensure_mesh().await?;
        Ok(ExodusCdnTicket::encode(&self.node_id, &host, port, content_hash))
    }

    /// Join a room topic (lobby or group chat id).
    pub fn join_room(&self, room_id: &str) -> Result<(), String> {
        let mut rooms = self.joined_rooms.lock().map_err(|e| e.to_string())?;
        rooms.insert(room_id.to_string());
        Ok(())
    }

    pub fn leave_room(&self, room_id: &str) -> Result<(), String> {
        let mut rooms = self.joined_rooms.lock().map_err(|e| e.to_string())?;
        rooms.remove(room_id);
        Ok(())
    }

    pub fn joined_rooms_list(&self) -> Result<Vec<String>, String> {
        Ok(self
            .joined_rooms
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .cloned()
            .collect())
    }

    /// Ingest + in-process gossip + external gossip overlay publish.
    pub async fn announce_and_publish(&self, ann: CdnAnnouncement) -> Result<CdnAnnouncement, String> {
        self.ingest_announcement(ann.clone())?;
        {
            let mut gossip = self.gossip_mut()?;
            gossip.publish(&ann);
        }
        let _ = external_gossip::try_publish(&ann).await;
        Ok(ann)
    }

    /// Pull CDN announcements from the gossip microservice for a room.
    pub async fn pull_external_gossip(&self, room_id: &str) -> Result<usize, String> {
        let anns = external_gossip::try_pull_room(room_id, 200).await?;
        let mut count = 0usize;
        for ann in anns {
            if ann.node_id == self.node_id {
                continue;
            }
            self.ingest_announcement(ann)?;
            count += 1;
        }
        Ok(count)
    }

    /// Register local node as seeder after HTTP or file import.
    pub async fn register_local_seed(
        &self,
        room_id: &str,
        content_hash: &str,
        title: &str,
        kind: CdnContentKind,
        size_bytes: u64,
        source_url: Option<String>,
    ) -> Result<CdnAnnouncement, String> {
        let ticket = self.make_ticket(content_hash).await.ok();
        let ann = CdnAnnouncement {
            room_id: room_id.to_string(),
            content_hash: content_hash.to_string(),
            node_id: self.node_id.clone(),
            title: title.to_string(),
            kind,
            size_bytes,
            mime_type: None,
            source_url,
            ticket,
            timestamp: Utc::now().timestamp_millis() as u64,
        };
        self.announce_and_publish(ann).await
    }

    pub fn ingest_announcement(&self, ann: CdnAnnouncement) -> Result<(), String> {
        let asset = CdnAsset {
            content_hash: ann.content_hash.clone(),
            title: ann.title.clone(),
            kind: ann.kind.clone(),
            size_bytes: ann.size_bytes,
            mime_type: ann.mime_type.clone(),
            source_url: ann.source_url.clone(),
            room_id: ann.room_id.clone(),
            announcer_node_id: ann.node_id.clone(),
            announced_at: ann.timestamp,
        };
        {
            let mut rooms = self.assets_by_room.lock().map_err(|e| e.to_string())?;
            rooms
                .entry(ann.room_id.clone())
                .or_default()
                .insert(ann.content_hash.clone(), asset);
        }
        let peer = CdnPeerSource {
            node_id: ann.node_id.clone(),
            content_hash: ann.content_hash.clone(),
            ticket: ann.ticket.clone(),
            last_seen: ann.timestamp,
            rtt_ms: None,
        };
        {
            let mut peers = self.peers_by_hash.lock().map_err(|e| e.to_string())?;
            let list = peers.entry(ann.content_hash.clone()).or_default();
            if let Some(existing) = list.iter_mut().find(|p| p.node_id == peer.node_id) {
                *existing = peer;
            } else {
                list.push(peer);
            }
        }
        Ok(())
    }

    pub fn sync_gossip(&self) -> Result<usize, String> {
        let rooms: Vec<String> = self
            .joined_rooms
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .cloned()
            .collect();
        let mut gossip = self.gossip_mut()?;
        let mut count = 0usize;
        for room_id in rooms {
            for ann in gossip.drain_room(&room_id) {
                self.ingest_announcement(ann)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Build room feed after syncing in-process and external gossip queues.
    pub async fn room_feed_async(&self, room_id: &str) -> Result<CdnRoomFeed, String> {
        let _ = self.pull_external_gossip(room_id).await;
        self.room_feed(room_id)
    }

    pub fn room_feed(&self, room_id: &str) -> Result<CdnRoomFeed, String> {
        let _ = self.sync_gossip();
        let assets = {
            let rooms = self.assets_by_room.lock().map_err(|e| e.to_string())?;
            rooms
                .get(room_id)
                .map(|m| m.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };
        let mut peer_map = HashMap::new();
        let peers = self.peers_by_hash.lock().map_err(|e| e.to_string())?;
        for asset in &assets {
            if let Some(list) = peers.get(&asset.content_hash) {
                peer_map.insert(asset.content_hash.clone(), list.clone());
            }
        }
        Ok(CdnRoomFeed {
            room_id: room_id.to_string(),
            assets,
            peer_map,
        })
    }

    pub fn list_peers(&self, content_hash: &str) -> Result<Vec<CdnPeerSource>, String> {
        let peers = self.peers_by_hash.lock().map_err(|e| e.to_string())?;
        Ok(peers.get(content_hash).cloned().unwrap_or_default())
    }

    pub fn get_asset(&self, room_id: &str, content_hash: &str) -> Option<CdnAsset> {
        let rooms = self.assets_by_room.lock().ok()?;
        rooms.get(room_id)?.get(content_hash).cloned()
    }

    pub fn upsert_job(&self, job: CdnDownloadJob) -> Result<(), String> {
        let mut jobs = self.jobs.lock().map_err(|e| e.to_string())?;
        jobs.insert(job.job_id.clone(), job);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_job(&self, job_id: &str) -> Option<CdnDownloadJob> {
        let jobs = self.jobs.lock().ok()?;
        jobs.get(job_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_p2p_cdn_state_creation() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Verify state initialization
        assert!(!state.node_id.is_empty());
        assert!(state.node_id.starts_with("exodus-"));
        assert_eq!(state.node_id.len(), 19); // "exodus-" + 12 chars
    }

    #[test]
    fn test_join_and_leave_room() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Test joining a room
        state.join_room("test-room").expect("Failed to join room");
        let rooms = state.joined_rooms_list().expect("Failed to get joined rooms");
        assert!(rooms.contains(&"test-room".to_string()));
        
        // Test leaving a room
        state.leave_room("test-room").expect("Failed to leave room");
        let rooms = state.joined_rooms_list().expect("Failed to get joined rooms");
        assert!(!rooms.contains(&"test-room".to_string()));
    }

    #[test]
    fn test_mesh_endpoint_initially_none() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Initially, mesh endpoint should be None
        assert!(state.mesh_endpoint().is_none());
    }

    #[test]
    fn test_store_access() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Test store access - just verify we can get a reference
        let _store = state.store();
        let _store_arc = state.store_arc();
        // If we got here without panicking, access worked
    }

    #[test]
    fn test_gossip_mut_access() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Test gossip mutable access
        let gossip = state.gossip_mut().expect("Failed to get gossip");
        // GossipBridge should be accessible
        assert!(true); // If we got here, access worked
        drop(gossip);
    }

    #[test]
    fn test_job_management() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Create a test job
        let job = CdnDownloadJob {
            job_id: "test-job-1".to_string(),
            content_hash: "hash-123".to_string(),
            title: "Test Download".to_string(),
            status: "pending".to_string(),
            progress_percent: 0.0,
            bytes_done: 0,
            bytes_total: 1000,
            source: "test-source".to_string(),
            peer_count: 0,
            local_path: None,
            error: None,
        };
        
        // Upsert job
        state.upsert_job(job.clone()).expect("Failed to upsert job");
        
        // Get job
        let retrieved = state.get_job("test-job-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected job to exist").job_id, "test-job-1");
    }

    #[test]
    fn test_multiple_rooms() {
        let temp_dir = std::env::temp_dir().join(format!("test_cdn_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let state = P2pCdnState::new(&temp_dir).expect("Failed to create state");
        
        // Join multiple rooms
        state.join_room("room-1").expect("Failed to join room-1");
        state.join_room("room-2").expect("Failed to join room-2");
        state.join_room("room-3").expect("Failed to join room-3");
        
        let rooms = state.joined_rooms_list().expect("Failed to get joined rooms");
        assert_eq!(rooms.len(), 3);
        assert!(rooms.contains(&"room-1".to_string()));
        assert!(rooms.contains(&"room-2".to_string()));
        assert!(rooms.contains(&"room-3".to_string()));
    }
}
