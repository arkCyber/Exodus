//! Exodus Browser — in-page password autofill and capture hooks.

/// Build fill script for eval in a content webview.
#[tauri::command]
pub fn password_build_fill_script(username: String, password: String) -> String {
    password_fill_script(&username, &password)
}

/// Document-start script: listen for login form submits and queue credentials for the host UI.
pub fn password_capture_init_script() -> &'static str {
    r#"(function() {
  if (window.__exodusPasswordCaptureInstalled) return;
  window.__exodusPasswordCaptureInstalled = true;
  function findUserField(form) {
    var u = form.querySelector('input[type="email"], input[autocomplete="username"], input[name*="user" i], input[name*="login" i], input[id*="user" i], input[id*="email" i]');
    if (u) return u;
    var text = form.querySelector('input[type="text"]');
    return text;
  }
  function findPassField(form) {
    return form.querySelector('input[type="password"]');
  }
  function captureFromForm(form) {
    var pass = findPassField(form);
    if (!pass || !pass.value) return;
    var user = findUserField(form);
    var username = user && user.value ? user.value : '';
    window.__exodusPasswordCapture = {
      url: location.href,
      username: username,
      password: pass.value
    };
  }
  document.addEventListener('submit', function(e) {
    var form = e.target;
    if (form && form.tagName === 'FORM') captureFromForm(form);
  }, true);
  document.addEventListener('click', function(e) {
    var t = e.target;
    if (t && (t.type === 'submit' || (t.tagName === 'BUTTON' && t.type !== 'button'))) {
      var form = t.closest && t.closest('form');
      if (form) setTimeout(function() { captureFromForm(form); }, 0);
    }
  }, true);
})();"#
}

/// Fill username/password fields on the current document.
pub fn password_fill_script(username: &str, password: &str) -> String {
    let user_json = serde_json::to_string(username).unwrap_or_else(|_| "\"\"".to_string());
    let pass_json = serde_json::to_string(password).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"(function() {{
  var user = {user_json};
  var pass = {pass_json};
  function findUserField(root) {{
    return root.querySelector('input[type="email"], input[autocomplete="username"], input[name*="user" i], input[name*="login" i], input[id*="user" i], input[id*="email" i]') ||
      root.querySelector('input[type="text"]');
  }}
  function findPassField(root) {{
    return root.querySelector('input[type="password"]');
  }}
  var forms = document.querySelectorAll('form');
  var filled = false;
  for (var i = 0; i < forms.length; i++) {{
    var f = forms[i];
    var p = findPassField(f);
    if (!p) continue;
    var u = findUserField(f);
    if (u && user) {{ u.value = user; u.dispatchEvent(new Event('input', {{ bubbles: true }})); }}
    if (pass) {{ p.value = pass; p.dispatchEvent(new Event('input', {{ bubbles: true }})); }}
    filled = true;
  }}
  if (!filled) {{
    var p = findPassField(document);
    var u = findUserField(document);
    if (p && pass) {{ p.value = pass; p.dispatchEvent(new Event('input', {{ bubbles: true }})); }}
    if (u && user) {{ u.value = user; u.dispatchEvent(new Event('input', {{ bubbles: true }})); }}
  }}
}})();"#
    )
}
