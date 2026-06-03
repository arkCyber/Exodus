//! Exodus Browser — per-extension local storage (chrome.storage.local backend).

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::error::PluginError;
use super::permissions::{require_permission, Permission};

/// JSON file-backed storage per extension id.
pub struct ExtensionStorage {
    root: PathBuf,
}

impl ExtensionStorage {
    /// Open or create storage under `app_data/extensions/storage`.
    pub fn new(app_data_dir: &Path) -> Result<Self, PluginError> {
        let root = app_data_dir.join("extensions").join("storage");
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path_for(&self, extension_id: &str) -> PathBuf {
        self.root.join(format!("{extension_id}.json"))
    }

    fn load_map(&self, extension_id: &str) -> Result<Map<String, Value>, PluginError> {
        let path = self.path_for(extension_id);
        if !path.exists() {
            return Ok(Map::new());
        }
        let raw = std::fs::read_to_string(&path)?;
        let value: Value = serde_json::from_str(&raw)?;
        match value {
            Value::Object(map) => Ok(map),
            _ => Err(PluginError::Parse("Storage root must be object".into())),
        }
    }

    fn save_map(&self, extension_id: &str, map: &Map<String, Value>) -> Result<(), PluginError> {
        let path = self.path_for(extension_id);
        let json = serde_json::to_string_pretty(map)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Read keys from extension storage (chrome.storage.local.get).
    pub fn get(
        &self,
        extension_id: &str,
        granted: &[Permission],
        keys: Option<Vec<String>>,
    ) -> Result<Map<String, Value>, PluginError> {
        require_permission(extension_id, granted, Permission::Storage)?;
        let map = self.load_map(extension_id)?;
        Ok(select_keys(&map, keys))
    }

    /// Write items to extension storage (chrome.storage.local.set).
    pub fn set(
        &self,
        extension_id: &str,
        granted: &[Permission],
        items: Map<String, Value>,
    ) -> Result<(), PluginError> {
        require_permission(extension_id, granted, Permission::Storage)?;
        let mut map = self.load_map(extension_id)?;
        for (k, v) in items {
            map.insert(k, v);
        }
        self.save_map(extension_id, &map)
    }

    /// Remove keys from extension storage (chrome.storage.local.remove).
    #[allow(dead_code)]
    pub fn remove(
        &self,
        extension_id: &str,
        granted: &[Permission],
        keys: Vec<String>,
    ) -> Result<(), PluginError> {
        require_permission(extension_id, granted, Permission::Storage)?;
        let mut map = self.load_map(extension_id)?;
        for key in keys {
            map.remove(&key);
        }
        self.save_map(extension_id, &map)
    }

    /// Clear all extension storage (chrome.storage.local.clear).
    #[allow(dead_code)]
    pub fn clear(
        &self,
        extension_id: &str,
        granted: &[Permission],
    ) -> Result<(), PluginError> {
        require_permission(extension_id, granted, Permission::Storage)?;
        self.save_map(extension_id, &Map::new())
    }

    /// Remove extension storage file on uninstall.
    pub fn remove_extension(&self, extension_id: &str) -> Result<(), PluginError> {
        let path = self.path_for(extension_id);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

fn select_keys(map: &Map<String, Value>, keys: Option<Vec<String>>) -> Map<String, Value> {
    match keys {
        None => map.clone(),
        Some(list) if list.is_empty() => map.clone(),
        Some(list) => {
            let mut out = Map::new();
            for key in list {
                if let Some(v) = map.get(&key) {
                    out.insert(key, v.clone());
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::permissions::Permission;
    use serde_json::json;

    fn temp_storage() -> ExtensionStorage {
        let dir = std::env::temp_dir().join(format!("exodus_ext_store_{}", uuid::Uuid::new_v4()));
        ExtensionStorage::new(&dir).expect("storage")
    }

    #[test]
    fn set_and_get_roundtrip() {
        let store = temp_storage();
        let perms = vec![Permission::Storage];
        let mut items = Map::new();
        items.insert("count".into(), json!(1));
        store.set("ext-a", &perms, items).expect("set");
        let got = store
            .get("ext-a", &perms, Some(vec!["count".into()]))
            .expect("get");
        assert_eq!(got.get("count"), Some(&json!(1)));
    }

    #[test]
    fn remove_keys() {
        let store = temp_storage();
        let perms = vec![Permission::Storage];
        let mut items = Map::new();
        items.insert("a".into(), json!(1));
        items.insert("b".into(), json!(2));
        store.set("ext-a", &perms, items).expect("set");
        store
            .remove("ext-a", &perms, vec!["a".into()])
            .expect("remove");
        let got = store.get("ext-a", &perms, None).expect("get");
        assert!(!got.contains_key("a"));
        assert!(got.contains_key("b"));
    }

    #[test]
    fn storage_requires_permission() {
        let store = temp_storage();
        let err = store
            .get("ext-a", &[], None)
            .expect_err("denied");
        assert!(matches!(err, PluginError::PermissionDenied { .. }));
    }
}
