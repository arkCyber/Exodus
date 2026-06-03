//! Exodus P2P CDN — disk-backed BLAKE3 content store (iroh-blobs compatible hashing).

use std::fs::{self, File};
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::io::Read as _;
use std::path::{Path, PathBuf};

use blake3::Hasher;

/// Chunk size aligned with iroh-blobs group granularity (16 KiB).
pub const CHUNK_SIZE: usize = 16 * 1024;

/// On-disk layout: `{data_dir}/{hash_hex}/meta.json` + `chunks/{index}`.
pub struct CdnBlobStore {
    data_dir: PathBuf,
}

impl CdnBlobStore {
    /// Open store under app data.
    pub fn new(data_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            data_dir: data_dir.to_path_buf(),
        })
    }

    /// Hash file bytes (streaming) for content addressing.
    pub fn hash_file(path: &Path) -> Result<(String, u64), String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut hasher = Hasher::new();
        let mut buf = [0u8; CHUNK_SIZE];
        let mut total = 0u64;
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            total += n as u64;
        }
        Ok((hasher.finalize().to_hex().to_string(), total))
    }

    /// Import a local file into the store (chunked); returns content hash.
    pub fn import_file(&self, source: &Path) -> Result<(String, u64), String> {
        let (hash, size) = Self::hash_file(source)?;
        let dest_dir = self.blob_dir(&hash);
        if dest_dir.join("complete").exists() {
            return Ok((hash, size));
        }
        fs::create_dir_all(dest_dir.join("chunks")).map_err(|e| e.to_string())?;

        let mut file = File::open(source).map_err(|e| e.to_string())?;
        let mut index = 0u32;
        let mut buf = vec![0u8; CHUNK_SIZE];
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            let chunk_path = dest_dir.join("chunks").join(format!("{index:06}"));
            let mut out = File::create(&chunk_path).map_err(|e| e.to_string())?;
            out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            index += 1;
        }
        fs::write(dest_dir.join("complete"), b"1").map_err(|e| e.to_string())?;
        let meta = serde_json::json!({
            "hash": hash,
            "sizeBytes": size,
            "chunkCount": index,
        });
        fs::write(
            dest_dir.join("meta.json"),
            serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok((hash, size))
    }

    /// Whether we have a complete local copy.
    pub fn has_complete(&self, hash: &str) -> bool {
        self.blob_dir(hash).join("complete").exists()
    }

    /// Read blob metadata (`size_bytes`, `chunk_count`).
    pub fn meta(&self, hash: &str) -> Result<(u64, u32), String> {
        let dir = self.blob_dir(hash);
        if !dir.join("complete").exists() {
            return Err(format!("Blob not complete locally: {hash}"));
        }
        let meta_raw = fs::read_to_string(dir.join("meta.json")).map_err(|e| e.to_string())?;
        let meta: serde_json::Value =
            serde_json::from_str(&meta_raw).map_err(|e| e.to_string())?;
        let size = meta
            .get("sizeBytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let chunks = meta
            .get("chunkCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        Ok((size, chunks))
    }

    /// Read one stored chunk by index.
    pub fn read_chunk(&self, hash: &str, index: u32) -> Result<Vec<u8>, String> {
        let path = self
            .blob_dir(hash)
            .join("chunks")
            .join(format!("{index:06}"));
        fs::read(path).map_err(|e| e.to_string())
    }

    /// Export blob to a destination file path.
    pub fn export_to_file(&self, hash: &str, dest: &Path) -> Result<u64, String> {
        let dir = self.blob_dir(hash);
        if !dir.join("complete").exists() {
            return Err(format!("Blob not complete locally: {hash}"));
        }
        let meta_raw = fs::read_to_string(dir.join("meta.json")).map_err(|e| e.to_string())?;
        let meta: serde_json::Value =
            serde_json::from_str(&meta_raw).map_err(|e| e.to_string())?;
        let chunk_count = meta
            .get("chunkCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = File::create(dest).map_err(|e| e.to_string())?;
        let mut total = 0u64;
        for i in 0..chunk_count {
            let chunk_path = dir.join("chunks").join(format!("{i:06}"));
            let mut chunk = Vec::new();
            File::open(&chunk_path)
                .and_then(|mut f| f.read_to_end(&mut chunk))
                .map_err(|e| e.to_string())?;
            out.write_all(&chunk).map_err(|e| e.to_string())?;
            total += chunk.len() as u64;
        }
        Ok(total)
    }

    /// Store raw bytes (small assets) as single-chunk blob.
    pub fn put_bytes(&self, data: &[u8]) -> Result<(String, u64), String> {
        let hash = blake3::hash(data).to_hex().to_string();
        let dest_dir = self.blob_dir(&hash);
        if dest_dir.join("complete").exists() {
            return Ok((hash, data.len() as u64));
        }
        fs::create_dir_all(dest_dir.join("chunks")).map_err(|e| e.to_string())?;
        let chunk_path = dest_dir.join("chunks").join("000000");
        fs::write(&chunk_path, data).map_err(|e| e.to_string())?;
        fs::write(dest_dir.join("complete"), b"1").map_err(|e| e.to_string())?;
        let meta = serde_json::json!({
            "hash": hash,
            "sizeBytes": data.len(),
            "chunkCount": 1,
        });
        fs::write(
            dest_dir.join("meta.json"),
            serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok((hash, data.len() as u64))
    }

    fn blob_dir(&self, hash: &str) -> PathBuf {
        self.data_dir.join(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn meta_and_read_chunk_for_multichunk_file() {
        let dir = std::env::temp_dir().join(format!("exodus_cdn_chunk_{}", uuid::Uuid::new_v4()));
        let store = CdnBlobStore::new(&dir).expect("store");
        let src = dir.join("chunked.bin");
        let data = vec![7u8; CHUNK_SIZE + 512];
        std::fs::write(&src, &data).expect("Failed to write file");
        let (hash, size) = store.import_file(&src).expect("import");
        let (meta_size, count) = store.meta(&hash).expect("meta");
        assert_eq!(meta_size, size);
        assert!(count >= 2);
        let c0 = store.read_chunk(&hash, 0).expect("c0");
        assert!(!c0.is_empty());
    }

    #[test]
    fn import_and_export_roundtrip() {
        let dir = std::env::temp_dir().join(format!("exodus_cdn_store_{}", uuid::Uuid::new_v4()));
        let store = CdnBlobStore::new(&dir).expect("store");
        let src = dir.join("sample.bin");
        {
            let mut f = File::create(&src).expect("Failed to create file");
            f.write_all(b"exodus-p2p-cdn-test-payload").expect("Failed to write data");
        }
        let (hash, _) = store.import_file(&src).expect("import");
        assert!(store.has_complete(&hash));
        let out = dir.join("out.bin");
        store.export_to_file(&hash, &out).expect("export");
        let data = fs::read(&out).expect("Failed to read file");
        assert_eq!(data, b"exodus-p2p-cdn-test-payload");
    }
}
