/**
 * Exodus Browser — persisted settings types (mirrors Tauri ExodusConfig).
 */

/** localStorage key for bookmark bar visibility. */
export const SHOW_BOOKMARK_BAR_KEY = 'exodus-show-bookmark-bar';

/** Read persisted bookmark bar visibility (default: visible). */
export function readShowBookmarkBar(): boolean {
  if (typeof localStorage === 'undefined') return true;
  return localStorage.getItem(SHOW_BOOKMARK_BAR_KEY) !== 'false';
}

/** Persist bookmark bar visibility. */
export function writeShowBookmarkBar(show: boolean): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(SHOW_BOOKMARK_BAR_KEY, show ? 'true' : 'false');
}

/** Full config from `get_ai_config`. */
export type ExodusConfigDto = {
  ai_port: number;
  ai_model: string;
  embedding_model: string;
  homepage_url: string;
  search_engine_url: string;
  status_clear_ms: number;
  spawn_sidecar: boolean;
  spawn_allama?: boolean;
  extension_store_url?: string;
  require_crx_signature?: boolean;
  confirm_host_permissions_on_install?: boolean;
};

/** Sidecar status from `get_sidecar_status` / `restart_sidecar`. */
export type SidecarStatusDto = {
  state: 'disabled' | 'not_found' | 'spawn_failed' | 'running' | 'exited' | 'unknown' | string;
  port: number;
  detail: string;
  endpointOnline: boolean;
};

/** Human-readable sidecar state label for settings UI. */
export function sidecarStateLabel(state: string): string {
  switch (state) {
    case 'disabled':
      return 'Disabled';
    case 'not_found':
      return 'Binary not found';
    case 'spawn_failed':
      return 'Failed to start';
    case 'running':
      return 'Running';
    case 'exited':
      return 'Exited';
    default:
      return 'Unknown';
  }
}
