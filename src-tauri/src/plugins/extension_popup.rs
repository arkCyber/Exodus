//! Extension action popup — dedicated `WebviewWindow` + `chrome.action` helpers.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, State};
use tauri_utils::config::WebviewUrl;
use url::Url as StdUrl;

use super::chrome_bridge::build_extension_popup_prelude;
use super::extension_url::{file_url_for_path, parse_extension_url, resolve_extension_file};
use super::manager::ExtensionState;
use super::tabs::TabRegistry;

/// Per-extension action UI state (title/badge).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionActionState {
    pub title: Option<String>,
    pub badge_text: Option<String>,
    pub badge_background_color: Option<String>,
}

/// Store for `chrome.action` metadata.
pub struct ExtensionActionStore {
    states: Arc<Mutex<std::collections::HashMap<String, ExtensionActionState>>>,
}

impl Default for ExtensionActionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionActionStore {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn set_title(&self, extension_id: &str, title: Option<String>) {
        if let Ok(mut m) = self.states.lock() {
            m.entry(extension_id.to_string())
                .or_default()
                .title = title;
        }
    }

    pub fn set_badge(
        &self,
        extension_id: &str,
        text: Option<String>,
        color: Option<String>,
    ) {
        if let Ok(mut m) = self.states.lock() {
            let e = m.entry(extension_id.to_string()).or_default();
            e.badge_text = text;
            e.badge_background_color = color;
        }
    }

    pub fn get(&self, extension_id: &str) -> ExtensionActionState {
        self.states
            .lock()
            .ok()
            .and_then(|m| m.get(extension_id).cloned())
            .unwrap_or_default()
    }
}

fn popup_window_label(extension_id: &str) -> String {
    format!("exodus-ext-win-{extension_id}")
}

/// Open extension action popup in a dedicated window (not an overlay on the tab strip).
#[tauri::command]
pub fn extension_open_popup_window(
    app: AppHandle,
    ext_state: State<'_, ExtensionState>,
    extension_id: String,
) -> Result<(), String> {
    let label = popup_window_label(&extension_id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.set_focus();
        return Ok(());
    }

    let (file_url, title, init_script) = {
        let mgr = ext_state
            .lock()
            .map_err(|e| format!("Extension lock: {}", e))?;
        let title = mgr.display_name_for(&extension_id);
        let file_url = {
            let popup_path = mgr
                .popup_path_for(&extension_id)
                .ok_or_else(|| "Extension has no action popup".to_string())?;
            let ext_url =
                parse_extension_url(&format!("extension://{extension_id}/{popup_path}"))
                    .ok_or_else(|| "Invalid popup URL".to_string())?;
            let path = resolve_extension_file(&mgr, &ext_url).map_err(|e| e.to_string())?;
            file_url_for_path(&path).map_err(|e| e.to_string())?
        };
        let tabs = app
            .try_state::<TabRegistry>()
            .ok_or_else(|| "Tab registry missing".to_string())?;
        let init_script = build_extension_popup_prelude(&mgr, &tabs, &extension_id);
        (file_url, title, init_script)
    };

    let parsed = StdUrl::parse(&file_url).map_err(|e| format!("Popup URL parse: {}", e))?;
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
        .title(title)
        .inner_size(420.0, 560.0)
        .resizable(true)
        .initialization_script(&init_script)
        .build()
        .map_err(|e| format!("Create popup window failed: {}", e))?;

    Ok(())
}

/// Close extension popup window if open.
#[tauri::command]
pub fn extension_close_popup_window(app: AppHandle, extension_id: String) -> Result<(), String> {
    let label = popup_window_label(&extension_id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close()
            .map_err(|e| format!("Close popup window failed: {}", e))?;
    }
    Ok(())
}

/// `chrome.action.setTitle` backend.
#[tauri::command]
pub fn extension_action_set_title(
    extension_id: String,
    title: Option<String>,
    store: State<'_, Arc<ExtensionActionStore>>,
) -> Result<(), String> {
    store.set_title(&extension_id, title);
    Ok(())
}

/// `chrome.action.setBadgeText` / background color backend.
#[tauri::command]
pub fn extension_action_set_badge(
    extension_id: String,
    text: Option<String>,
    color: Option<String>,
    store: State<'_, Arc<ExtensionActionStore>>,
) -> Result<(), String> {
    store.set_badge(&extension_id, text, color);
    Ok(())
}

/// Read action state for toolbar rendering.
#[tauri::command]
pub fn extension_action_get_state(
    extension_id: String,
    store: State<'_, Arc<ExtensionActionStore>>,
) -> Result<ExtensionActionState, String> {
    Ok(store.get(&extension_id))
}
