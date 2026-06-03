/**
 * Exodus Browser — WebRTC ICE / media defaults for P2P calls and meetings.
 */

const TURN_STORAGE_KEY = 'exodus-turn-ice-servers';

/** Public STUN servers (add TURN in production for strict NAT). */
export const DEFAULT_ICE_SERVERS: RTCIceServer[] = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
];

export const DEFAULT_PC_CONFIG: RTCConfiguration = {
  iceServers: DEFAULT_ICE_SERVERS,
};

/** Load optional TURN URLs from localStorage (one URL per line). */
export function loadCustomTurnUrls(): string[] {
  if (typeof localStorage === 'undefined') return [];
  const raw = localStorage.getItem(TURN_STORAGE_KEY) ?? '';
  return raw
    .split('\n')
    .map((s) => s.trim())
    .filter(Boolean);
}

export function saveCustomTurnUrls(urls: string[]): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(TURN_STORAGE_KEY, urls.filter(Boolean).join('\n'));
}

/** STUN defaults plus any saved TURN entries. */
export function buildIceServers(turnUrls: string[] = loadCustomTurnUrls()): RTCIceServer[] {
  const servers: RTCIceServer[] = [...DEFAULT_ICE_SERVERS];
  for (const url of turnUrls) {
    servers.push({ urls: url });
  }
  return servers;
}

export function getRtcConfiguration(): RTCConfiguration {
  return { iceServers: buildIceServers() };
}
