//! Exodus Browser — declarativeNetRequest MVP (URL filter block/redirect at navigation).

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Rule action for MVP dynamic rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetRuleAction {
    Block,
    Redirect,
}

/// Parsed dynamic network rule (subset of Chrome DNR).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetRule {
    pub extension_id: String,
    pub id: String,
    pub url_filter: String,
    pub action: NetRuleAction,
    #[serde(default)]
    pub redirect_url: Option<String>,
}

/// Result when navigation is blocked or redirected.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetNavigationDecision {
    pub blocked: bool,
    pub redirect_url: Option<String>,
    pub rule_id: String,
    pub extension_id: String,
}

/// Thread-safe store of extension network rules.
pub struct NetRuleStore {
    rules: Mutex<Vec<NetRule>>,
}

impl NetRuleStore {
    /// Create an empty rule store.
    pub fn new() -> Self {
        Self {
            rules: Mutex::new(Vec::new()),
        }
    }

    /// Replace rules for an extension (after DNR flush).
    #[allow(dead_code)]
    pub fn set_rules_for_extension(&self, extension_id: &str, rules: Vec<NetRule>) -> Result<(), String> {
        let mut guard = self
            .rules
            .lock()
            .map_err(|e| format!("Net rule store lock error: {e}"))?;
        guard.retain(|r| r.extension_id != extension_id);
        guard.extend(rules);
        Ok(())
    }

    /// Apply add/remove from a DNR update payload.
    pub fn apply_update(
        &self,
        extension_id: &str,
        add_rules: &[serde_json::Value],
        remove_rule_ids: &[String],
    ) -> Result<(), String> {
        let mut guard = self
            .rules
            .lock()
            .map_err(|e| format!("Net rule store lock error: {e}"))?;
        if !remove_rule_ids.is_empty() {
            guard.retain(|r| {
                r.extension_id != extension_id || !remove_rule_ids.contains(&r.id)
            });
        }
        for raw in add_rules {
            if let Some(rule) = parse_dnr_rule(extension_id, raw) {
                guard.retain(|r| !(r.extension_id == extension_id && r.id == rule.id));
                guard.push(rule);
            }
        }
        Ok(())
    }

    /// Evaluate navigation URL against all rules (first match wins).
    pub fn evaluate_navigation(&self, url: &str) -> Option<NetNavigationDecision> {
        let guard = self.rules.lock().ok()?;
        for rule in guard.iter() {
            if url_matches_filter(url, &rule.url_filter) {
                return Some(match rule.action {
                    NetRuleAction::Block => NetNavigationDecision {
                        blocked: true,
                        redirect_url: None,
                        rule_id: rule.id.clone(),
                        extension_id: rule.extension_id.clone(),
                    },
                    NetRuleAction::Redirect => NetNavigationDecision {
                        blocked: false,
                        redirect_url: rule.redirect_url.clone(),
                        rule_id: rule.id.clone(),
                        extension_id: rule.extension_id.clone(),
                    },
                });
            }
        }
        None
    }
}

impl Default for NetRuleStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Chrome-style `urlFilter` rule from JSON (MVP subset).
fn parse_dnr_rule(extension_id: &str, raw: &serde_json::Value) -> Option<NetRule> {
    let id = raw.get("id")?.as_i64()?.to_string();
    let condition = raw.get("condition")?;
    let url_filter = condition.get("urlFilter")?.as_str()?.to_string();
    let action_type = raw.get("action")?.get("type")?.as_str()?;
    let (action, redirect_url) = match action_type {
        "block" => (NetRuleAction::Block, None),
        "redirect" => {
            let redirect = raw
                .get("action")?
                .get("redirect")?
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (NetRuleAction::Redirect, redirect)
        }
        _ => return None,
    };
    Some(NetRule {
        extension_id: extension_id.to_string(),
        id,
        url_filter,
        action,
        redirect_url,
    })
}

/// Simple `*` wildcard URL filter match (MVP).
pub fn url_matches_filter(url: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }
    if filter == "*" || filter == "<all_urls>" {
        return true;
    }
    if !filter.contains('*') {
        return url.contains(filter);
    }
    let parts: Vec<&str> = filter.split('*').collect();
    if parts.is_empty() {
        return true;
    }
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = url[pos..].find(part) {
            if i == 0 && !filter.starts_with('*') && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }
    if !filter.ends_with('*') {
        return url.ends_with(parts.last().copied().unwrap_or(""));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_filter_wildcard() {
        assert!(url_matches_filter("https://ads.example.com/x", "*ads*"));
        assert!(!url_matches_filter("https://example.com/", "*ads*"));
    }

    #[test]
    fn block_rule_applies() {
        let store = NetRuleStore::new();
        store
            .apply_update(
                "ext",
                &[serde_json::json!({
                    "id": 1,
                    "priority": 1,
                    "action": { "type": "block" },
                    "condition": { "urlFilter": "*tracker*" }
                })],
                &[],
            )
            .expect("apply");
        let d = store
            .evaluate_navigation("https://tracker.evil/ping")
            .expect("match");
        assert!(d.blocked);
    }
}
