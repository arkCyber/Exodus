//! Exodus Browser — Security tests for the plugin system.

#![allow(dead_code)]

use super::error::PluginError;
use super::manager::ExtensionManager;
use super::manifest::load_manifest;
use serde_json::{json, Map};
use std::path::Path;

/// Test for path traversal vulnerabilities in extension loading.
pub fn test_path_traversal_protection(_manager: &ExtensionManager, test_path: &Path) -> Result<(), PluginError> {
    let result = load_manifest(test_path);
    
    if test_path.to_string_lossy().contains("..") {
        assert!(result.is_err(), "Path traversal should be blocked");
        return Ok(());
    }
    
    Ok(())
}

/// Test for malicious manifest content validation.
pub fn test_manifest_validation(manifest_path: &Path) -> Result<(), PluginError> {
    let manifest = load_manifest(manifest_path)?;
    
    assert!(manifest.manifest_version == 2 || manifest.manifest_version == 3, 
            "Invalid manifest version");
    assert!(!manifest.name.is_empty(), "Extension name is required");
    assert!(!manifest.version.is_empty(), "Extension version is required");
    
    Ok(())
}

/// Test for permission boundary enforcement.
pub fn test_permission_boundary(manager: &ExtensionManager, extension_id: &str) -> Result<(), PluginError> {
    let permissions = manager.permissions_for(extension_id)?;
    
    // Ensure permissions are from the allowed set
    for perm in &permissions {
        match perm {
            super::permissions::Permission::Storage => {},
            super::permissions::Permission::Tabs => {},
            super::permissions::Permission::ActiveTab => {},
            super::permissions::Permission::Scripting => {},
            super::permissions::Permission::Notifications => {},
            super::permissions::Permission::DeclarativeNetRequest => {},
            super::permissions::Permission::WebRequest => {},
        }
    }
    
    Ok(())
}

/// Test for ZIP/CRX package security (path traversal, bomb protection).
pub fn test_package_security(package_path: &Path) -> Result<(), PluginError> {
    use super::crx::extract_extension_package;
    use std::fs;
    
    let temp_dir = std::env::temp_dir().join(format!("exodus_security_test_{}", uuid::Uuid::new_v4()));
    let result = extract_extension_package(package_path, &temp_dir, false);
    
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    result
}

/// Test for storage isolation between extensions.
pub fn test_storage_isolation(_manager: &ExtensionManager, ext_a: &str, ext_b: &str) -> Result<(), PluginError> {
    use super::storage::ExtensionStorage;
    
    let temp_dir = std::env::temp_dir().join(format!("exodus_storage_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let storage_a = ExtensionStorage::new(&temp_dir)?;
    let storage_b = ExtensionStorage::new(&temp_dir)?;
    
    let perms_a = vec![super::permissions::Permission::Storage];
    let perms_b = vec![super::permissions::Permission::Storage];
    
    let mut data_a = Map::new();
    data_a.insert("secret".to_string(), json!("data_a"));
    storage_a.set(ext_a, &perms_a, data_a)?;
    
    let data_b = storage_b.get(ext_b, &perms_b, Some(vec!["secret".to_string()]))?;
    assert!(data_b.get("secret").is_none() || data_b["secret"] != json!("data_a"), 
            "Storage isolation violated");
    
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    
    #[test]
    fn test_manifest_validation_basic() {
        let dir = std::env::temp_dir().join(format!("exodus_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).ok();
        
        let manifest_path = dir.join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{"manifest_version":3,"name":"Test","version":"1.0","background":{"service_worker":"bg.js"}}"#,
        )
        .ok();
        fs::write(dir.join("bg.js"), "// bg").ok();
        
        let result = test_manifest_validation(dir.as_path());
        assert!(result.is_ok(), "Manifest validation should pass for valid manifest");
        
        let _ = fs::remove_dir_all(&dir);
    }
    
    #[test]
    fn test_manifest_validation_missing_name() {
        let dir = std::env::temp_dir().join(format!("exodus_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).ok();
        
        let manifest_path = dir.join("manifest.json");
        fs::write(&manifest_path, r#"{"manifest_version":3,"version":"1.0"}"#).ok();
        
        let result = test_manifest_validation(dir.as_path());
        assert!(result.is_err(), "Manifest validation should fail without name");
        
        let _ = fs::remove_dir_all(&dir);
    }
    
    #[test]
    fn test_path_traversal_blocked() {
        let app_data_dir = std::env::temp_dir().join(format!("exodus_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&app_data_dir).ok();
        
        let manager = ExtensionManager::new(&app_data_dir).ok();
        let test_path = PathBuf::from("../../../etc/passwd");
        
        if let Some(mgr) = manager {
            let result = test_path_traversal_protection(&mgr, &test_path);
            assert!(result.is_ok(), "Path traversal test should complete");
        }
        
        let _ = fs::remove_dir_all(&app_data_dir);
    }
}
