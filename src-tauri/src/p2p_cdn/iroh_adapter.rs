//! Exodus P2P CDN — peer transport (iroh-blobs when `iroh-cdn` feature; else Exodus mesh HTTP).

use std::path::Path;

use super::mesh_fetch::fetch_from_mesh_peers;
use super::store::CdnBlobStore;
use super::types::CdnPeerSource;

/// Peer blob fetch (iroh-blobs or Exodus mesh HTTP).
pub struct PeerBlobTransport {
    #[allow(dead_code)]
    pub local_node_id: String,
}

impl PeerBlobTransport {
    pub fn new(local_node_id: String) -> Self {
        Self { local_node_id }
    }

    /// Download from peer list in parallel (mesh HTTP; iroh-blobs when feature enabled).
    pub async fn download_from_peers(
        &self,
        content_hash: &str,
        peers: &[CdnPeerSource],
        downloads_dir: &Path,
        title: &str,
        store: &CdnBlobStore,
    ) -> Result<u64, String> {
        if store.has_complete(content_hash) {
            let dest = downloads_dir.join(safe_name(title, content_hash));
            return store.export_to_file(content_hash, &dest);
        }

        #[cfg(feature = "iroh-cdn")]
        {
            if let Ok(n) = self.download_via_iroh(content_hash, peers, downloads_dir, title).await
            {
                return Ok(n);
            }
        }

        let dest = downloads_dir.join(safe_name(title, content_hash));
        fetch_from_mesh_peers(content_hash, peers, &dest, store).await
    }

    #[cfg(feature = "iroh-cdn")]
    async fn download_via_iroh(
        &self,
        content_hash: &str,
        peers: &[CdnPeerSource],
        downloads_dir: &Path,
        title: &str,
    ) -> Result<u64, String> {
        let _ = (self, content_hash, peers, downloads_dir, title);
        Err("iroh-cdn: enable iroh crates when netwatch builds on this OS".into())
    }
}

fn safe_name(title: &str, hash: &str) -> String {
    let base: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(60)
        .collect();
    let short = if hash.len() > 8 { &hash[..8] } else { hash };
    format!("{base}-{short}.bin")
}
