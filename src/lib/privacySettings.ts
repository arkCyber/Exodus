/**
 * Exodus Browser — privacy settings helpers (mirrors Tauri get_privacy_settings tuple).
 */

/** Privacy flags from backend config. */
export type PrivacySettings = {
  httpsOnly: boolean;
  privateMode: boolean;
  blockPopups: boolean;
  sessionRestore: boolean;
};

/** Parse `(https_only, private_mode, block_popups, session_restore)` from `get_privacy_settings`. */
export function parsePrivacyTuple(
  tuple: [boolean, boolean, boolean, boolean],
): PrivacySettings {
  return {
    httpsOnly: tuple[0],
    privateMode: tuple[1],
    blockPopups: tuple[2],
    sessionRestore: tuple[3],
  };
}

/**
 * Upgrade http:// navigations to https:// when HTTPS-only mode is enabled.
 * Mirrors backend `webview_url_from_str` for iframe / omnibox paths.
 */
export function applyHttpsOnly(url: string, httpsOnly: boolean): string {
  if (!httpsOnly || !url.startsWith('http://')) return url;
  try {
    const parsed = new URL(url);
    parsed.protocol = 'https:';
    return parsed.href;
  } catch {
    return url;
  }
}

/**
 * iframe `sandbox` attribute for fallback browsing (non-Tauri / dev).
 * Omits `allow-popups` when popup blocking is enabled.
 */
export function iframeSandboxAttr(blockPopups: boolean): string {
  const base = 'allow-same-origin allow-scripts allow-forms';
  return blockPopups ? base : `${base} allow-popups`;
}

/**
 * Whether tab session snapshots should be written or restored.
 * Private mode never persists open tabs to disk.
 */
export function shouldPersistSession(sessionRestore: boolean, privateMode: boolean): boolean {
  return sessionRestore && !privateMode;
}
