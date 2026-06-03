//! Tracking Protection for Exodus Browser
//! 
//! This module provides tracking protection by blocking known trackers,
//! fingerprinting scripts, and unwanted third-party cookies.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Tracker category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrackerCategory {
    Advertising,
    Analytics,
    Social,
    Fingerprinting,
    Cryptomining,
    Tracking,
}

impl TrackerCategory {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "advertising" => TrackerCategory::Advertising,
            "analytics" => TrackerCategory::Analytics,
            "social" => TrackerCategory::Social,
            "fingerprinting" => TrackerCategory::Fingerprinting,
            "cryptomining" => TrackerCategory::Cryptomining,
            "tracking" => TrackerCategory::Tracking,
            _ => TrackerCategory::Tracking,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            TrackerCategory::Advertising => "advertising",
            TrackerCategory::Analytics => "analytics",
            TrackerCategory::Social => "social",
            TrackerCategory::Fingerprinting => "fingerprinting",
            TrackerCategory::Cryptomining => "cryptomining",
            TrackerCategory::Tracking => "tracking",
        }
    }
}

/// Tracker rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRule {
    /// Rule ID
    pub id: String,
    /// Domain pattern to match
    pub domain: String,
    /// Tracker category
    pub category: TrackerCategory,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Number of times blocked
    pub block_count: u64,
    /// Last blocked timestamp
    pub last_blocked: Option<u64>,
}

impl TrackerRule {
    pub fn new(domain: String, category: TrackerCategory) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            domain,
            category,
            enabled: true,
            block_count: 0,
            last_blocked: None,
        }
    }
    
    pub fn matches(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();
        let domain_lower = self.domain.to_lowercase();
        
        url_lower.contains(&domain_lower) || url_lower.ends_with(&domain_lower)
    }
    
    pub fn record_block(&mut self) {
        self.block_count += 1;
        self.last_blocked = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        );
    }
}

/// Tracking protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingProtectionSettings {
    /// Enable tracking protection
    pub enabled: bool,
    /// Block advertising trackers
    pub block_advertising: bool,
    /// Block analytics trackers
    pub block_analytics: bool,
    /// Block social trackers
    pub block_social: bool,
    /// Block fingerprinting scripts
    pub block_fingerprinting: bool,
    /// Block cryptomining scripts
    pub block_cryptomining: bool,
    /// Block known tracking domains
    pub block_tracking: bool,
    /// Subscription URL for JSON blocklist (`domains` array).
    #[serde(default)]
    pub subscription_url: Option<String>,
    /// Auto-refresh interval in hours (0 = manual only).
    #[serde(default = "default_subscription_refresh_hours")]
    pub subscription_refresh_hours: u32,
    /// Last subscription refresh (unix seconds).
    #[serde(default)]
    pub last_subscription_refresh: u64,
}

fn default_subscription_refresh_hours() -> u32 {
    24
}

impl Default for TrackingProtectionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            block_advertising: true,
            block_analytics: true,
            block_social: false,
            block_fingerprinting: true,
            block_cryptomining: true,
            block_tracking: true,
            subscription_url: None,
            subscription_refresh_hours: default_subscription_refresh_hours(),
            last_subscription_refresh: 0,
        }
    }
}

/// Tracking protection manager
pub struct TrackingProtectionManager {
    rules: Arc<Mutex<HashMap<String, TrackerRule>>>,
    settings: Arc<Mutex<TrackingProtectionSettings>>,
    blocked_domains: Arc<Mutex<HashSet<String>>>,
    /// Per-site allow trackers (hostname -> true).
    site_overrides: Arc<Mutex<HashMap<String, bool>>>,
    /// Cosmetic element-hiding rules from subscribed filter lists.
    cosmetic_rules: Arc<Mutex<Vec<crate::filter_list_parser::CosmeticRule>>>,
    /// Cosmetic exceptions (`#@#`) — selectors not hidden on a host.
    cosmetic_exceptions: Arc<Mutex<Vec<crate::filter_list_parser::CosmeticRule>>>,
    storage_path: PathBuf,
}

impl TrackingProtectionManager {
    /// Create a new tracking protection manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(TrackingProtectionSettings::default())),
            blocked_domains: Arc::new(Mutex::new(HashSet::new())),
            site_overrides: Arc::new(Mutex::new(HashMap::new())),
            cosmetic_rules: Arc::new(Mutex::new(Vec::new())),
            cosmetic_exceptions: Arc::new(Mutex::new(Vec::new())),
            storage_path,
        };
        
        manager.load_default_rules()?;
        manager.load_subscribed_blocklist()?;
        manager.load_subscribed_cosmetic()?;
        manager.load_embedded_blocklist()?;
        manager.load_site_overrides()?;
        manager.load_from_disk()?;
        Ok(manager)
    }

    /// Load user-subscribed blocklist from disk (if present).
    fn load_subscribed_blocklist(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.storage_path.join("subscribed-blocklist.json");
        if !path.exists() {
            return Ok(());
        }
        self.merge_blocklist_file(&path)
    }

    /// Load cosmetic rules from `subscribed-cosmetic.json` (if present).
    fn load_subscribed_cosmetic(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.storage_path.join("subscribed-cosmetic.json");
        if !path.exists() {
            return Ok(());
        }
        #[derive(Deserialize)]
        struct CosmeticFile {
            cosmetics: Vec<crate::filter_list_parser::CosmeticRule>,
            #[serde(default)]
            exceptions: Vec<crate::filter_list_parser::CosmeticRule>,
        }
        let content = std::fs::read_to_string(&path)?;
        let file: CosmeticFile = serde_json::from_str(&content)?;
        let mut rules = self
            .cosmetic_rules
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        rules.clear();
        rules.extend(file.cosmetics);
        let mut exc = self
            .cosmetic_exceptions
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        exc.clear();
        exc.extend(file.exceptions);
        Ok(())
    }

    fn merge_blocklist_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct BlocklistEntry {
            domain: String,
            category: String,
        }
        #[derive(Deserialize)]
        struct BlocklistFile {
            domains: Vec<BlocklistEntry>,
        }
        let content = std::fs::read_to_string(path)?;
        let file: BlocklistFile = serde_json::from_str(&content)?;
        let mut rules = self.rules.lock().map_err(|e| format!("Lock error: {}", e))?;
        for entry in file.domains {
            let category = TrackerCategory::from_str(&entry.category);
            let rule = TrackerRule::new(entry.domain, category);
            rules.insert(rule.id.clone(), rule);
        }
        Ok(())
    }

    /// Refresh subscription when `subscription_refresh_hours` has elapsed.
    pub async fn refresh_subscription_if_due(&self) -> Result<Option<usize>, String> {
        let (url, hours, last) = {
            let s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            (
                s.subscription_url.clone(),
                s.subscription_refresh_hours,
                s.last_subscription_refresh,
            )
        };
        if hours == 0 {
            return Ok(None);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now.saturating_sub(last) < u64::from(hours) * 3600 {
            return Ok(None);
        }
        let count = self.refresh_subscribed_blocklist(url).await?;
        Ok(Some(count))
    }

    /// Update subscription URL and refresh interval (hours).
    pub fn set_subscription(
        &self,
        subscription_url: Option<String>,
        refresh_hours: Option<u32>,
    ) -> Result<(), String> {
        let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
        if let Some(u) = subscription_url {
            s.subscription_url = if u.trim().is_empty() {
                None
            } else {
                Some(u)
            };
        }
        if let Some(h) = refresh_hours {
            s.subscription_refresh_hours = h;
        }
        self.save_to_disk()
            .map_err(|e| format!("Save tracking settings: {}", e))
    }

    /// Refresh subscribed blocklist from optional remote URL or embedded fallback.
    pub async fn refresh_subscribed_blocklist(
        &self,
        url: Option<String>,
    ) -> Result<usize, String> {
        let fetch_url = url.or_else(|| {
            self.settings
                .lock()
                .ok()
                .and_then(|s| s.subscription_url.clone())
        });
        if let Some(fetch_url) = fetch_url.filter(|u| u.starts_with("http")) {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| format!("HTTP client: {}", e))?;
            if let Ok(response) = client.get(&fetch_url).send().await {
                if response.status().is_success() {
                    if let Ok(bytes) = response.bytes().await {
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            let path = self.storage_path.join("subscribed-blocklist.json");
                            let body = if text.contains("\"domains\"") {
                                text.to_string()
                            } else if crate::filter_list_parser::looks_like_abp_filter_list(&text) {
                                let domains =
                                    crate::filter_list_parser::parse_easylist_domains(&text);
                                let (cosmetics, cosmetic_exc) =
                                    crate::filter_list_parser::parse_easylist_cosmetic_pair(&text);
                                let cosmetic_path =
                                    self.storage_path.join("subscribed-cosmetic.json");
                                let cosmetic_body = crate::filter_list_parser::cosmetics_to_json(
                                    &cosmetics,
                                    &cosmetic_exc,
                                );
                                std::fs::write(&cosmetic_path, &cosmetic_body)
                                    .map_err(|e| format!("Write cosmetic list: {}", e))?;
                                self.load_subscribed_cosmetic()
                                    .map_err(|e| format!("Load cosmetic list: {}", e))?;
                                crate::filter_list_parser::domains_to_blocklist_json(&domains)
                            } else {
                                String::new()
                            };
                            if !body.is_empty() {
                                std::fs::write(&path, &body)
                                    .map_err(|e| format!("Write blocklist: {}", e))?;
                                self.merge_blocklist_file(&path)
                                    .map_err(|e| format!("Merge blocklist: {}", e))?;
                                self.touch_subscription_refresh()?;
                                return Ok(self.active_block_domains().len());
                            }
                        }
                    }
                }
            }
        }
        let count = self.refresh_from_embedded_fallback()?;
        self.touch_subscription_refresh()?;
        Ok(count)
    }

    fn touch_subscription_refresh(&self) -> Result<(), String> {
        let mut s = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
        s.last_subscription_refresh = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.save_to_disk()
            .map_err(|e| format!("Save tracking settings: {}", e))
    }

    fn refresh_from_embedded_fallback(&self) -> Result<usize, String> {
        let embedded = include_str!("../assets/tracker-blocklist.json");
        let path = self.storage_path.join("subscribed-blocklist.json");
        std::fs::write(&path, embedded).map_err(|e| format!("Write embedded blocklist: {}", e))?;
        self.merge_blocklist_file(&path)
            .map_err(|e| format!("Merge blocklist: {}", e))?;
        Ok(self.active_block_domains().len())
    }

    fn load_site_overrides(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.storage_path.join("site_shield_overrides.json");
        if !path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&path)?;
        let map: HashMap<String, bool> = serde_json::from_str(&content)?;
        *self.site_overrides.lock().map_err(|e| format!("Lock: {}", e))? = map;
        Ok(())
    }

    fn save_site_overrides(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.storage_path.join("site_shield_overrides.json");
        let map = self.site_overrides.lock().map_err(|e| format!("Lock: {}", e))?;
        std::fs::write(&path, serde_json::to_string_pretty(&*map)?)?;
        Ok(())
    }

    fn normalize_host(host: &str) -> String {
        host.trim().to_lowercase().trim_start_matches("www.").to_string()
    }

    /// Whether trackers are allowed on this top-level site.
    pub fn allows_trackers_on_site(&self, page_host: Option<&str>) -> bool {
        let Some(host) = page_host.map(Self::normalize_host) else {
            return false;
        };
        if host.is_empty() {
            return false;
        }
        self.site_overrides
            .lock()
            .unwrap_or_else(|_| panic!("Lock error"))
            .get(&host)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_site_tracker_allowance(
        &self,
        host: &str,
        allow_trackers: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::normalize_host(host);
        let mut map = self
            .site_overrides
            .lock()
            .map_err(|e| format!("Lock: {}", e))?;
        if allow_trackers {
            map.insert(key, true);
        } else {
            map.remove(&key);
        }
        drop(map);
        self.save_site_overrides()
    }

    pub fn get_site_tracker_allowance(&self, host: &str) -> bool {
        self.allows_trackers_on_site(Some(host))
    }

    /// Load domains from embedded JSON blocklist (Brave-style curated list).
    fn load_embedded_blocklist(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct BlocklistEntry {
            domain: String,
            category: String,
        }
        #[derive(Deserialize)]
        struct BlocklistFile {
            domains: Vec<BlocklistEntry>,
        }

        let raw = include_str!("../assets/tracker-blocklist.json");
        let file: BlocklistFile = serde_json::from_str(raw)
            .map_err(|e| format!("tracker-blocklist.json parse error: {}", e))?;

        let mut rules = self
            .rules
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        for entry in file.domains {
            let category = TrackerCategory::from_str(&entry.category);
            let rule = TrackerRule::new(entry.domain, category);
            rules.insert(rule.id.clone(), rule);
        }

        Ok(())
    }
    
    /// Load default tracker rules
    fn load_default_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_trackers = vec![
            // Advertising trackers
            ("doubleclick.net", TrackerCategory::Advertising),
            ("googleads.g.doubleclick.net", TrackerCategory::Advertising),
            ("googlesyndication.com", TrackerCategory::Advertising),
            ("advertising.com", TrackerCategory::Advertising),
            ("adsystem.com", TrackerCategory::Advertising),
            ("adnxs.com", TrackerCategory::Advertising),
            ("criteo.com", TrackerCategory::Advertising),
            ("outbrain.com", TrackerCategory::Advertising),
            ("taboola.com", TrackerCategory::Advertising),
            
            // Analytics trackers
            ("google-analytics.com", TrackerCategory::Analytics),
            ("analytics.google.com", TrackerCategory::Analytics),
            ("stats.g.doubleclick.net", TrackerCategory::Analytics),
            ("hotjar.com", TrackerCategory::Analytics),
            ("mixpanel.com", TrackerCategory::Analytics),
            ("segment.io", TrackerCategory::Analytics),
            ("amplitude.com", TrackerCategory::Analytics),
            
            // Social trackers
            ("facebook.com/tr", TrackerCategory::Social),
            ("connect.facebook.net", TrackerCategory::Social),
            ("platform.twitter.com", TrackerCategory::Social),
            ("static.ads-twitter.com", TrackerCategory::Social),
            ("analytics.twitter.com", TrackerCategory::Social),
            
            // Fingerprinting
            ("fingerprintjs.com", TrackerCategory::Fingerprinting),
            ("clientjs.org", TrackerCategory::Fingerprinting),
            
            // Cryptomining
            ("coinhive.com", TrackerCategory::Cryptomining),
            ("jsecoin.com", TrackerCategory::Cryptomining),
        ];
        
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for (domain, category) in default_trackers {
            let rule = TrackerRule::new(domain.to_string(), category);
            rules.insert(rule.id.clone(), rule);
        }
        
        Ok(())
    }
    
    /// Check if a URL should be blocked (optional top-level page host for per-site shields).
    pub fn should_block(&self, url: &str, page_host: Option<&str>) -> Option<TrackerRule> {
        if self.allows_trackers_on_site(page_host) {
            return None;
        }
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return None;
        }
        
        let rules = self.rules.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }
            
            let should_block = match rule.category {
                TrackerCategory::Advertising => settings.block_advertising,
                TrackerCategory::Analytics => settings.block_analytics,
                TrackerCategory::Social => settings.block_social,
                TrackerCategory::Fingerprinting => settings.block_fingerprinting,
                TrackerCategory::Cryptomining => settings.block_cryptomining,
                TrackerCategory::Tracking => settings.block_tracking,
            };
            
            if should_block && rule.matches(url) {
                return Some(rule.clone());
            }
        }
        
        None
    }
    
    /// Unique tracker domains currently eligible for blocking (enabled rules + settings).
    pub fn active_block_domains(&self) -> Vec<String> {
        let settings = self.settings.lock().unwrap_or_else(|_| panic!("Lock error"));
        if !settings.enabled {
            return Vec::new();
        }
        let rules = self.rules.lock().unwrap_or_else(|_| panic!("Lock error"));
        let mut domains: Vec<String> = rules
            .values()
            .filter(|rule| {
                if !rule.enabled {
                    return false;
                }
                match rule.category {
                    TrackerCategory::Advertising => settings.block_advertising,
                    TrackerCategory::Analytics => settings.block_analytics,
                    TrackerCategory::Social => settings.block_social,
                    TrackerCategory::Fingerprinting => settings.block_fingerprinting,
                    TrackerCategory::Cryptomining => settings.block_cryptomining,
                    TrackerCategory::Tracking => settings.block_tracking,
                }
            })
            .map(|r| r.domain.clone())
            .collect();
        domains.sort();
        domains.dedup();
        domains
    }

    /// Document-start script for a page host (empty when per-site shields allow trackers).
    pub fn init_script_for_page(&self, page_host: Option<&str>) -> String {
        if self.allows_trackers_on_site(page_host) {
            return String::new();
        }
        let tracker = self.init_script();
        let cosmetic = self.cosmetic_script_for_page(page_host);
        match (tracker.is_empty(), cosmetic.is_empty()) {
            (true, true) => String::new(),
            (false, true) => tracker,
            (true, false) => cosmetic,
            (false, false) => format!("{tracker}\n{cosmetic}"),
        }
    }

    /// Inject `display:none` for subscribed cosmetic selectors on the current page host.
    fn cosmetic_script_for_page(&self, page_host: Option<&str>) -> String {
        let host = match page_host {
            Some(h) if !h.is_empty() => h.to_lowercase(),
            _ => return String::new(),
        };
        let settings = self.settings.lock().unwrap_or_else(|_| panic!("Lock error"));
        if !settings.enabled {
            return String::new();
        }
        let rules = self
            .cosmetic_rules
            .lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let exceptions = self
            .cosmetic_exceptions
            .lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let host_matches = |r: &crate::filter_list_parser::CosmeticRule| {
            host == r.host || host.ends_with(&format!(".{}", r.host))
        };
        let rule_key = |r: &crate::filter_list_parser::CosmeticRule| {
            format!("{:?}:{}", r.kind, r.selector)
        };
        let exc_set: std::collections::HashSet<String> = exceptions
            .iter()
            .filter(|r| host_matches(r))
            .map(rule_key)
            .collect();
        let mut css_selectors: Vec<String> = Vec::new();
        let mut regex_patterns: Vec<String> = Vec::new();
        let mut procedural_selectors: Vec<String> = Vec::new();
        for r in rules
            .iter()
            .filter(|r| host_matches(r) && !exc_set.contains(&rule_key(r)))
        {
            if !crate::filter_list_parser::is_safe_cosmetic_selector(&r.selector, &r.kind) {
                continue;
            }
            match r.kind {
                crate::filter_list_parser::CosmeticSelectorKind::Regex => {
                    if regex_patterns.len() < 40 {
                        regex_patterns.push(r.selector.clone());
                    }
                }
                crate::filter_list_parser::CosmeticSelectorKind::Css => {
                    if css_selectors.len() < 300 {
                        css_selectors.push(r.selector.clone());
                    }
                }
                crate::filter_list_parser::CosmeticSelectorKind::Procedural => {
                    if procedural_selectors.len() < 60 {
                        procedural_selectors.push(r.selector.clone());
                    }
                }
            }
        }
        if css_selectors.is_empty() && regex_patterns.is_empty() && procedural_selectors.is_empty() {
            return String::new();
        }
        let css_json = serde_json::to_string(&css_selectors).unwrap_or_else(|_| "[]".to_string());
        let regex_json =
            serde_json::to_string(&regex_patterns).unwrap_or_else(|_| "[]".to_string());
        let proc_json = serde_json::to_string(&procedural_selectors)
            .unwrap_or_else(|_| "[]".to_string());
        format!(
            r#"(function() {{
  if (window.__exodusCosmetic) return;
  window.__exodusCosmetic = true;
  const sels = {css_json};
  const regexes = {regex_json};
  const abpHas = {proc_json};
  try {{
    if (sels.length) {{
      const css = sels.map(function(s) {{ return s + '{{display:none !important;}}'; }}).join('\n');
      const el = document.createElement('style');
      el.setAttribute('data-exodus-cosmetic', '1');
      el.textContent = css;
      (document.documentElement || document.head || document).appendChild(el);
    }}
    if (regexes.length) {{
      const nodes = document.querySelectorAll('div,section,article,aside,iframe,img,a,span,ins');
      regexes.forEach(function(pat) {{
        try {{
          var re = new RegExp(pat, 'i');
          nodes.forEach(function(el) {{
            var hay = (el.className || '') + ' ' + (el.id || '') + ' ' + (el.getAttribute('src') || '');
            if (re.test(hay)) el.style.setProperty('display', 'none', 'important');
          }});
        }} catch (_) {{}}
      }});
    }}
    if (abpHas.length) {{
      function hideAbpHas(innerSel) {{
        try {{
          document.querySelectorAll('*').forEach(function(el) {{
            if (el.querySelector && el.querySelector(innerSel)) {{
              el.style.setProperty('display', 'none', 'important');
            }}
          }});
        }} catch (_) {{}}
      }}
      abpHas.forEach(hideAbpHas);
      var procDebounce;
      var obs = new MutationObserver(function() {{
        clearTimeout(procDebounce);
        procDebounce = setTimeout(function() {{ abpHas.forEach(hideAbpHas); }}, 80);
      }});
      obs.observe(document.documentElement || document, {{ childList: true, subtree: true }});
    }}
  }} catch (_) {{}}
}})();"#
        )
    }

    /// Document-start script: patch fetch/XHR/beacon for known tracker hosts.
    pub fn init_script(&self) -> String {
        let domains = self.active_block_domains();
        if domains.is_empty() {
            return String::new();
        }
        let json = match serde_json::to_string(&domains) {
            Ok(j) => j,
            Err(_) => return String::new(),
        };
        format!(
            r#"(function() {{
  if (window.__exodusTrackerGuard) return;
  window.__exodusTrackerGuard = true;
  const blocked = {json};
  function hostBlocked(host) {{
    const h = (host || '').toLowerCase();
    return blocked.some(function(d) {{
      const d0 = d.toLowerCase();
      return h === d0 || h.endsWith('.' + d0) || h.includes(d0);
    }});
  }}
  window.__exodusTrackerBlocked = window.__exodusTrackerBlocked || [];
  function recordBlock(url) {{
    try {{
      var h = new URL(url, location.href).hostname;
      if (h) window.__exodusTrackerBlocked.push(h);
    }} catch (_) {{}}
  }}
  function urlBlocked(url) {{
    try {{ return hostBlocked(new URL(url, location.href).hostname); }}
    catch {{ return false; }}
  }}
  const origFetch = window.fetch;
  if (origFetch) {{
    window.fetch = function(input, init) {{
      const url = typeof input === 'string' ? input : (input && input.url) || '';
      if (urlBlocked(url)) {{ recordBlock(url); return Promise.reject(new Error('Exodus: tracker blocked')); }}
      return origFetch.apply(this, arguments);
    }};
  }}
  const XHR = window.XMLHttpRequest;
  if (XHR) {{
    const open = XHR.prototype.open;
    XHR.prototype.open = function(method, url) {{
      if (urlBlocked(String(url))) {{ recordBlock(String(url)); throw new Error('Exodus: tracker blocked'); }}
      return open.apply(this, arguments);
    }};
  }}
  const origBeacon = navigator.sendBeacon;
  if (origBeacon) {{
    navigator.sendBeacon = function(url, data) {{
      if (urlBlocked(String(url))) {{ recordBlock(String(url)); return false; }}
      return origBeacon.apply(this, arguments);
    }};
  }}
}})();"#
        )
    }

    /// Block a URL and record the block
    pub fn block_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(rule) = self.should_block(url, None) {
            let mut rules = self.rules.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(rule) = rules.get_mut(&rule.id) {
                rule.record_block();
            }
            
            let mut blocked = self.blocked_domains.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(domain) = Self::extract_domain(url) {
                blocked.insert(domain);
            }
            
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return Some(host.to_string());
            }
        }
        None
    }
    
    /// Get all tracker rules
    pub fn get_rules(&self) -> Vec<TrackerRule> {
        let rules = self.rules.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        rules.values().cloned().collect()
    }
    
    /// Add a custom tracker rule
    pub fn add_rule(&self, rule: TrackerRule) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        rules.insert(rule.id.clone(), rule);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a tracker rule
    pub fn remove_rule(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        rules.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get tracking protection settings
    pub fn get_settings(&self) -> TrackingProtectionSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update tracking protection settings
    pub fn update_settings(&self, settings: TrackingProtectionSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get blocked domains
    pub fn get_blocked_domains(&self) -> HashSet<String> {
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        blocked.clone()
    }
    
    /// Clear blocked domains
    pub fn clear_blocked_domains(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        blocked.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get blocking statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let rules = self.rules.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        for rule in rules.values() {
            stats.insert(rule.category.as_str().to_string(), rule.block_count);
        }
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("tracking_settings.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: TrackingProtectionSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("tracking_settings.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Check if a URL should be blocked
#[tauri::command]
pub fn should_block_url(
    url: String,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<bool, String> {
    Ok(manager.should_block(&url, None).is_some())
}

/// Per-site shields: allow trackers on a hostname.
#[tauri::command]
pub fn set_site_shield_override(
    host: String,
    allow_trackers: bool,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager
        .set_site_tracker_allowance(&host, allow_trackers)
        .map_err(|e| format!("set_site_shield_override: {}", e))
}

/// Read per-site tracker allowance for a hostname.
#[tauri::command]
pub fn get_site_shield_override(
    host: String,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<bool, String> {
    Ok(manager.get_site_tracker_allowance(&host))
}

/// Refresh tracker blocklist subscription (remote URL or embedded fallback).
#[tauri::command]
pub async fn refresh_tracker_blocklist(
    url: Option<String>,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<usize, String> {
    manager.refresh_subscribed_blocklist(url).await
}

/// Configure blocklist subscription URL and auto-refresh interval (hours).
#[tauri::command]
pub fn set_tracking_subscription(
    subscription_url: Option<String>,
    refresh_hours: Option<u32>,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.set_subscription(subscription_url, refresh_hours)
}

/// Run subscription refresh when the configured interval has elapsed.
#[tauri::command]
pub async fn run_tracking_subscription_refresh_if_due(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<Option<usize>, String> {
    manager.refresh_subscription_if_due().await
}

/// Block a URL
#[tauri::command]
pub fn block_url(
    url: String,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.block_url(&url)
        .map_err(|e| format!("Failed to block URL: {}", e))
}

/// Get all tracker rules
#[tauri::command]
pub fn get_tracker_rules(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<Vec<TrackerRule>, String> {
    Ok(manager.get_rules())
}

/// Add a custom tracker rule
#[tauri::command]
pub fn add_tracker_rule(
    rule: TrackerRule,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.add_rule(rule)
        .map_err(|e| format!("Failed to add rule: {}", e))
}

/// Remove a tracker rule
#[tauri::command]
pub fn remove_tracker_rule(
    id: String,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.remove_rule(&id)
        .map_err(|e| format!("Failed to remove rule: {}", e))
}

/// Get tracking protection settings
#[tauri::command]
pub fn get_tracking_settings(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<TrackingProtectionSettings, String> {
    Ok(manager.get_settings())
}

/// Update tracking protection settings
#[tauri::command]
pub fn update_tracking_settings(
    settings: TrackingProtectionSettings,
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get blocked domains
#[tauri::command]
pub fn get_blocked_domains(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_blocked_domains().into_iter().collect())
}

/// Clear blocked domains
#[tauri::command]
pub fn clear_blocked_domains(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<(), String> {
    manager.clear_blocked_domains()
        .map_err(|e| format!("Failed to clear blocked domains: {}", e))
}

/// Get blocking statistics
#[tauri::command]
pub fn get_blocking_stats(
    manager: State<'_, Arc<TrackingProtectionManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_tracker_rule_creation() {
        let rule = TrackerRule::new(
            "example.com".to_string(),
            TrackerCategory::Advertising,
        );
        
        assert_eq!(rule.domain, "example.com");
        assert_eq!(rule.category, TrackerCategory::Advertising);
        assert!(rule.enabled);
        assert_eq!(rule.block_count, 0);
    }
    
    #[test]
    fn test_tracker_rule_matching() {
        let rule = TrackerRule::new(
            "example.com".to_string(),
            TrackerCategory::Advertising,
        );
        
        assert!(rule.matches("https://example.com"));
        assert!(rule.matches("https://sub.example.com"));
        assert!(!rule.matches("https://other.com"));
    }
    
    #[test]
    fn test_site_shield_allows_trackers() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TrackingProtectionManager::new(temp_dir.path().to_path_buf()).expect("Failed to create TrackingProtectionManager");
        manager
            .set_site_tracker_allowance("news.example.com", true)
            .expect("Failed to set site tracker allowance");
        assert!(manager.allows_trackers_on_site(Some("news.example.com")));
        assert!(manager
            .should_block("https://google-analytics.com/collect", Some("news.example.com"))
            .is_none());
        assert!(manager
            .should_block("https://google-analytics.com/collect", Some("other.com"))
            .is_some());
    }

    #[test]
    fn test_tracking_protection_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TrackingProtectionManager::new(temp_dir.path().to_path_buf()).expect("Failed to create TrackingProtectionManager");
        
        let should_block = manager.should_block("https://google-analytics.com/test", None);
        assert!(should_block.is_some());
        
        let should_block = manager.should_block("https://example.com", None);
        assert!(should_block.is_none());
    }
    
    #[test]
    fn test_tracking_settings() {
        let settings = TrackingProtectionSettings::default();
        assert!(settings.enabled);
        assert!(settings.block_advertising);
        assert!(!settings.block_social);
    }

    #[test]
    fn init_script_contains_tracker_domain() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TrackingProtectionManager::new(temp_dir.path().to_path_buf()).expect("Failed to create TrackingProtectionManager");
        let script = manager.init_script();
        assert!(script.contains("google-analytics.com"));
        assert!(script.contains("__exodusTrackerGuard"));
    }
}
