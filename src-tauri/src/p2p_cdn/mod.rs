//! Exodus Browser — AI-driven P2P CDN (iroh-blobs compatible content addressing).
//!
//! Orchestrates room/lobby gossip announcements, BLAKE3 content hashes, peer swarm
//! indexing, and parallel fetch (peers first, HTTP fallback). Real iroh-blobs transport
//! plugs in via [`IrohBlobAdapter`] when the `iroh-cdn` feature is enabled.

mod commands;
mod download;
pub mod external_gossip;
mod gossip_bridge;
mod iroh_adapter;
mod mesh_fetch;
mod mesh_server;
mod mesh_ticket;
mod store;
mod swarm;
mod types;

#[cfg(test)]
mod integration_tests;

pub use commands::{
    p2p_cdn_announce_asset, p2p_cdn_announce_group_hot, p2p_cdn_announce_url_hot,
    p2p_cdn_download, p2p_cdn_get_asset, p2p_cdn_group_send_message, p2p_cdn_hash_file,
    p2p_cdn_join_room, p2p_cdn_leave_room, p2p_cdn_list_peers, p2p_cdn_node_info,
    p2p_cdn_register_local_seed, p2p_cdn_room_feed, p2p_cdn_start_mesh, p2p_cdn_sync_gossip,
    p2p_cdn_url_status,
};
pub use swarm::P2pCdnState;
pub use mesh_fetch::fetch_from_mesh_peers;
pub use mesh_ticket::{tickets_from_peers, ExodusCdnTicket};
pub use types::{CdnContentKind, CdnPeerSource};
pub use download::start_cdn_download;
