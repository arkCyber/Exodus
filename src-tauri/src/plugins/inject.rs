//! Exodus Browser — content script injection bundle builder.

use super::manifest::RunAt;
use super::manager::ResolvedContentScript;

/// Build a single JavaScript bundle for injection at document start.
pub fn build_document_start_bundle(scripts: &[ResolvedContentScript], page_url: &str) -> String {
    build_bundle_for_run_at(scripts, page_url, RunAt::DocumentStart)
}

/// Build injection script for document_end / document_idle (eval after load).
pub fn build_document_end_bundle(scripts: &[ResolvedContentScript], page_url: &str) -> String {
    let parts = vec![
        build_bundle_for_run_at(scripts, page_url, RunAt::DocumentEnd),
        build_bundle_for_run_at(scripts, page_url, RunAt::DocumentIdle),
    ];
    parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n")
}

fn build_bundle_for_run_at(
    scripts: &[ResolvedContentScript],
    page_url: &str,
    run_at: RunAt,
) -> String {
    let mut blocks = Vec::new();
    for script in scripts {
        if script.run_at != run_at {
            continue;
        }
        if !script.matches_url(page_url) {
            continue;
        }
        blocks.push(wrap_content_script(
            &script.extension_id,
            &script.js_bodies,
            &script.css_bodies,
            script.all_frames,
        ));
    }
    blocks.join("\n")
}

/// Escape CSS for embedding inside a JS string literal.
fn escape_css_for_js(css: &str) -> String {
    css.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Build inner script body (CSS + JS) for one extension in a document.
fn build_inner_script_body(
    extension_id: &str,
    js_bodies: &[String],
    css_bodies: &[String],
) -> String {
    let escaped_id = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    let mut joined = String::new();
    for css in css_bodies {
        if css.trim().is_empty() {
            continue;
        }
        let escaped_css = escape_css_for_js(css);
        joined.push_str(&format!(
            r#"    var s = document.createElement('style');
    s.setAttribute('data-exodus-extension', '{escaped_id}');
    s.textContent = '{escaped_css}';
    (doc.head || doc.documentElement).appendChild(s);
"#
        ));
    }
    for body in js_bodies {
        joined.push_str("    ");
        joined.push_str(body);
        joined.push('\n');
    }
    joined
}

fn wrap_content_script(
    extension_id: &str,
    js_bodies: &[String],
    css_bodies: &[String],
    all_frames: bool,
) -> String {
    let escaped_id = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    let inner = build_inner_script_body(extension_id, js_bodies, css_bodies);
    let marker_key = format!("__exodusContentScripts_{escaped_id}");

    if all_frames {
        format!(
            r#"(function() {{
  var extId = '{escaped_id}';
  var markerKey = '{marker_key}';
  function injectIntoDocument(doc) {{
    if (!doc || doc[markerKey]) return;
    doc[markerKey] = true;
    try {{
      var docRef = doc;
{inner}
    }} catch (e) {{
      console.error('[Exodus extension ' + extId + ']', e);
    }}
  }}
  function injectAllFrames() {{
    injectIntoDocument(document);
    var iframes = document.querySelectorAll('iframe');
    for (var i = 0; i < iframes.length; i++) {{
      try {{
        var frameDoc = iframes[i].contentDocument;
        if (frameDoc) injectIntoDocument(frameDoc);
      }} catch (e) {{}}
    }}
  }}
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.runtime) window.chrome.runtime = {{}};
  window.chrome.runtime.id = extId;
  injectAllFrames();
  if (document.documentElement) {{
    new MutationObserver(injectAllFrames).observe(document.documentElement, {{
      childList: true,
      subtree: true
    }});
  }}
}})();"#
        )
    } else {
        format!(
            r#"(function() {{
  if (window.__exodusContentScripts && window.__exodusContentScripts['{escaped_id}']) return;
  window.__exodusContentScripts = window.__exodusContentScripts || {{}};
  window.__exodusContentScripts['{escaped_id}'] = true;
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.runtime) window.chrome.runtime = {{}};
  window.chrome.runtime.id = '{escaped_id}';
  try {{
    var doc = document;
{inner}
  }} catch (e) {{
    console.error('[Exodus extension {escaped_id}]', e);
  }}
}})();"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::manifest::RunAt;

    #[test]
    fn bundle_includes_extension_marker() {
        let scripts = vec![ResolvedContentScript {
            extension_id: "hello".into(),
            matches: vec!["<all_urls>".into()],
            exclude_matches: vec![],
            js_bodies: vec!["document.title = 'x';".into()],
            css_bodies: vec![],
            all_frames: false,
            run_at: RunAt::DocumentStart,
        }];
        let bundle = build_document_start_bundle(&scripts, "https://example.com/");
        assert!(bundle.contains("__exodusContentScripts"));
        assert!(bundle.contains("hello"));
    }

    #[test]
    fn all_frames_bundle_uses_mutation_observer() {
        let scripts = vec![ResolvedContentScript {
            extension_id: "frames".into(),
            matches: vec!["<all_urls>".into()],
            exclude_matches: vec![],
            js_bodies: vec!["window.__inFrame = true;".into()],
            css_bodies: vec![],
            all_frames: true,
            run_at: RunAt::DocumentStart,
        }];
        let bundle = build_document_start_bundle(&scripts, "https://example.com/");
        assert!(bundle.contains("MutationObserver"));
        assert!(bundle.contains("contentDocument"));
    }
}
