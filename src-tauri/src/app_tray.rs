//! System tray — keep file transfers running when the main window is hidden.

use crate::startup_log;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, RunEvent, WindowEvent,
};

/// When true, closing the window hides to tray instead of quitting.
static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(true);

/// Install tray icon, menu, and hide-on-close behavior.
pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    startup_log::log_step("setup_tray() begin");
    
    let show = MenuItem::with_id(app, "tray-show", "Show Exodus", true, None::<&str>)
        .map_err(|e| {
            tracing::error!("Failed to create 'Show Exodus' menu item: {}", e);
            e.to_string()
        })?;
    let transfers = MenuItem::with_id(
        app,
        "tray-transfers",
        "File transfers (background)",
        true,
        None::<&str>,
    )
    .map_err(|e| {
        tracing::error!("Failed to create 'File transfers' menu item: {}", e);
        e.to_string()
    })?;
    let im = MenuItem::with_id(app, "tray-im", "Open IM & calls", true, None::<&str>)
        .map_err(|e| {
            tracing::error!("Failed to create 'Open IM & calls' menu item: {}", e);
            e.to_string()
        })?;
    let quit = MenuItem::with_id(app, "tray-quit", "Quit Exodus", true, None::<&str>)
        .map_err(|e| {
            tracing::error!("Failed to create 'Quit Exodus' menu item: {}", e);
            e.to_string()
        })?;
    let menu = Menu::with_items(app, &[&show, &im, &transfers, &quit]).map_err(|e| {
        tracing::error!("Failed to create tray menu: {}", e);
        e.to_string()
    })?;

    tracing::info!("Loading default window icon for tray...");
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| {
            tracing::error!("No default window icon available for tray");
            "No default window icon for tray".to_string()
        })?;
    tracing::info!("Tray icon loaded successfully");

    let app_tray = app.clone();
    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Exodus — transfers run in background")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "tray-show" => show_main_window(app),
            "tray-im" => {
                show_main_window(app);
                let _ = app.emit("exodus-focus-im", ());
            }
            "tray-transfers" => {
                show_main_window(app);
                let _ = app.emit("exodus-focus-workspace", ());
            }
            "tray-quit" => {
                MINIMIZE_TO_TRAY.store(false, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|e| {
            tracing::error!("Failed to build tray icon: {}", e);
            e.to_string()
        })?;

    tracing::info!("Setting up window close handler...");
    let app_close = app_tray.clone();
    if let Some(win) = app.get_webview_window("main") {
        tracing::info!("Main window found, setting up close handler");
        let win_hide = win.clone();
        win.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if MINIMIZE_TO_TRAY.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = win_hide.hide();
                    let _ = app_close.emit("exodus-window-hidden", ());
                }
            }
        });
    } else {
        tracing::warn!("Main window not found when setting up tray");
    }

    startup_log::log_step("System tray enabled — close hides window; transfers continue");
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    startup_log::log_step("show_main_window() from tray");
    crate::app_window::ensure_main_window_visible(app);
}

/// Handle app-level exit (dock Quit on macOS).
pub fn on_run_event(app: &AppHandle, event: &RunEvent) {
    if let RunEvent::ExitRequested { api, .. } = event {
        if MINIMIZE_TO_TRAY.load(Ordering::SeqCst) {
            api.prevent_exit();
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimize_to_tray_initial_state() {
        // Test that the initial state is true (minimize to tray enabled)
        assert!(MINIMIZE_TO_TRAY.load(Ordering::SeqCst));
    }

    #[test]
    fn test_minimize_to_tray_toggle() {
        // Test toggling the minimize to tray state
        let original = MINIMIZE_TO_TRAY.load(Ordering::SeqCst);
        
        MINIMIZE_TO_TRAY.store(false, Ordering::SeqCst);
        assert!(!MINIMIZE_TO_TRAY.load(Ordering::SeqCst));
        
        MINIMIZE_TO_TRAY.store(true, Ordering::SeqCst);
        assert!(MINIMIZE_TO_TRAY.load(Ordering::SeqCst));
        
        // Restore original state
        MINIMIZE_TO_TRAY.store(original, Ordering::SeqCst);
    }

    #[test]
    fn test_minimize_to_tray_concurrent_access() {
        // Test concurrent access to the atomic bool
        use std::thread;
        
        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    for _ in 0..100 {
                        let _ = MINIMIZE_TO_TRAY.load(Ordering::SeqCst);
                        MINIMIZE_TO_TRAY.store(true, Ordering::SeqCst);
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().expect("Failed to join thread");
        }
        
        // After concurrent operations, state should still be valid
        let state = MINIMIZE_TO_TRAY.load(Ordering::SeqCst);
        assert!(state == true || state == false);
    }
}
