//! Exodus Browser — main window visibility and macOS dock policy at startup.

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::startup_log;

/// Keep Dock icon visible on macOS (Regular activation policy).
pub fn configure_macos_app_policy(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri::ActivationPolicy;
        match app.set_activation_policy(ActivationPolicy::Regular) {
            Ok(()) => startup_log::log_step("macOS activation policy set to Regular (dock icon)"),
            Err(e) => startup_log::log_error(&format!("set_activation_policy failed: {e}")),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
    }
}

/// Set the dock icon for the application (for dev mode).
pub fn set_dock_icon(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use std::path::PathBuf;
        
        // Try to load icon from the icons directory
        let icon_path = app.path().resolve("icons/icon.icns", tauri::path::BaseDirectory::Resource);
        
        match icon_path {
            Ok(path) if path.exists() => {
                startup_log::log_step(&format!("Setting dock icon from: {:?}", path));
                // Use NSApplication to set the application icon
                unsafe {
                    use cocoa::appkit::NSApp;
                    use cocoa::foundation::NSString;
                    use objc::{msg_send, sel, sel_impl};
                    
                    let ns_app: cocoa::base::id = NSApp();
                    let ns_string: cocoa::base::id = NSString::alloc(cocoa::base::nil).init_str(&path.to_string_lossy());
                    
                    let _: () = msg_send![ns_app, setApplicationIconImage: ns_string];
                    startup_log::log_step("Dock icon set successfully");
                }
            }
            Ok(path) => {
                startup_log::log_warn(&format!("Icon file not found at: {:?}", path));
            }
            Err(e) => {
                startup_log::log_warn(&format!("Failed to resolve icon path: {}", e));
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
    }
}

/// Log main window label, URL, visibility, and focus for diagnostics.
pub fn log_main_window_state(app: &AppHandle, phase: &str) {
    let Some(win) = app.get_webview_window("main") else {
        startup_log::log_error(&format!("{phase}: main webview window not found"));
        return;
    };
    let url = win.url().map(|u| u.to_string()).unwrap_or_else(|e| format!("err:{e}"));
    let visible = win.is_visible().unwrap_or(false);
    let focused = win.is_focused().unwrap_or(false);
    let minimized = win.is_minimized().unwrap_or(false);
    startup_log::log_step(&format!(
        "{phase}: main window url={url} visible={visible} focused={focused} minimized={minimized}"
    ));
}

/// Show, unminimize, and focus the main window (fixes hidden-on-launch / tray-only).
pub fn ensure_main_window_visible(app: &AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        startup_log::log_error("ensure_main_window_visible: main window missing");
        return;
    };
    apply_show_and_focus(&win);
    log_main_window_state(app, "after ensure_main_window_visible");
}

fn apply_show_and_focus(win: &WebviewWindow) {
    if let Err(e) = win.show() {
        startup_log::log_error(&format!("main.show failed: {e}"));
    }
    if let Err(e) = win.unminimize() {
        startup_log::log_warn(&format!("main.unminimize failed: {e}"));
    }
    if let Err(e) = win.set_focus() {
        startup_log::log_warn(&format!("main.set_focus failed: {e}"));
    }
}
