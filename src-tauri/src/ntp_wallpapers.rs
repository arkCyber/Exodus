//! New-tab wallpaper library on disk (`app_data/new_tab_page/wallpapers`).
//!
//! Default wallpaper id is defined in `assets/ntp-wallpaper-manifest.json` (`defaultId`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

/// Catalog entry (matches frontend `manifest.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpWallpaperEntry {
    pub id: String,
    pub name: String,
    pub file: String,
    pub accent: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub custom: bool,
}

#[derive(Debug, Deserialize)]
struct BundledManifest {
    #[allow(dead_code)]
    version: u32,
    #[serde(rename = "defaultId")]
    default_id: String,
    wallpapers: Vec<NtpWallpaperEntry>,
}

const BUNDLED_MANIFEST: &str = include_str!("../assets/ntp-wallpaper-manifest.json");

/// Subdirectory under app data for user + seeded wallpapers.
pub fn wallpapers_dir(app_data: &Path) -> PathBuf {
    app_data.join("new_tab_page").join("wallpapers")
}

/// Copy bundled SVG wallpapers into the user library directory (idempotent).
pub fn seed_wallpaper_library(app_data: &Path) -> Result<PathBuf, String> {
    let dir = wallpapers_dir(app_data);
    fs::create_dir_all(&dir).map_err(|e| format!("Create wallpaper dir failed: {}", e))?;

    let manifest: BundledManifest =
        serde_json::from_str(BUNDLED_MANIFEST).map_err(|e| format!("Manifest parse: {}", e))?;

    for entry in &manifest.wallpapers {
        let dest = dir.join(&entry.file);
        if dest.exists() {
            continue;
        }
        let asset_path = format!("assets/ntp-wallpapers/{}", entry.file);
        let svg = load_embedded_svg(&entry.file, &asset_path)?;
        fs::write(&dest, svg).map_err(|e| format!("Write wallpaper {}: {}", entry.file, e))?;
    }

    let user_manifest = dir.join("user-manifest.json");
    if !user_manifest.exists() {
        let hint = serde_json::json!({
            "hint": "Drop .svg, .png, or .jpg files here. They appear in Settings and the new-tab picker."
        });
        let json = serde_json::to_string_pretty(&hint)
            .map_err(|e| format!("Serialize hint: {}", e))?;
        fs::write(&user_manifest, json)
            .map_err(|e| format!("Write user-manifest: {}", e))?;
    }

    Ok(dir)
}

fn load_embedded_svg(file: &str, _asset_path: &str) -> Result<String, String> {
    match file {
        "aurora.svg" => Ok(include_str!("../assets/ntp-wallpapers/aurora.svg").to_string()),
        "ocean.svg" => Ok(include_str!("../assets/ntp-wallpapers/ocean.svg").to_string()),
        "sunset.svg" => Ok(include_str!("../assets/ntp-wallpapers/sunset.svg").to_string()),
        "forest.svg" => Ok(include_str!("../assets/ntp-wallpapers/forest.svg").to_string()),
        "nebula.svg" => Ok(include_str!("../assets/ntp-wallpapers/nebula.svg").to_string()),
        "midnight.svg" => Ok(include_str!("../assets/ntp-wallpapers/midnight.svg").to_string()),
        other => Err(format!("Unknown bundled wallpaper: {}", other)),
    }
}

/// List bundled + custom wallpapers for the UI catalog.
pub fn list_wallpaper_catalog(app_data: &Path) -> Result<Vec<NtpWallpaperEntry>, String> {
    let _ = seed_wallpaper_library(app_data)?;
    let dir = wallpapers_dir(app_data);

    let manifest: BundledManifest =
        serde_json::from_str(BUNDLED_MANIFEST).map_err(|e| format!("Manifest parse: {}", e))?;

    let mut out: Vec<NtpWallpaperEntry> = manifest
        .wallpapers
        .into_iter()
        .map(|mut w| {
            w.custom = false;
            w
        })
        .collect();

    if dir.is_dir() {
        for entry in fs::read_dir(&dir).map_err(|e| format!("Read wallpaper dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Read dir entry: {}", e))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if file_name.starts_with('.') || file_name == "user-manifest.json" {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !["svg", "png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                continue;
            }
            if out.iter().any(|w| w.file == file_name) {
                continue;
            }
            let id = file_name
                .trim_end_matches(&format!(".{}", ext))
                .to_string();
            out.push(NtpWallpaperEntry {
                id: format!("custom-{}", id),
                name: id.replace('-', " "),
                file: file_name,
                accent: "#94a3b8".to_string(),
                description: Some("Custom wallpaper from library folder".to_string()),
                custom: true,
            });
        }
    }

    Ok(out)
}

/// Default wallpaper id from bundled manifest (`defaultId` in JSON).
pub fn default_wallpaper_id() -> String {
    serde_json::from_str::<BundledManifest>(BUNDLED_MANIFEST)
        .map(|m| m.default_id)
        .unwrap_or_else(|_| "nebula".to_string())
}

#[tauri::command]
pub fn ntp_get_default_wallpaper_id() -> String {
    default_wallpaper_id()
}

#[tauri::command]
pub fn ntp_get_wallpaper_library_path(app: tauri::AppHandle) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {}", e))?;
    let dir = seed_wallpaper_library(&app_data)?;
    Ok(dir.to_string_lossy().to_string())
}

/// Read a wallpaper file as a data URL (for WebView underlay + custom images).
#[tauri::command]
pub fn ntp_wallpaper_file_data_url(app: tauri::AppHandle, file: String) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {}", e))?;
    let path = wallpapers_dir(&app_data).join(&file);
    if !path.is_file() {
        return Err(format!("Wallpaper not found: {}", file));
    }
    let bytes = fs::read(&path).map_err(|e| format!("Read wallpaper: {}", e))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
pub fn ntp_list_wallpaper_catalog(app: tauri::AppHandle) -> Result<Vec<NtpWallpaperEntry>, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {}", e))?;
    list_wallpaper_catalog(&app_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn seed_and_list_wallpapers() {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        seed_wallpaper_library(tmp.path()).expect("Failed to seed wallpaper library");
        let list = list_wallpaper_catalog(tmp.path()).expect("Failed to list wallpapers");
        assert!(list.iter().any(|w| w.id == "aurora"));
    }
}
