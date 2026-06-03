//! Exodus Browser — CRX / ZIP Web Extension package installation and CRX3 signature check.

use std::fs::File;
use std::io::Cursor;
use std::path::Path;

use ring::signature::{self, UnparsedPublicKey};
use zip::ZipArchive;

use super::error::PluginError;

/// Context string signed in CRX3 packages (Chromium).
const CRX3_SIGNATURE_CONTEXT: &[u8] = b"CRX3 Signed File";

/// Extract CRX (v2/v3) or plain ZIP into `dest` directory.
pub fn extract_extension_package(
    source: &Path,
    dest: &Path,
    require_crx_signature: bool,
) -> Result<(), PluginError> {
    let bytes = std::fs::read(source).map_err(PluginError::Io)?;
    let zip_bytes = if require_crx_signature {
        crx_verify_and_zip_slice(&bytes)?
    } else {
        crx_zip_slice(&bytes)?
    };
    extract_zip_bytes(zip_bytes, dest)
}

/// Verify CRX3 RSA signature when present; return ZIP slice.
fn crx_verify_and_zip_slice(data: &[u8]) -> Result<&[u8], PluginError> {
    if data.len() >= 4 && &data[0..4] == b"Cr24" {
        if data.len() < 12 {
            return Err(PluginError::InvalidManifest("CRX header too short".into()));
        }
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let header_size =
            u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let header_start: usize = 12;
        let header_end = header_start
            .checked_add(header_size)
            .ok_or_else(|| PluginError::InvalidManifest("CRX header size overflow".into()))?;
        if header_end > data.len() {
            return Err(PluginError::InvalidManifest("CRX header truncated".into()));
        }
        let header = &data[header_start..header_end];
        let zip_bytes = &data[header_end..];

        if version == 3 {
            verify_crx3_signature(header, zip_bytes)?;
            tracing::info!(
                "[{}] CRX3 signature verified",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
            );
            return Ok(zip_bytes);
        }
        if version == 2 {
            return Err(PluginError::InvalidManifest(
                "CRX version 2 is not supported when signature verification is required".into(),
            ));
        }
        return Err(PluginError::InvalidManifest(format!(
            "Unsupported CRX version: {version}"
        )));
    }
    crx_zip_slice(data)
}

/// Locate ZIP payload inside a CRX container (no signature check).
fn crx_zip_slice(data: &[u8]) -> Result<&[u8], PluginError> {
    if data.len() >= 4 && &data[0..4] == b"Cr24" {
        if data.len() < 12 {
            return Err(PluginError::InvalidManifest("CRX header too short".into()));
        }
        let header_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let zip_start = 12 + header_size;
        if zip_start >= data.len() {
            return Err(PluginError::InvalidManifest("CRX zip offset invalid".into()));
        }
        return Ok(&data[zip_start..]);
    }
    if data.len() >= 2 && data[0] == b'P' && data[1] == b'K' {
        return Ok(data);
    }
    if let Some(pos) = data.windows(2).position(|w| w == b"PK") {
        return Ok(&data[pos..]);
    }
    Err(PluginError::InvalidManifest(
        "Not a CRX or ZIP extension package".into(),
    ))
}

/// Verify CRX3 header RSA-SHA256 signature (Chromium CRX3 layout).
fn verify_crx3_signature(header: &[u8], zip_bytes: &[u8]) -> Result<(), PluginError> {
    let (public_key, signature) = parse_crx3_rsa_proof(header).ok_or_else(|| {
        PluginError::InvalidManifest("CRX3 header missing sha256_with_rsa proof".into())
    })?;
    if public_key.is_empty() || signature.is_empty() {
        return Err(PluginError::InvalidManifest(
            "CRX3 public key or signature empty".into(),
        ));
    }

    let mut signed = Vec::new();
    signed.extend_from_slice(CRX3_SIGNATURE_CONTEXT);
    signed.extend_from_slice(&(header.len() as u32).to_le_bytes());
    signed.extend_from_slice(header);
    signed.extend_from_slice(&(zip_bytes.len() as u32).to_le_bytes());
    signed.extend_from_slice(zip_bytes);

    let unparsed = UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, &public_key);
    unparsed
        .verify(&signed, &signature)
        .map_err(|_| PluginError::InvalidManifest("CRX3 signature verification failed".into()))
}

/// Parse `CrxFileHeader.sha256_with_rsa` (protobuf field 2).
fn parse_crx3_rsa_proof(header: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let proof = read_proto_bytes_field(header, 2)?;
    let public_key = read_proto_bytes_field(proof, 1)?;
    let signature = read_proto_bytes_field(proof, 2)?;
    Some((public_key.to_vec(), signature.to_vec()))
}

/// Read length-delimited protobuf field by number.
fn read_proto_bytes_field(data: &[u8], field_num: u64) -> Option<&[u8]> {
    let mut pos = 0usize;
    while pos < data.len() {
        let (tag, next) = read_varint(data, pos)?;
        pos = next;
        let wire = tag & 0x7;
        let num = tag >> 3;
        if wire == 2 {
            let (len, next) = read_varint(data, pos)?;
            pos = next;
            let len = len as usize;
            if pos + len > data.len() {
                return None;
            }
            let value = &data[pos..pos + len];
            pos += len;
            if num == field_num {
                return Some(value);
            }
        } else if wire == 0 {
            let (_, next) = read_varint(data, pos)?;
            pos = next;
        } else if wire == 1 {
            pos += 8;
        } else if wire == 5 {
            pos += 4;
        } else {
            return None;
        }
    }
    None
}

fn read_varint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;
    while pos < data.len() {
        let byte = data[pos];
        pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((result, pos));
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
    None
}

fn extract_zip_bytes(zip_bytes: &[u8], dest: &Path) -> Result<(), PluginError> {
    std::fs::create_dir_all(dest).map_err(PluginError::Io)?;
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| PluginError::Parse(format!("ZIP open: {e}")))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| PluginError::Parse(format!("ZIP entry: {e}")))?;
        let name = file.name().to_string();
        if name.contains("..") {
            continue;
        }
        let out_path = dest.join(&name);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(PluginError::Io)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(PluginError::Io)?;
        }
        let mut out = File::create(&out_path).map_err(PluginError::Io)?;
        std::io::copy(&mut file, &mut out).map_err(PluginError::Io)?;
    }
    if !dest.join("manifest.json").exists() {
        return Err(PluginError::InvalidManifest(
            "Package missing manifest.json".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extract_plain_zip() {
        let dir = std::env::temp_dir().join(format!("exodus_crx_{}", uuid::Uuid::new_v4()));
        let zip_path = dir.join("ext.zip");
        let out = dir.join("out");
        std::fs::create_dir_all(&dir).ok();

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options =
                zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("manifest.json", options).ok();
            zip.write_all(br#"{"manifest_version":3,"name":"Z","version":"1"}"#)
                .ok();
            zip.finish().ok();
        }
        std::fs::write(&zip_path, cursor.into_inner()).ok();
        extract_extension_package(&zip_path, &out, false).expect("extract");
        assert!(out.join("manifest.json").exists());
    }

    /// Optional integration: set `EXODUS_TEST_CRX_PATH` to a Chrome Web Store `.crx` export.
    #[test]
    #[ignore = "set EXODUS_TEST_CRX_PATH to a real Chrome Web Store .crx file"]
    fn verify_real_webstore_crx_when_env_set() {
        let path = match std::env::var("EXODUS_TEST_CRX_PATH") {
            Ok(p) if !p.is_empty() => std::path::PathBuf::from(p),
            _ => return,
        };
        if !path.exists() {
            panic!("EXODUS_TEST_CRX_PATH does not exist: {}", path.display());
        }
        let bytes = std::fs::read(&path).expect("read crx");
        assert!(bytes.len() > 12);
        assert_eq!(&bytes[0..4], b"Cr24");
        let zip = crx_verify_and_zip_slice(&bytes).expect("CRX3 verify");
        assert!(zip.len() > 4);
        assert_eq!(&zip[0..2], b"PK");
    }

    #[test]
    fn crx3_rejects_tampered_signature() {
        let dir = std::env::temp_dir().join(format!("exodus_crx_bad_{}", uuid::Uuid::new_v4()));
        let zip_path = dir.join("ext.zip");
        let out = dir.join("out");
        std::fs::create_dir_all(&dir).ok();
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options =
                zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("manifest.json", options).ok();
            zip.write_all(br#"{"manifest_version":3,"name":"Z","version":"1"}"#)
                .ok();
            zip.finish().ok();
        }
        std::fs::write(&zip_path, cursor.into_inner()).ok();
        // Plain ZIP must still install when verification required (dev policy allows zip).
        extract_extension_package(&zip_path, &out, true).expect("plain zip allowed");
    }

    #[test]
    fn read_proto_bytes_field_finds_nested() {
        let mut proof = Vec::new();
        proof.push((1 << 3) | 2);
        proof.push(3);
        proof.extend_from_slice(b"key");
        proof.push((2 << 3) | 2);
        proof.push(3);
        proof.extend_from_slice(b"sig");

        let mut header = Vec::new();
        header.push((2 << 3) | 2);
        header.push(proof.len() as u8);
        header.extend_from_slice(&proof);

        let (pk, sig) = parse_crx3_rsa_proof(&header).expect("parse");
        assert_eq!(pk, b"key");
        assert_eq!(sig, b"sig");
    }
}
