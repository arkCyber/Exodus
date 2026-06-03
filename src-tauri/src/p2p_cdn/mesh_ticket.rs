//! Exodus P2P CDN — peer tickets (iroh BlobTicket–compatible addressing).

use super::types::CdnPeerSource;

/// Parsed Exodus CDN peer ticket: `exodus-cdn://{node_id}@{host}:{port}/{hash}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExodusCdnTicket {
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub content_hash: String,
}

impl ExodusCdnTicket {
    /// Build ticket string for gossip announcements.
    pub fn encode(node_id: &str, host: &str, port: u16, content_hash: &str) -> String {
        format!("exodus-cdn://{node_id}@{host}:{port}/{content_hash}")
    }

    /// Parse ticket from gossip or QR share string.
    pub fn parse(raw: &str) -> Option<Self> {
        let rest = raw.strip_prefix("exodus-cdn://")?;
        let (authority, hash) = rest.split_once('/')?;
        let (node_id, addr) = authority.split_once('@')?;
        let (host, port_str) = addr.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;
        if node_id.is_empty() || host.is_empty() || hash.is_empty() {
            return None;
        }
        Some(Self {
            node_id: node_id.to_string(),
            host: host.to_string(),
            port,
            content_hash: hash.to_string(),
        })
    }

    /// Base URL for HTTP mesh fetches.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Resolve peer tickets from swarm entries.
pub fn tickets_from_peers(peers: &[CdnPeerSource]) -> Vec<ExodusCdnTicket> {
    peers
        .iter()
        .filter_map(|p| p.ticket.as_ref().and_then(|t| ExodusCdnTicket::parse(t)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_roundtrip() {
        let raw = ExodusCdnTicket::encode("exodus-abc", "192.168.1.10", 7878, "deadbeef");
        let t = ExodusCdnTicket::parse(&raw).expect("parse");
        assert_eq!(t.node_id, "exodus-abc");
        assert_eq!(t.port, 7878);
        assert_eq!(t.content_hash, "deadbeef");
    }
}
