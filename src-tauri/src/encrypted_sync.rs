//! Encrypted bookmark sync vault (local AES-256-GCM at rest; optional cloud PUT/GET).

use base64::Engine;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

const VAULT_FILE: &str = "sync_vault.enc";
const SALT_FILE: &str = "sync_salt.bin";
const NONCE_LEN: usize = 12;

/// Encrypted sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSyncSettings {
    pub enabled: bool,
    pub has_passphrase: bool,
    pub last_sync_at: u64,
    /// HTTPS endpoint base for vault sync (e.g. `https://sync.example.com/api`).
    #[serde(default)]
    pub sync_server_url: Option<String>,
    /// Bearer token or API key for sync server (optional).
    #[serde(default)]
    pub sync_token: Option<String>,
    /// Stable device id for remote vault path.
    #[serde(default)]
    pub device_id: Option<String>,
}

impl Default for EncryptedSyncSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            has_passphrase: false,
            last_sync_at: 0,
            sync_server_url: None,
            sync_token: None,
            device_id: None,
        }
    }
}

/// Manages passphrase-derived encryption for sync payloads.
pub struct EncryptedSyncManager {
    settings: Arc<Mutex<EncryptedSyncSettings>>,
    passphrase: Arc<Mutex<Option<Vec<u8>>>>,
    salt: Arc<Mutex<Option<Vec<u8>>>>,
    storage_path: PathBuf,
}

impl EncryptedSyncManager {
    /// Open encrypted sync storage under `storage_path`.
    pub fn new(storage_path: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&storage_path).map_err(|e| format!("Create sync dir: {}", e))?;
        let mgr = Self {
            settings: Arc::new(Mutex::new(EncryptedSyncSettings::default())),
            passphrase: Arc::new(Mutex::new(None)),
            salt: Arc::new(Mutex::new(None)),
            storage_path,
        };
        mgr.load_settings()?;
        mgr.load_salt()?;
        mgr.ensure_device_id()?;
        Ok(mgr)
    }

    fn device_id_path(&self) -> PathBuf {
        self.storage_path.join("sync_device_id.txt")
    }

    fn ensure_device_id(&self) -> Result<String, String> {
        if let Ok(s) = self.settings.lock() {
            if let Some(id) = &s.device_id {
                if !id.is_empty() {
                    return Ok(id.clone());
                }
            }
        }
        let path = self.device_id_path();
        if path.exists() {
            let id = std::fs::read_to_string(&path).map_err(|e| format!("Read device id: {}", e))?;
            let id = id.trim().to_string();
            if let Ok(mut s) = self.settings.lock() {
                s.device_id = Some(id.clone());
            }
            return Ok(id);
        }
        let id = uuid::Uuid::new_v4().to_string();
        std::fs::write(&path, &id).map_err(|e| format!("Write device id: {}", e))?;
        if let Ok(mut s) = self.settings.lock() {
            s.device_id = Some(id.clone());
        }
        self.save_settings()?;
        Ok(id)
    }

    pub fn set_sync_server(&self, url: Option<String>, token: Option<String>) -> Result<(), String> {
        let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
        s.sync_server_url = url.filter(|u| u.starts_with("http"));
        if let Some(t) = token {
            s.sync_token = if t.trim().is_empty() { None } else { Some(t) };
        }
        self.save_settings()
    }

    fn settings_path(&self) -> PathBuf {
        self.storage_path.join("encrypted_sync_settings.json")
    }

    fn vault_path(&self) -> PathBuf {
        self.storage_path.join(VAULT_FILE)
    }

    fn salt_path(&self) -> PathBuf {
        self.storage_path.join(SALT_FILE)
    }

    fn load_settings(&self) -> Result<(), String> {
        let path = self.settings_path();
        if !path.exists() {
            return Ok(());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| format!("Read settings: {}", e))?;
        let s: EncryptedSyncSettings =
            serde_json::from_str(&text).map_err(|e| format!("Parse settings: {}", e))?;
        if let Ok(mut cur) = self.settings.lock() {
            *cur = s;
        }
        Ok(())
    }

    fn save_settings(&self) -> Result<(), String> {
        let s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
        let text =
            serde_json::to_string_pretty(&*s).map_err(|e| format!("Serialize settings: {}", e))?;
        std::fs::write(self.settings_path(), text).map_err(|e| format!("Write settings: {}", e))?;
        Ok(())
    }

    fn load_salt(&self) -> Result<(), String> {
        let path = self.salt_path();
        if !path.exists() {
            return Ok(());
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("Read salt: {}", e))?;
        if let Ok(mut salt) = self.salt.lock() {
            *salt = Some(bytes);
        }
        Ok(())
    }

    fn ensure_salt(&self) -> Result<Vec<u8>, String> {
        let mut salt_guard = self.salt.lock().map_err(|e| format!("Lock: {}", e))?;
        if let Some(s) = salt_guard.clone() {
            return Ok(s);
        }
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; 16];
        rng.fill(&mut salt)
            .map_err(|_| "RNG salt failed".to_string())?;
        std::fs::write(self.salt_path(), &salt).map_err(|e| format!("Write salt: {}", e))?;
        *salt_guard = Some(salt.clone());
        Ok(salt)
    }

    fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<[u8; 32], String> {
        let mut key = [0u8; 32];
        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(100_000).expect("100_000 should be a valid non-zero value"),
            salt,
            passphrase,
            &mut key,
        );
        Ok(key)
    }

    fn less_safe_key(&self) -> Result<LessSafeKey, String> {
        let pass = self
            .passphrase
            .lock()
            .map_err(|e| format!("Lock: {}", e))?
            .clone()
            .ok_or_else(|| "Sync passphrase not set".to_string())?;
        let salt = self.ensure_salt()?;
        let key_bytes = Self::derive_key(&pass, &salt)?;
        let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| "AES key init failed".to_string())?;
        Ok(LessSafeKey::new(unbound))
    }

    /// Set sync passphrase (enables encrypted vault).
    pub fn set_passphrase(&self, passphrase: String) -> Result<(), String> {
        if passphrase.len() < 8 {
            return Err("Passphrase must be at least 8 characters".to_string());
        }
        {
            let mut pass = self.passphrase.lock().map_err(|e| format!("Lock: {}", e))?;
            *pass = Some(passphrase.into_bytes());
        }
        self.ensure_salt()?;
        {
            let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            s.has_passphrase = true;
            s.enabled = true;
        }
        self.save_settings()
    }

    pub fn get_settings(&self) -> EncryptedSyncSettings {
        self.settings
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Encrypt bookmark sync JSON and write vault file (nonce || ciphertext+tag).
    pub fn encrypt_and_store_vault(&self, plaintext_json: &str) -> Result<String, String> {
        let key = self.less_safe_key()?;
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| "RNG nonce failed".to_string())?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext_json.as_bytes().to_vec();
        let tag = key
            .seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| "Encrypt failed".to_string())?;
        let mut file_bytes = Vec::with_capacity(NONCE_LEN + in_out.len() + tag.as_ref().len());
        file_bytes.extend_from_slice(&nonce_bytes);
        file_bytes.extend_from_slice(&in_out);
        file_bytes.extend_from_slice(tag.as_ref());
        std::fs::write(self.vault_path(), &file_bytes)
            .map_err(|e| format!("Write vault: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
        {
            let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            s.last_sync_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        self.save_settings()?;
        Ok(b64)
    }

    /// Decrypt vault file to JSON string.
    pub fn decrypt_vault(&self) -> Result<String, String> {
        let file_bytes = std::fs::read(self.vault_path()).map_err(|e| format!("Read vault: {}", e))?;
        if file_bytes.len() < NONCE_LEN + 16 {
            return Err("Vault file too short".to_string());
        }
        let (nonce_bytes, rest) = file_bytes.split_at(NONCE_LEN);
        let mut nonce_arr = [0u8; NONCE_LEN];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_arr);
        let key = self.less_safe_key()?;
        let mut in_out = rest.to_vec();
        let plain = key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| "Decrypt failed (wrong passphrase?)".to_string())?;
        String::from_utf8(plain.to_vec()).map_err(|e| format!("UTF-8: {}", e))
    }

    /// Upload encrypted vault blob to configured sync server (PUT).
    pub async fn upload_vault_to_cloud(&self) -> Result<String, String> {
        let (base, token, device_id) = {
            let s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            let base = s
                .sync_server_url
                .clone()
                .ok_or_else(|| "Sync server URL not configured".to_string())?;
            (base, s.sync_token.clone(), s.device_id.clone())
        };
        let device_id = device_id.unwrap_or_else(|| self.ensure_device_id().unwrap_or_default());
        if device_id.is_empty() {
            return Err("Device id missing".to_string());
        }
        if !self.vault_path().exists() {
            return Err("Local vault empty — encrypt bookmarks first".to_string());
        }
        let bytes = std::fs::read(self.vault_path()).map_err(|e| format!("Read vault: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let url = format!(
            "{}/vault/{}",
            base.trim_end_matches('/'),
            device_id
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;
        let mut req = client.put(&url).json(&serde_json::json!({
            "deviceId": device_id,
            "payload": b64,
            "updatedAt": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }));
        if let Some(tok) = token.filter(|t| !t.is_empty()) {
            req = req.bearer_auth(tok);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("Upload failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Upload HTTP {}", resp.status()));
        }
        {
            let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            s.last_sync_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        self.save_settings()?;
        Ok(format!("Uploaded {} bytes to {}", bytes.len(), url))
    }

    /// Download encrypted vault from sync server and replace local file.
    pub async fn download_vault_from_cloud(&self) -> Result<usize, String> {
        let (base, token, device_id) = {
            let s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            let base = s
                .sync_server_url
                .clone()
                .ok_or_else(|| "Sync server URL not configured".to_string())?;
            (base, s.sync_token.clone(), s.device_id.clone())
        };
        let device_id = device_id.unwrap_or_else(|| self.ensure_device_id().unwrap_or_default());
        let url = format!(
            "{}/vault/{}",
            base.trim_end_matches('/'),
            device_id
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;
        let mut req = client.get(&url);
        if let Some(tok) = token.filter(|t| !t.is_empty()) {
            req = req.bearer_auth(tok);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Download HTTP {}", resp.status()));
        }
        #[derive(serde::Deserialize)]
        struct RemoteVault {
            payload: String,
        }
        let body: RemoteVault = resp
            .json()
            .await
            .map_err(|e| format!("Parse response: {}", e))?;
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(body.payload.trim())
            .map_err(|e| format!("Base64 decode: {}", e))?;
        std::fs::write(self.vault_path(), &bytes).map_err(|e| format!("Write vault: {}", e))?;
        Ok(bytes.len())
    }
}

#[tauri::command]
pub fn encrypted_sync_get_settings(
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<EncryptedSyncSettings, String> {
    Ok(mgr.get_settings())
}

#[tauri::command]
pub fn encrypted_sync_set_passphrase(
    passphrase: String,
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<(), String> {
    mgr.set_passphrase(passphrase)
}

#[tauri::command]
pub fn encrypted_sync_store_bookmarks(
    bookmarks_json: String,
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<String, String> {
    mgr.encrypt_and_store_vault(&bookmarks_json)
}

#[tauri::command]
pub fn encrypted_sync_load_bookmarks(
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<String, String> {
    mgr.decrypt_vault()
}

#[tauri::command]
pub fn encrypted_sync_set_server(
    sync_server_url: Option<String>,
    sync_token: Option<String>,
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<(), String> {
    mgr.set_sync_server(sync_server_url, sync_token)
}

#[tauri::command]
pub async fn encrypted_sync_upload_vault(
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<String, String> {
    mgr.upload_vault_to_cloud().await
}

#[tauri::command]
pub async fn encrypted_sync_download_vault(
    mgr: State<'_, Arc<EncryptedSyncManager>>,
) -> Result<usize, String> {
    mgr.download_vault_from_cloud().await
}
