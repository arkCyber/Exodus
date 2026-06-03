//! Exodus Browser — Web Extension (Manifest V3) and Native Rust plugin subsystem.
//!
//! Provides extension loading, content-script injection, chrome.storage.local backend,
//! and native Rust plugin support via dynamic libraries.

pub mod background;
pub mod chrome_bridge;
pub mod commands;
pub mod crx;
pub mod error;
pub mod extension_url;
mod inject;
mod manager;
pub mod manifest;
mod match_patterns;
pub mod native_commands;
pub mod native_plugin;
pub mod performance_tests;
pub mod permissions;
pub mod runtime;
pub mod security_tests;
mod storage;
mod tabs;
pub mod notifications;
pub mod net_rules;
pub mod web_request;
pub mod extension_popup;
pub mod permission_pending;
pub mod site_permissions;
pub mod host_install_pending;
pub mod browser_site_permissions;

pub use commands::dev_extensions_dir;
pub use manager::{ExtensionManager, ExtensionState};
pub use native_plugin::NativePluginManager;
pub use tabs::TabRegistry;

/// Log plugin module initialization timestamp.
pub fn log_plugin_module_init() {
    println!(
        "[{}] exodus plugins (Web Extension) module loaded",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
    );
}
