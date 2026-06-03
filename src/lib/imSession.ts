/**
 * Exodus Browser — local IM identity (P2P node id + display name).
 */
import { videoRtcNodeInfo } from '$lib/videoRtc';

const DISPLAY_NAME_KEY = 'exodus-im-display-name';
const LEGACY_USER_ID = 'exodus-local-user';

export type LocalImIdentity = {
  /** Group-chat member id (stable per device). */
  userId: string;
  displayName: string;
  nodeId: string;
};

/** Persisted display name for outgoing messages and calls. */
export function getSavedDisplayName(): string {
  if (typeof localStorage === 'undefined') return 'You';
  return localStorage.getItem(DISPLAY_NAME_KEY)?.trim() || 'You';
}

export function setSavedDisplayName(name: string): void {
  if (typeof localStorage === 'undefined') return;
  const v = name.trim() || 'You';
  localStorage.setItem(DISPLAY_NAME_KEY, v);
}

/**
 * Resolve node id from video RTC and a stable group-chat user id.
 * `userId` stays `exodus-local-user` for backward compatibility with existing groups.
 */
export async function resolveLocalIdentity(): Promise<LocalImIdentity> {
  const node = await videoRtcNodeInfo();
  const displayName = getSavedDisplayName();
  return {
    userId: LEGACY_USER_ID,
    displayName,
    nodeId: node.nodeId,
  };
}

/** True when a sender id looks like a P2P node (not the legacy local alias). */
export function isLikelyPeerNodeId(senderId: string): boolean {
  if (!senderId || senderId === LEGACY_USER_ID) return false;
  if (senderId.startsWith('exodus-') && senderId.length < 20) return false;
  return senderId.length >= 8;
}
