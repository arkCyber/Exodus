/**
 * Exodus Browser — per-host "never save password" (localStorage).
 */

const STORAGE_KEY = 'exodus-password-never-save-hosts';

/** Hostnames where the save-password prompt is suppressed. */
export function getNeverSaveHosts(): string[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    return Array.isArray(parsed) ? parsed.filter((h) => typeof h === 'string') : [];
  } catch {
    return [];
  }
}

/** Whether save-password should be skipped for this URL. */
export function isNeverSavePasswordUrl(url: string): boolean {
  try {
    const host = new URL(url).hostname.toLowerCase();
    return getNeverSaveHosts().includes(host);
  } catch {
    return false;
  }
}

/** Remember never to offer save for this URL's host. */
export function addNeverSavePasswordHost(url: string): void {
  try {
    const host = new URL(url).hostname.toLowerCase();
    const hosts = new Set(getNeverSaveHosts());
    hosts.add(host);
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...hosts]));
  } catch (error) {
    console.error('addNeverSavePasswordHost failed:', error);
  }
}
