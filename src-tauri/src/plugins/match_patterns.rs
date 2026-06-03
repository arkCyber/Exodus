//! Exodus Browser — Chrome extension match pattern matching for content scripts.

use url::Url;

/// Returns true when `url` matches a Chrome-style match pattern.
pub fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    if pattern == "<all_urls>" {
        return matches_scheme(url, &["http", "https"]);
    }
    if let Ok(parsed) = Url::parse(url) {
        return match_pattern_url(&parsed, pattern);
    }
    false
}

/// Returns true when `url` matches any pattern and no exclude pattern.
pub fn url_matches_content_script(
    url: &str,
    matches: &[String],
    exclude_matches: &[String],
) -> bool {
    if matches.is_empty() {
        return false;
    }
    if exclude_matches
        .iter()
        .any(|p| url_matches_pattern(url, p))
    {
        return false;
    }
    matches.iter().any(|p| url_matches_pattern(url, p))
}

fn matches_scheme(url: &str, schemes: &[&str]) -> bool {
    Url::parse(url)
        .ok()
        .map(|u| schemes.contains(&u.scheme()))
        .unwrap_or(false)
}

fn match_pattern_url(url: &Url, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split("://").collect();
    if parts.len() != 2 {
        return false;
    }
    let scheme_part = parts[0];
    let rest = parts[1];

    if !scheme_matches(url.scheme(), scheme_part) {
        return false;
    }

    let (host_pattern, path_pattern) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };

    let host = url.host_str().unwrap_or("");
    if !host_matches(host, host_pattern) {
        return false;
    }

    path_matches(url.path(), path_pattern)
}

fn scheme_matches(scheme: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return scheme == "http" || scheme == "https";
    }
    scheme == pattern
}

fn host_matches(host: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return !host.is_empty();
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }
    host == pattern
}

fn path_matches(path: &str, pattern: &str) -> bool {
    if pattern == "/*" || pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return path.starts_with(prefix.trim_end_matches('/'))
            || format!("{path}/").starts_with(prefix);
    }
    path == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_urls_matches_http_https() {
        assert!(url_matches_pattern("https://example.com/a", "<all_urls>"));
        assert!(url_matches_pattern("http://example.com/", "<all_urls>"));
        assert!(!url_matches_pattern("file:///tmp", "<all_urls>"));
    }

    #[test]
    fn host_wildcard() {
        assert!(url_matches_pattern(
            "https://www.google.com/search",
            "*://*.google.com/*"
        ));
        assert!(!url_matches_pattern(
            "https://example.com/",
            "*://*.google.com/*"
        ));
    }

    #[test]
    fn exclude_matches() {
        assert!(!url_matches_content_script(
            "https://example.com/",
            &["<all_urls>".into()],
            &["*://example.com/*".into()],
        ));
    }
}
