//! Exodus P2P CDN — cross-module integration tests.

use std::io::Write;
use std::sync::Arc;

use super::mesh_fetch::fetch_from_mesh_peers;
use super::mesh_ticket::ExodusCdnTicket;
use super::store::CdnBlobStore;
use super::swarm::P2pCdnState;
use super::types::{CdnAnnouncement, CdnContentKind, CdnPeerSource};

#[tokio::test]
async fn two_hub_mesh_seed_and_fetch() {
    let dir = std::env::temp_dir().join(format!("cdn_integ_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("Failed to create temp dir");

    let seeder = P2pCdnState::new(&dir.join("seeder")).expect("seeder");
    let leecher = P2pCdnState::new(&dir.join("leecher")).expect("leecher");

    let payload = b"exodus-full-pipeline-test";
    let (hash, size) = seeder.store().put_bytes(payload).expect("put");
    assert_eq!(size, payload.len() as u64);

    seeder
        .register_local_seed(
            "lobby",
            &hash,
            "Pipeline Test",
            CdnContentKind::Article,
            size,
            None,
        )
        .await
        .expect("seed");

    let (host, port) = seeder.mesh_endpoint().expect("mesh");
    let ticket = ExodusCdnTicket::encode(&seeder.node_id, &host, port, &hash);
    let peers = vec![CdnPeerSource {
        node_id: seeder.node_id.clone(),
        content_hash: hash.clone(),
        ticket: Some(ticket),
        last_seen: 1,
        rtt_ms: None,
    }];

    let dest = dir.join("leeched.bin");
    let n = fetch_from_mesh_peers(&hash, &peers, &dest, leecher.store())
        .await
        .expect("fetch");
    assert_eq!(n, payload.len() as u64);
    assert_eq!(std::fs::read(&dest).expect("Failed to read dest file").as_slice(), payload);
}

#[tokio::test]
async fn room_feed_lists_announced_assets() {
    let dir = std::env::temp_dir().join(format!("cdn_feed_{}", uuid::Uuid::new_v4()));
    let hub = P2pCdnState::new(&dir).expect("hub");
    hub.join_room("group-1").expect("Failed to join room");
    let (hash, _) = hub
        .store()
        .put_bytes(b"feed-test")
        .expect("put");
    hub.register_local_seed(
        "group-1",
        &hash,
        "Hot Model",
        CdnContentKind::AiModel,
        9,
        Some("https://example.com/model".into()),
    )
    .await
    .expect("announce");

    let feed = hub.room_feed("group-1").expect("feed");
    assert_eq!(feed.room_id, "group-1");
    assert_eq!(feed.assets.len(), 1);
    assert_eq!(feed.assets[0].title, "Hot Model");
    assert!(!feed.peer_map.get(&hash).expect("Failed to get peer from peer_map").is_empty());
}

#[tokio::test]
async fn announce_url_hot_via_swarm() {
    let dir = std::env::temp_dir().join(format!("cdn_url_{}", uuid::Uuid::new_v4()));
    let hub = P2pCdnState::new(&dir).expect("hub");
    let url = "https://example.com/hot.bin";
    let hash = blake3::hash(url.as_bytes()).to_hex().to_string();
    hub.join_room("lobby").expect("Failed to join room");
    let ann = CdnAnnouncement {
        room_id: "lobby".into(),
        content_hash: hash.clone(),
        node_id: hub.node_id.clone(),
        title: "Hot URL".into(),
        kind: CdnContentKind::AiModel,
        size_bytes: 9_000_000,
        mime_type: None,
        source_url: Some(url.into()),
        ticket: None,
        timestamp: 1,
    };
    hub.announce_and_publish(ann).await.expect("ann");
    let feed = hub.room_feed("lobby").expect("feed");
    assert!(feed.assets.iter().any(|a| a.content_hash == hash));
}

#[tokio::test]
async fn url_status_discovery_hash_for_announced_url() {
    let dir = std::env::temp_dir().join(format!("cdn_url_status_{}", uuid::Uuid::new_v4()));
    let hub = P2pCdnState::new(&dir).expect("hub");
    let url = "https://example.com/status-page";
    let discovery_hash = blake3::hash(url.as_bytes()).to_hex().to_string();
    hub.join_room("lobby").expect("Failed to join room");
    let ann = CdnAnnouncement {
        room_id: "lobby".into(),
        content_hash: discovery_hash.clone(),
        node_id: hub.node_id.clone(),
        title: "Status Page".into(),
        kind: CdnContentKind::Article,
        size_bytes: 0,
        mime_type: None,
        source_url: Some(url.into()),
        ticket: None,
        timestamp: 1,
    };
    hub.announce_and_publish(ann).await.expect("ann");
    let asset = hub.get_asset("lobby", &discovery_hash);
    assert!(asset.is_some());
    assert!(!hub.store().has_complete(&discovery_hash));
}

#[test]
fn hash_file_matches_import() {
    let dir = std::env::temp_dir().join(format!("cdn_hash_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
    let path = dir.join("sample.bin");
    std::fs::write(&path, b"hash-file-test").expect("Failed to write file");
    let (hash, size) = CdnBlobStore::hash_file(&path).expect("Failed to hash file");
    assert_eq!(size, 14);
    let store = CdnBlobStore::new(&dir.join("blobs")).expect("Failed to create store");
    let (imported, _) = store.import_file(&path).expect("Failed to import file");
    assert_eq!(hash, imported);
}

#[test]
fn large_file_chunked_store_and_mesh_meta() {
    let dir = std::env::temp_dir().join(format!("cdn_chunk_{}", uuid::Uuid::new_v4()));
    let store = Arc::new(CdnBlobStore::new(&dir).expect("Failed to create store"));
    let path = dir.join("large.bin");
    {
        let mut f = std::fs::File::create(&path).expect("Failed to create file");
        let data = vec![0xABu8; super::store::CHUNK_SIZE * 3 + 100];
        f.write_all(&data).expect("Failed to write data");
    }
    let (hash, size) = store.import_file(&path).expect("Failed to import file");
    assert!(size > super::store::CHUNK_SIZE as u64);
    let (meta_size, chunks) = store.meta(&hash).expect("Failed to get meta");
    assert_eq!(meta_size, size);
    assert!(chunks >= 4);
}
