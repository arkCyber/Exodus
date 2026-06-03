//! ExodusWorkSpace — per-node shared folder for P2P file exchange.
//!
//! Layout under app data:
//! `{app_data}/ExodusWorkSpace/{shared,inbox,outbox}/` plus `workspace.json` manifest.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Gossip / CDN room id for workspace file announcements.
pub const WORKSPACE_ROOM_ID: &str = "ExodusWorkSpace";

/// Subdirectories inside the workspace root.
pub const DIR_SHARED: &str = "shared";
pub const DIR_INBOX: &str = "inbox";
pub const DIR_OUTBOX: &str = "outbox";

/// One file entry in the workspace manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileEntry {
    pub name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub content_hash: Option<String>,
    pub added_at: u64,
}

/// Workspace manifest persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifest {
    pub root: String,
    pub node_id: String,
    pub files: Vec<WorkspaceFileEntry>,
    pub updated_at: u64,
}

/// Managed ExodusWorkSpace paths and manifest.
pub struct ExodusWorkspace {
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: Mutex<WorkspaceManifest>,
}

impl ExodusWorkspace {
    /// Open or create workspace under `app_data_dir/ExodusWorkSpace`.
    pub fn new(app_data_dir: &Path, node_id: impl Into<String>) -> Result<Self, String> {
        let root = app_data_dir.join("ExodusWorkSpace");
        for sub in [DIR_SHARED, DIR_INBOX, DIR_OUTBOX] {
            fs::create_dir_all(root.join(sub)).map_err(|e| format!("Create {sub}: {e}"))?;
        }
        let manifest_path = root.join("workspace.json");
        let manifest = if manifest_path.exists() {
            let raw = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_else(|_| WorkspaceManifest {
                root: root.to_string_lossy().to_string(),
                node_id: node_id.into(),
                files: Vec::new(),
                updated_at: current_timestamp(),
            })
        } else {
            WorkspaceManifest {
                root: root.to_string_lossy().to_string(),
                node_id: node_id.into(),
                files: Vec::new(),
                updated_at: current_timestamp(),
            }
        };
        let ws = Self {
            root,
            manifest_path,
            manifest: Mutex::new(manifest),
        };
        ws.save_manifest()?;
        Ok(ws)
    }

    /// Absolute path to workspace root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to the shared (publish) folder.
    pub fn shared_dir(&self) -> PathBuf {
        self.root.join(DIR_SHARED)
    }

    /// Path to inbox (received files).
    pub fn inbox_dir(&self) -> PathBuf {
        self.root.join(DIR_INBOX)
    }

    /// Path to outbox (pending outbound copies).
    pub fn outbox_dir(&self) -> PathBuf {
        self.root.join(DIR_OUTBOX)
    }

    /// Copy a local file into `shared/` and register in the manifest.
    pub fn publish_file(
        &self,
        source: &Path,
        content_hash: Option<String>,
    ) -> Result<WorkspaceFileEntry, String> {
        if !source.is_file() {
            return Err(format!("Not a file: {}", source.display()));
        }
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?
            .to_string();
        let dest = self.resolve_unique_path(&self.shared_dir(), &file_name);
        fs::copy(source, &dest).map_err(|e| format!("Copy to workspace: {e}"))?;
        let meta = fs::metadata(&dest).map_err(|e| e.to_string())?;
        let rel = format!("{DIR_SHARED}/{}", dest.file_name().and_then(|n| n.to_str()).unwrap_or(&file_name));
        let entry = WorkspaceFileEntry {
            name: dest
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_name)
                .to_string(),
            relative_path: rel,
            size_bytes: meta.len(),
            content_hash,
            added_at: current_timestamp(),
        };
        {
            let mut m = self.manifest.lock().map_err(|e| e.to_string())?;
            m.files.retain(|f| f.relative_path != entry.relative_path);
            m.files.push(entry.clone());
            m.updated_at = current_timestamp();
        }
        self.save_manifest()?;
        Ok(entry)
    }

    /// List manifest entries (shared files).
    pub fn list_files(&self) -> Result<Vec<WorkspaceFileEntry>, String> {
        let m = self.manifest.lock().map_err(|e| e.to_string())?;
        Ok(m.files.clone())
    }

    /// Full manifest snapshot for UI.
    pub fn manifest_snapshot(&self) -> Result<WorkspaceManifest, String> {
        let m = self.manifest.lock().map_err(|e| e.to_string())?;
        Ok(m.clone())
    }

    /// Resolve a unique destination path when `name` already exists.
    fn resolve_unique_path(&self, dir: &Path, name: &str) -> PathBuf {
        let mut dest = dir.join(name);
        if !dest.exists() {
            return dest;
        }
        let (stem, ext) = split_name_ext(name);
        let mut n = 1u32;
        loop {
            let candidate = if ext.is_empty() {
                dir.join(format!("{stem}_{n}"))
            } else {
                dir.join(format!("{stem}_{n}.{ext}"))
            };
            if !candidate.exists() {
                return candidate;
            }
            n += 1;
        }
    }

    fn save_manifest(&self) -> Result<(), String> {
        let m = self.manifest.lock().map_err(|e| e.to_string())?;
        let raw = serde_json::to_string_pretty(&*m).map_err(|e| e.to_string())?;
        fs::write(&self.manifest_path, raw).map_err(|e| e.to_string())
    }
}

fn split_name_ext(name: &str) -> (String, String) {
    match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), e.to_string()),
        _ => (name.to_string(), String::new()),
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn creates_workspace_layout() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ExodusWorkspace::new(tmp.path(), "node-test").expect("workspace");
        assert!(ws.shared_dir().is_dir());
        assert!(ws.inbox_dir().is_dir());
        assert!(ws.manifest_snapshot().expect("Failed to get manifest snapshot").node_id == "node-test");
    }

    #[test]
    fn publish_file_updates_manifest() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ExodusWorkspace::new(tmp.path(), "n1").expect("workspace");
        let src = tmp.path().join("hello.txt");
        let mut f = fs::File::create(&src).expect("create");
        writeln!(f, "exodus workspace test").expect("write");
        let entry = ws
            .publish_file(&src, Some("abc123".into()))
            .expect("publish");
        assert!(entry.relative_path.starts_with("shared/"));
        let list = ws.list_files().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].content_hash.as_deref(), Some("abc123"));
    }
}
