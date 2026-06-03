//! EasyList / Adblock Plus filter list parser (domain + cosmetic extraction MVP).

use serde::{Deserialize, Serialize};

/// One blocked domain extracted from a filter list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDomainEntry {
    pub domain: String,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "tracking".to_string()
}

/// Detect whether text looks like an ABP / EasyList filter list.
pub fn looks_like_abp_filter_list(text: &str) -> bool {
    let sample: String = text.chars().take(4096).collect();
    sample.contains("||")
        || sample.contains("[Adblock")
        || sample.contains("! Checksum:")
        || sample.lines().any(|l| l.starts_with("||") && l.contains('^'))
}

/// Parse EasyList/ABP lines into unique domain entries (`||domain^` rules).
pub fn parse_easylist_domains(text: &str) -> Vec<FilterDomainEntry> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('!')
            || line.starts_with('[')
            || line.starts_with("@@")
        {
            continue;
        }
        if line.starts_with("@@") {
            continue;
        }
        if line.starts_with('/') && line.ends_with('/') {
            continue;
        }
        if let Some(domain) = parse_abp_line(line) {
            let d = domain.to_lowercase();
            if d.len() > 2 && seen.insert(d.clone()) {
                out.push(FilterDomainEntry {
                    domain: d,
                    category: "tracking".to_string(),
                });
            }
        }
    }
    out
}

fn parse_abp_line(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix("||") {
        let host = rest
            .split('^')
            .next()
            .unwrap_or(rest)
            .split('/')
            .next()
            .unwrap_or(rest)
            .split('$')
            .next()
            .unwrap_or(rest)
            .trim();
        if host.contains('*') || host.is_empty() {
            return None;
        }
        return Some(host.to_string());
    }
    if line.starts_with('|') && line.ends_with('|') && line.len() > 2 {
        let inner = &line[1..line.len() - 1];
        if let Ok(url) = url::Url::parse(inner) {
            return url.host_str().map(|h| h.to_string());
        }
    }
    None
}

/// Cosmetic selector kind (`css`, `/regex/`, or procedural `:-abp-has`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CosmeticSelectorKind {
    #[default]
    Css,
    Regex,
    Procedural,
}

/// Cosmetic hide rule (`example.com##.ad-banner` or `example.com##/banner/i`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CosmeticRule {
    pub host: String,
    pub selector: String,
    #[serde(default)]
    pub kind: CosmeticSelectorKind,
}

/// Reject selectors that could break injection or enable script execution.
pub fn is_safe_cosmetic_selector(selector: &str, kind: &CosmeticSelectorKind) -> bool {
    let s = selector.trim();
    if s.is_empty() || s.len() > 512 {
        return false;
    }
    let lower = s.to_lowercase();
    for bad in [
        "<script",
        "</script",
        "javascript:",
        "expression(",
        "@import",
        "behavior:",
        "-moz-binding",
        "url(",
        "data:",
    ] {
        if lower.contains(bad) {
            return false;
        }
    }
    match kind {
        CosmeticSelectorKind::Regex => {
            if s.len() > 120 {
                return false;
            }
            let re_meta = s.chars().filter(|c| *c == '(' || *c == ')' || *c == '+').count();
            re_meta <= 24
        }
        CosmeticSelectorKind::Procedural => {
            !s.contains('{') && !s.contains('}') && !s.contains(';')
        }
        CosmeticSelectorKind::Css => {
            !s.contains('{') && !s.contains('}') && !s.contains(';') && !s.contains('<')
        }
    }
}

/// Parse `:-abp-has(.inner)` or `:has(.inner)` procedural selector body.
pub fn parse_procedural_selector(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if let Some(inner) = raw.strip_prefix(":-abp-has(").and_then(|s| s.strip_suffix(')')) {
        let inner = inner.trim();
        if !inner.is_empty() {
            return Some(inner.to_string());
        }
    }
    if let Some(inner) = raw.strip_prefix(":has(").and_then(|s| s.strip_suffix(')')) {
        let inner = inner.trim();
        if !inner.is_empty() {
            return Some(inner.to_string());
        }
    }
    None
}

fn classify_cosmetic_selector(raw: &str) -> (CosmeticSelectorKind, String) {
    let raw = raw.trim();
    if let Some(inner) = parse_procedural_selector(raw) {
        return (CosmeticSelectorKind::Procedural, inner);
    }
    if raw.starts_with('/') && raw.ends_with('/') && raw.len() > 2 && !raw.contains('[') {
        return (
            CosmeticSelectorKind::Regex,
            raw[1..raw.len() - 1].to_string(),
        );
    }
    (CosmeticSelectorKind::Css, raw.to_string())
}

/// Parse ABP cosmetic hide rules (`domain##selector`).
pub fn parse_easylist_cosmetic(text: &str) -> Vec<CosmeticRule> {
    parse_easylist_cosmetic_pair(text).0
}

/// Parse hide + exception (`domain#@#selector`) cosmetic rules.
pub fn parse_easylist_cosmetic_pair(text: &str) -> (Vec<CosmeticRule>, Vec<CosmeticRule>) {
    let mut seen_hide = std::collections::HashSet::new();
    let mut seen_exc = std::collections::HashSet::new();
    let mut hide = Vec::new();
    let mut exceptions = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('!') {
            continue;
        }
        if let Some((host, selector)) = line.split_once("#@#?#") {
            let host = host.trim().to_lowercase();
            let selector = selector.trim();
            if host.is_empty() || selector.is_empty() {
                continue;
            }
            let (kind, selector) = classify_cosmetic_selector(selector);
            if !is_safe_cosmetic_selector(&selector, &kind) {
                continue;
            }
            let key = format!("{host}\0{:?}\0{selector}", kind);
            if seen_exc.insert(key) {
                exceptions.push(CosmeticRule {
                    host,
                    selector,
                    kind,
                });
            }
            continue;
        }
        if let Some((host, selector)) = line.split_once("#?#") {
            let host = host.trim().to_lowercase();
            let selector = selector.trim();
            if host.is_empty() || selector.is_empty() {
                continue;
            }
            let (kind, selector) = classify_cosmetic_selector(selector);
            if !is_safe_cosmetic_selector(&selector, &kind) {
                continue;
            }
            let key = format!("{host}\0{:?}\0{selector}", kind);
            if seen_hide.insert(key) {
                hide.push(CosmeticRule {
                    host,
                    selector,
                    kind,
                });
            }
            continue;
        }
        if let Some((host, selector)) = line.split_once("#@#") {
            let host = host.trim().to_lowercase();
            let selector = selector.trim();
            if host.is_empty() || selector.is_empty() {
                continue;
            }
            let (kind, selector) = classify_cosmetic_selector(selector);
            if !is_safe_cosmetic_selector(&selector, &kind) {
                continue;
            }
            let key = format!("{host}\0{:?}\0{selector}", kind);
            if seen_exc.insert(key) {
                exceptions.push(CosmeticRule {
                    host,
                    selector,
                    kind,
                });
            }
            continue;
        }
        if let Some((host, selector)) = line.split_once("##") {
            let host = host.trim().to_lowercase();
            let selector = selector.trim();
            if host.is_empty() || selector.is_empty() || selector.contains("##") {
                continue;
            }
            let (kind, selector) = classify_cosmetic_selector(selector);
            if !is_safe_cosmetic_selector(&selector, &kind) {
                continue;
            }
            let key = format!("{host}\0{:?}\0{selector}", kind);
            if seen_hide.insert(key) {
                hide.push(CosmeticRule {
                    host,
                    selector,
                    kind,
                });
            }
        }
    }
    (hide, exceptions)
}

/// Build `subscribed-cosmetic.json` body from parsed cosmetic rules.
pub fn cosmetics_to_json(hide: &[CosmeticRule], exceptions: &[CosmeticRule]) -> String {
    #[derive(Serialize)]
    struct CosmeticFile {
        cosmetics: Vec<CosmeticRule>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        exceptions: Vec<CosmeticRule>,
    }
    serde_json::to_string_pretty(&CosmeticFile {
        cosmetics: hide.to_vec(),
        exceptions: exceptions.to_vec(),
    })
    .unwrap_or_else(|_| r#"{"cosmetics":[]}"#.to_string())
}

/// Build `subscribed-blocklist.json` body from parsed domains.
pub fn domains_to_blocklist_json(domains: &[FilterDomainEntry]) -> String {
    #[derive(Serialize)]
    struct BlocklistFile {
        domains: Vec<FilterDomainEntry>,
    }
    serde_json::to_string_pretty(&BlocklistFile {
        domains: domains.to_vec(),
    })
    .unwrap_or_else(|_| r#"{"domains":[]}"#.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_easylist_domain_rule() {
        let text = "! EasyList\n||tracker.evil^\n||ads.example.org^$third-party\n";
        let list = parse_easylist_domains(text);
        assert!(list.iter().any(|e| e.domain == "tracker.evil"));
        assert!(list.iter().any(|e| e.domain == "ads.example.org"));
    }

    #[test]
    fn detects_abp_format() {
        assert!(looks_like_abp_filter_list("||foo.com^\n"));
        assert!(!looks_like_abp_filter_list(r#"{"domains":[]}"#));
    }

    #[test]
    fn parses_cosmetic_rule() {
        let text = "example.com##.ad\nfoo.org##div.banner\n";
        let rules = parse_easylist_cosmetic(text);
        assert!(rules.contains(&CosmeticRule {
            host: "example.com".to_string(),
            selector: ".ad".to_string(),
            kind: CosmeticSelectorKind::Css,
        }));
    }

    #[test]
    fn parses_cosmetic_exception() {
        let text = "example.com##.ad\nexample.com#@#.ad\n";
        let (hide, exc) = parse_easylist_cosmetic_pair(text);
        assert_eq!(hide.len(), 1);
        assert_eq!(exc.len(), 1);
        assert_eq!(exc[0].selector, ".ad");
    }

    #[test]
    fn parses_cosmetic_regex() {
        let text = "example.com##/banner-ad/\n";
        let rules = parse_easylist_cosmetic(text);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].kind, CosmeticSelectorKind::Regex);
        assert_eq!(rules[0].selector, "banner-ad");
    }

    #[test]
    fn parses_procedural_abp_has() {
        let text = "example.com#?#:-abp-has(.ad-banner)\n";
        let rules = parse_easylist_cosmetic(text);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].kind, CosmeticSelectorKind::Procedural);
        assert_eq!(rules[0].selector, ".ad-banner");
    }

    #[test]
    fn rejects_unsafe_cosmetic_selector() {
        assert!(!is_safe_cosmetic_selector(
            "javascript:alert(1)",
            &CosmeticSelectorKind::Css
        ));
        assert!(is_safe_cosmetic_selector(".ad-banner", &CosmeticSelectorKind::Css));
    }

    #[test]
    fn parses_procedural_exception() {
        let text = "example.com#?#:-abp-has(.ad)\nexample.com#@#?#:-abp-has(.ad)\n";
        let (hide, exc) = parse_easylist_cosmetic_pair(text);
        assert_eq!(hide.len(), 1);
        assert_eq!(exc.len(), 1);
        assert_eq!(exc[0].kind, CosmeticSelectorKind::Procedural);
    }
}
