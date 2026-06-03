//! Chrome Extension API - chrome.cookies

use crate::config::ConfigState;
use crate::cookie_manager::{CookieEntry, CookieManager};
use crate::plugins::manager::ExtensionManager;
use crate::profile_stores::ProfileCookieStores;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use url::Url;

/// Active profile cookie store from privacy mode flag.
fn active_cookies<'a>(
    stores: &'a ProfileCookieStores,
    config: &ConfigState,
) -> Result<&'a Arc<CookieManager>, String> {
    let private = config
        .lock()
        .map_err(|e| format!("Config lock: {}", e))?
        .private_mode;
    Ok(stores.active(private))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expiration_date: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub url: String,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub value: Option<String>,
    pub expiration_date: Option<f64>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
}

impl From<CookieEntry> for Cookie {
    fn from(entry: CookieEntry) -> Self {
        Self {
            name: entry.name,
            value: entry.value,
            domain: entry.domain,
            path: entry.path,
            secure: entry.secure,
            http_only: entry.http_only,
            expiration_date: if entry.expires == 0 {
                None
            } else {
                Some(entry.expires as f64)
            },
        }
    }
}

#[tauri::command]
pub async fn chrome_cookies_get(
    details: Details,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Option<Cookie>, String> {
    let cookie_manager = active_cookies(&stores, &config)?;
    
    // Parse URL to get domain
    let url = Url::parse(&details.url)
        .map_err(|e| format!("Invalid URL: {}", e))?;
    let domain = url.host_str()
        .ok_or("Invalid URL: no host")?
        .to_string();
    
    // Get cookies for domain
    let cookies = cookie_manager.get_cookies_for_domain(&domain);
    
    // Filter by name if provided
    let filtered = if let Some(ref name) = details.name {
        cookies.into_iter()
            .filter(|c| &c.name == name)
            .collect()
    } else {
        cookies
    };
    
    // Filter by path if provided
    let filtered = if let Some(ref path) = details.path {
        filtered.into_iter()
            .filter(|c| &c.path == path)
            .collect()
    } else {
        filtered
    };
    
    // Return first match
    Ok(filtered.into_iter().next().map(Into::into))
}

#[tauri::command]
pub async fn chrome_cookies_get_all(
    details: Details,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<Cookie>, String> {
    let cookie_manager = active_cookies(&stores, &config)?;
    
    // Parse URL to get domain
    let url = Url::parse(&details.url)
        .map_err(|e| format!("Invalid URL: {}", e))?;
    let domain = url.host_str()
        .ok_or("Invalid URL: no host")?
        .to_string();
    
    // Get cookies for domain
    let cookies = cookie_manager.get_cookies_for_domain(&domain);
    
    // Filter by name if provided
    let filtered = if let Some(ref name) = details.name {
        cookies.into_iter()
            .filter(|c| &c.name == name)
            .collect()
    } else {
        cookies
    };
    
    // Filter by path if provided
    let filtered = if let Some(ref path) = details.path {
        filtered.into_iter()
            .filter(|c| &c.path == path)
            .collect()
    } else {
        filtered
    };
    
    // Convert to Cookie format
    let result: Vec<Cookie> = filtered.into_iter().map(Into::into).collect();
    Ok(result)
}

#[tauri::command]
pub async fn chrome_cookies_set(
    details: Details,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Option<Cookie>, String> {
    let cookie_manager = active_cookies(&stores, &config)?;
    
    // Parse URL to get domain
    let url = Url::parse(&details.url)
        .map_err(|e| format!("Invalid URL: {}", e))?;
    let domain = url.host_str()
        .ok_or("Invalid URL: no host")?
        .to_string();
    
    // Use provided domain or extract from URL
    let cookie_domain = details.domain.unwrap_or(domain);
    
    // Use provided path or default to "/"
    let cookie_path = details.path.unwrap_or("/".to_string());
    
    // Get value (required for set)
    let cookie_value = details.value.ok_or("Cookie value is required")?;
    
    // Calculate expiration
    let expiration = if let Some(exp) = details.expiration_date {
        exp as u64
    } else {
        0 // Session cookie
    };
    
    // Create cookie entry
    let cookie_entry = CookieEntry::new(
        cookie_domain,
        details.name.unwrap_or_default(),
        cookie_value,
        cookie_path,
        expiration,
        details.secure.unwrap_or(false),
        details.http_only.unwrap_or(false),
    );
    
    // Set cookie
    cookie_manager.set_cookie(cookie_entry)
        .map_err(|e| format!("Failed to set cookie: {}", e))?;
    
    // Return the set cookie
    Ok(None) // Would need to retrieve the cookie to return it
}

#[tauri::command]
pub async fn chrome_cookies_remove(
    details: Details,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    let cookie_manager = active_cookies(&stores, &config)?;
    
    // Parse URL to get domain
    let url = Url::parse(&details.url)
        .map_err(|e| format!("Invalid URL: {}", e))?;
    let domain = url.host_str()
        .ok_or("Invalid URL: no host")?
        .to_string();
    
    // Name is required for remove
    let name = details.name.ok_or("Cookie name is required for removal")?;
    
    // Get cookies for domain
    let cookies = cookie_manager.get_cookies_for_domain(&domain);
    
    // Find matching cookie
    if let Some(cookie) = cookies.iter().find(|c| &c.name == &name) {
        cookie_manager.delete_cookie(&cookie.id)
            .map_err(|e| format!("Failed to delete cookie: {}", e))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_serialization() {
        let cookie = Cookie {
            name: "test".to_string(),
            value: "value".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: false,
            expiration_date: Some(1234567890.0),
        };
        let json = serde_json::to_string(&cookie).unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("value"));
        assert!(json.contains("domain"));
    }

    #[test]
    fn test_details_serialization() {
        let details = Details {
            url: "https://example.com".to_string(),
            name: Some("test".to_string()),
            value: Some("value".to_string()),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            secure: Some(true),
            http_only: Some(false),
            expiration_date: Some(1234567890.0),
        };
        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("url"));
        assert!(json.contains("name"));
        assert!(json.contains("domain"));
    }

    #[test]
    fn test_cookie_all_fields() {
        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: ".example.com".to_string(),
            path: "/app".to_string(),
            secure: true,
            http_only: true,
            expiration_date: Some(9999999999.0),
        };
        let json = serde_json::to_string(&cookie).unwrap();
        assert!(json.contains("session"));
        assert!(json.contains("abc123"));
        assert!(json.contains(".example.com"));
        assert!(json.contains("/app"));
    }
}
