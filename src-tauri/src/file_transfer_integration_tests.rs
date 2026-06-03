//! Integration tests: ExodusWorkSpace + file transfer + P2P CDN store.

#[cfg(test)]
mod tests {
    use crate::exodus_workspace::{ExodusWorkspace, WORKSPACE_ROOM_ID};
    use crate::microservice::{FileTransferService, FileTransferServiceConfig};
    use crate::p2p_cdn::P2pCdnState;
    use std::fs;
    use std::io::Write;
    use std::sync::Arc;

    #[tokio::test]
    async fn workspace_seed_and_transfer_roundtrip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cdn = Arc::new(P2pCdnState::new(tmp.path()).expect("cdn"));
        cdn.join_room(WORKSPACE_ROOM_ID).expect("join");
        let (host, port) = cdn.ensure_mesh().await.expect("mesh");
        assert!(port > 0);
        assert!(!host.is_empty());

        let ws = ExodusWorkspace::new(tmp.path(), cdn.node_id.clone()).expect("ws");
        let src = tmp.path().join("share.me");
        let mut f = fs::File::create(&src).expect("create");
        writeln!(f, "exodus integration").expect("write");

        let (hash, size) = cdn.store().import_file(&src).expect("import");
        assert!(size > 0);
        let ann = cdn
            .register_local_seed(
                WORKSPACE_ROOM_ID,
                &hash,
                "share.me",
                crate::p2p_cdn::CdnContentKind::GenericFile,
                size,
                None,
            )
            .await
            .expect("seed");
        assert!(ann.ticket.is_some());

        let entry = ws.publish_file(&src, Some(hash.clone())).expect("publish");
        assert!(entry.relative_path.contains("shared"));

        let cfg = FileTransferServiceConfig::under_app_data(tmp.path(), &cdn.node_id);
        let ft = FileTransferService::new(cfg).expect("ft");
        let meta = ft
            .initiate_transfer(&src, None, Some(hash.clone()), ann.ticket, Some(entry.relative_path))
            .expect("initiate");
        assert_eq!(meta.room_id, WORKSPACE_ROOM_ID);
        assert!(cdn.store().has_complete(&hash));

        let inbox = ws.inbox_dir().join("received.me");
        cdn.store().export_to_file(&hash, &inbox).expect("export");
        let text = fs::read_to_string(&inbox).expect("read");
        assert!(text.contains("exodus integration"));
    }
}
