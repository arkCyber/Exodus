//! Exodus Browser — MV3 background service worker hosts and runtime messaging.

use serde::{Deserialize, Serialize};
use super::chrome_bridge::{build_extension_prelude_for_background, build_runtime_background_shim};
use super::manager::ExtensionManager;
use super::tabs::TabRegistry;

/// Webview label for an extension background context.
pub fn background_webview_label(extension_id: &str) -> String {
    format!("exodus-ext-bg-{extension_id}")
}

/// Background host metadata for the UI to create hidden webviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionBackgroundSpec {
    pub extension_id: String,
    pub webview_label: String,
    pub boot_script: String,
}

/// Build specs for enabled extensions that declare a background service worker.
pub fn list_background_specs(mgr: &ExtensionManager, tabs: &TabRegistry) -> Vec<ExtensionBackgroundSpec> {
    mgr.list()
        .into_iter()
        .filter(|e| e.enabled)
        .filter_map(|e| {
            mgr.background_boot_script(&e.id, tabs)
                .map(|boot_script| ExtensionBackgroundSpec {
                    extension_id: e.id.clone(),
                    webview_label: background_webview_label(&e.id),
                    boot_script,
                })
        })
        .collect()
}

/// Build the boot script for a background host (prelude + user service worker).
pub fn build_background_boot(
    mgr: &ExtensionManager,
    tabs: &TabRegistry,
    extension_id: &str,
    user_script: &str,
) -> String {
    let prelude = build_extension_prelude_for_background(mgr, tabs, extension_id);
    let runtime = build_runtime_background_shim(extension_id);
    format!(
        r#"(function() {{
{prelude}
{runtime}
try {{
{user_script}
}} catch (e) {{
  console.error('[Exodus background {extension_id}]', e);
}}
}})();"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::manager::ExtensionManager;

    #[test]
    fn background_label_is_stable() {
        assert_eq!(
            background_webview_label("sample-hello"),
            "exodus-ext-bg-sample-hello"
        );
    }

    #[test]
    fn list_specs_includes_background_extension() {
        let dir = std::env::temp_dir().join(format!("exodus_bg_{}", uuid::Uuid::new_v4()));
        let ext_dir = dir.join("plugins/web-extensions/bgext");
        std::fs::create_dir_all(&ext_dir).ok();
        std::fs::write(
            ext_dir.join("manifest.json"),
            r#"{"manifest_version":3,"name":"Bg","version":"1","permissions":["storage"],"background":{"service_worker":"background.js"}}"#,
        )
        .ok();
        std::fs::write(ext_dir.join("background.js"), "self.__bg=true;").ok();
        let mut mgr = ExtensionManager::new(&dir).expect("mgr");
        mgr.scan_and_load(None).ok();
        let tabs = TabRegistry::default();
        let specs = list_background_specs(&mgr, &tabs);
        assert_eq!(specs.len(), 1);
        assert!(specs[0].boot_script.contains("__bg"));
    }
}
