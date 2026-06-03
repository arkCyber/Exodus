//! Exodus Browser — route history/cookie managers to default vs private profile storage.

use crate::cookie_manager::CookieManager;
use crate::history_manager::HistoryManager;
use crate::profile::{profile_cookie_dir, profile_history_dir};
use std::path::Path;
use std::sync::Arc;

/// Dual history stores (Chrome-style separate incognito history).
#[derive(Clone)]
pub struct ProfileHistoryStores {
    default: Arc<HistoryManager>,
    private: Arc<HistoryManager>,
}

impl ProfileHistoryStores {
    /// Open both profile history directories under `app_data`.
    pub fn new(app_data: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            default: Arc::new(HistoryManager::new(profile_history_dir(
                app_data,
                false,
            ))?),
            private: Arc::new(HistoryManager::new(profile_history_dir(
                app_data,
                true,
            ))?),
        })
    }

    /// History manager for the active profile.
    pub fn active(&self, private_mode: bool) -> &Arc<HistoryManager> {
        if private_mode {
            &self.private
        } else {
            &self.default
        }
    }
}

/// Dual cookie stores for default vs private browsing.
#[derive(Clone)]
pub struct ProfileCookieStores {
    default: Arc<CookieManager>,
    private: Arc<CookieManager>,
}

impl ProfileCookieStores {
    /// Open both profile cookie directories under `app_data`.
    pub fn new(app_data: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            default: Arc::new(CookieManager::new(profile_cookie_dir(app_data, false))?),
            private: Arc::new(CookieManager::new(profile_cookie_dir(app_data, true))?),
        })
    }

    /// Cookie manager for the active profile.
    pub fn active(&self, private_mode: bool) -> &Arc<CookieManager> {
        if private_mode {
            &self.private
        } else {
            &self.default
        }
    }
}

/// Wipe incognito profile storage (history + cookies).
pub fn clear_private_profile(
    history: &ProfileHistoryStores,
    cookies: &ProfileCookieStores,
) -> Result<(), String> {
    history
        .private
        .clear_all()
        .map_err(|e| format!("Clear private history failed: {}", e))?;
    cookies
        .private
        .delete_all_cookies()
        .map_err(|e| format!("Clear private cookies failed: {}", e))?;
    Ok(())
}
